mod download;
mod manifest;
mod paths;

use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::AppHandle;

use crate::{
    engine,
    error::AppError,
    settings::{self, AppSettings},
};

use manifest::{ComponentArtifact, ComponentKind, ComponentManifest};
use paths::ComponentPaths;

#[derive(Default)]
pub struct ComponentManager {
    active: Arc<Mutex<Option<String>>>,
}

struct OperationGuard {
    active: Arc<Mutex<Option<String>>>,
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        *self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;
    }
}

impl ComponentManager {
    fn begin(&self, operation: impl Into<String>) -> Result<OperationGuard, AppError> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if active.is_some() {
            return Err(AppError::new(
                "別のコンポーネント処理を実行中です。完了してから再試行してください。",
            ));
        }
        *active = Some(operation.into());
        Ok(OperationGuard {
            active: Arc::clone(&self.active),
        })
    }

    fn current(&self) -> Option<String> {
        self.active
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ManagedComponentStatus {
    pub id: String,
    pub kind: String,
    pub display_name: String,
    pub version: String,
    pub recommended: bool,
    pub installed: bool,
    pub managed: bool,
    pub current: bool,
    pub download_size_bytes: u64,
    pub download_size_label: String,
    pub file_name: String,
    pub source_url: String,
    pub license: String,
    pub installation_source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentOverview {
    pub components: Vec<ManagedComponentStatus>,
    pub storage_directory: String,
    pub busy_component: Option<String>,
}

pub fn overview(
    app: &AppHandle,
    manager: &ComponentManager,
) -> Result<ComponentOverview, AppError> {
    let manifest = ComponentManifest::load()?;
    let paths = ComponentPaths::new(app)?;
    let settings = settings::load(app)?;
    let engine_status = engine::status(&settings);
    let components = manifest
        .components
        .iter()
        .map(|artifact| component_status(artifact, &paths, &settings, &engine_status))
        .collect();
    Ok(ComponentOverview {
        components,
        storage_directory: paths.root().to_string_lossy().into_owned(),
        busy_component: manager.current(),
    })
}

fn component_status(
    artifact: &ComponentArtifact,
    paths: &ComponentPaths,
    settings: &AppSettings,
    engine_status: &engine::EngineStatus,
) -> ManagedComponentStatus {
    let managed_entrypoint = paths.installed_entrypoint(artifact);
    let resolved = match artifact.kind {
        ComponentKind::Whisper => engine_status.whisper_path.as_deref(),
        ComponentKind::Ffmpeg => engine_status.ffmpeg_path.as_deref(),
        ComponentKind::Model => None,
    };
    let (installed, current) = if artifact.kind == ComponentKind::Model {
        let managed_current = managed_entrypoint
            .as_ref()
            .is_some_and(|path| same_path_string(&settings.model_path, path));
        let matching_manual = Path::new(&settings.model_path)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(&artifact.entrypoint))
            && Path::new(&settings.model_path).is_file();
        (
            managed_entrypoint.is_some() || matching_manual,
            managed_current || matching_manual,
        )
    } else {
        let installed =
            resolved.is_some_and(|path| Path::new(path).is_file()) || managed_entrypoint.is_some();
        let current = managed_entrypoint
            .as_ref()
            .is_some_and(|managed| resolved.is_some_and(|path| same_path_string(path, managed)));
        (
            installed,
            current || (installed && managed_entrypoint.is_none()),
        )
    };
    let managed = managed_entrypoint.is_some();
    ManagedComponentStatus {
        id: artifact.id.clone(),
        kind: artifact.kind.as_str().into(),
        display_name: artifact.display_name.clone(),
        version: artifact.version.clone(),
        recommended: artifact.recommended,
        installed,
        managed,
        current,
        download_size_bytes: artifact.download_size_bytes,
        download_size_label: format_size(artifact.download_size_bytes),
        file_name: artifact.entrypoint.clone(),
        source_url: artifact.source_url.clone(),
        license: artifact.license.clone(),
        installation_source: if managed {
            "アプリ管理".into()
        } else if installed {
            "手動指定または同梱".into()
        } else {
            "未インストール".into()
        },
    }
}

fn same_path_string(value: &str, path: &Path) -> bool {
    if value.trim().is_empty() {
        return false;
    }
    let left = PathBuf::from(value);
    match (left.canonicalize(), path.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == path,
    }
}

fn format_size(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes as f64 >= GIB {
        format!("約{:.1} GB", bytes as f64 / GIB)
    } else {
        format!("約{} MB", (bytes as f64 / MIB).round() as u64)
    }
}

pub async fn install_component(
    app: &AppHandle,
    manager: &ComponentManager,
    component_id: &str,
) -> Result<ComponentOverview, AppError> {
    let guard = manager.begin(component_id)?;
    let manifest = ComponentManifest::load()?;
    let artifact = manifest.find(component_id)?.clone();
    let paths = ComponentPaths::new(app)?;
    let installed = download::install(app, &paths, &artifact).await?;
    apply_installed_path(app, &artifact, &installed, false)?;
    drop(guard);
    overview(app, manager)
}

pub async fn setup_recommended(
    app: &AppHandle,
    manager: &ComponentManager,
) -> Result<ComponentOverview, AppError> {
    let guard = manager.begin("recommended-setup")?;
    let manifest = ComponentManifest::load()?;
    let paths = ComponentPaths::new(app)?;
    let mut current_settings = settings::load(app)?;
    let mut current_engine = engine::status(&current_settings);

    for id in ["whisper", "model-small", "ffmpeg"] {
        let artifact = manifest.find(id)?.clone();
        let already_available = match artifact.kind {
            ComponentKind::Whisper => current_engine.whisper_path.is_some(),
            ComponentKind::Model => {
                !current_settings.model_path.trim().is_empty()
                    && Path::new(&current_settings.model_path).is_file()
            }
            ComponentKind::Ffmpeg => current_engine.ffmpeg_path.is_some(),
        };
        if already_available {
            continue;
        }
        let installed = if let Some(existing) = paths.installed_entrypoint(&artifact) {
            existing
        } else {
            download::install(app, &paths, &artifact).await?
        };
        apply_installed_path(app, &artifact, &installed, true)?;
        current_settings = settings::load(app)?;
        current_engine = engine::status(&current_settings);
    }
    drop(guard);
    overview(app, manager)
}

fn apply_installed_path(
    app: &AppHandle,
    artifact: &ComponentArtifact,
    installed: &Path,
    force_model: bool,
) -> Result<(), AppError> {
    let mut app_settings = settings::load(app)?;
    let path = installed.to_string_lossy().into_owned();
    match artifact.kind {
        ComponentKind::Whisper => app_settings.whisper_path = path,
        ComponentKind::Ffmpeg => app_settings.ffmpeg_path = path,
        ComponentKind::Model
            if force_model
                || app_settings.model_path.trim().is_empty()
                || !Path::new(&app_settings.model_path).is_file() =>
        {
            app_settings.model_path = path;
        }
        ComponentKind::Model => {}
    }
    settings::save(app, &app_settings)
}

pub fn select_model(
    app: &AppHandle,
    manager: &ComponentManager,
    component_id: &str,
) -> Result<ComponentOverview, AppError> {
    let guard = manager.begin(component_id)?;
    let manifest = ComponentManifest::load()?;
    let artifact = manifest.find(component_id)?;
    if artifact.kind != ComponentKind::Model {
        return Err(AppError::new("このコンポーネントはモデルではありません。"));
    }
    let paths = ComponentPaths::new(app)?;
    let model = paths.installed_entrypoint(artifact).ok_or_else(|| {
        AppError::new("モデルがインストールされていません。先にダウンロードしてください。")
    })?;
    apply_installed_path(app, artifact, &model, true)?;
    drop(guard);
    overview(app, manager)
}

pub fn delete_model(
    app: &AppHandle,
    manager: &ComponentManager,
    component_id: &str,
) -> Result<ComponentOverview, AppError> {
    let guard = manager.begin(component_id)?;
    let manifest = ComponentManifest::load()?;
    let artifact = manifest.find(component_id)?;
    if artifact.kind != ComponentKind::Model {
        return Err(AppError::new("削除できるのはアプリ管理のモデルだけです。"));
    }
    let paths = ComponentPaths::new(app)?;
    let directory = paths.install_directory(artifact);
    let managed_model = paths.installed_entrypoint(artifact);
    if !directory.exists() {
        return Err(AppError::new(
            "このモデルはアプリ管理領域にインストールされていません。",
        ));
    }
    let mut app_settings = settings::load(app)?;
    if managed_model
        .as_ref()
        .is_some_and(|path| same_path_string(&app_settings.model_path, path))
    {
        app_settings.model_path.clear();
        settings::save(app, &app_settings)?;
    }
    fs::remove_dir_all(&directory).map_err(|_| {
        AppError::new(
            "モデルを削除できませんでした。使用中でないか、書き込み権限があるか確認してください。",
        )
    })?;
    drop(guard);
    overview(app, manager)
}
