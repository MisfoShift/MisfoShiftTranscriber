use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, State};

use crate::{
    audio::{self, MicrophoneDevice, MicrophoneManager},
    components::{self, ComponentManager, ComponentOverview},
    engine::{self, EngineStatus},
    error::AppError,
    settings::{self, AppSettings},
    storage,
};

#[tauri::command]
pub fn component_overview(
    app: AppHandle,
    manager: State<'_, ComponentManager>,
) -> Result<ComponentOverview, AppError> {
    components::overview(&app, &manager)
}

#[tauri::command]
pub async fn install_managed_component(
    app: AppHandle,
    manager: State<'_, ComponentManager>,
    component_id: String,
) -> Result<ComponentOverview, AppError> {
    components::install_component(&app, &manager, &component_id).await
}

#[tauri::command]
pub async fn setup_recommended_components(
    app: AppHandle,
    manager: State<'_, ComponentManager>,
) -> Result<ComponentOverview, AppError> {
    components::setup_recommended(&app, &manager).await
}

#[tauri::command]
pub fn select_managed_model(
    app: AppHandle,
    manager: State<'_, ComponentManager>,
    component_id: String,
) -> Result<ComponentOverview, AppError> {
    components::select_model(&app, &manager, &component_id)
}

#[tauri::command]
pub fn delete_managed_model(
    app: AppHandle,
    manager: State<'_, ComponentManager>,
    component_id: String,
) -> Result<ComponentOverview, AppError> {
    components::delete_model(&app, &manager, &component_id)
}

#[tauri::command]
pub fn list_microphones() -> Result<Vec<MicrophoneDevice>, AppError> {
    audio::list_input_devices()
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Result<AppSettings, AppError> {
    settings::load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<EngineStatus, AppError> {
    settings::save(&app, &settings)?;
    Ok(engine::status(&settings))
}

#[tauri::command]
pub fn clear_saved_settings(app: AppHandle) -> Result<AppSettings, AppError> {
    settings::clear(&app)
}

#[tauri::command]
pub fn engine_status(app: AppHandle) -> Result<EngineStatus, AppError> {
    Ok(engine::status(&settings::load(&app)?))
}

#[tauri::command]
pub fn start_microphone(
    app: AppHandle,
    manager: State<'_, MicrophoneManager>,
    device_id: String,
) -> Result<(), AppError> {
    let settings = settings::load(&app)?;
    if !engine::status(&settings).ready {
        return Err(AppError::new(engine::status(&settings).message));
    }
    manager.start(app, device_id, settings)
}

#[tauri::command]
pub fn stop_microphone(manager: State<'_, MicrophoneManager>) -> Result<(), AppError> {
    manager.stop()
}

#[tauri::command]
pub async fn transcribe_audio_file(
    app: AppHandle,
    manager: State<'_, MicrophoneManager>,
    path: String,
) -> Result<String, AppError> {
    if manager.is_running() {
        return Err(AppError::new(
            "音声ファイルを処理する前に、マイク文字起こしを停止してください。",
        ));
    }
    let settings = settings::load(&app)?;
    let status = engine::status(&settings);
    if !status.file_ready {
        return Err(AppError::new(status.message));
    }

    let _ = app.emit("file-progress", "音声を文字起こし用に変換しています…");
    let input = PathBuf::from(path);
    let result =
        tauri::async_runtime::spawn_blocking(move || engine::transcribe_file(&settings, &input))
            .await
            .map_err(|_| AppError::new("文字起こし処理を完了できませんでした。"))?;
    let _ = app.emit("file-progress", "");
    result
}

#[tauri::command]
pub fn save_text_file(path: String, text: String) -> Result<(), AppError> {
    storage::save_text(Path::new(&path), &text)
}
