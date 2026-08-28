use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

use crate::error::AppError;

use super::manifest::ComponentArtifact;

#[derive(Debug, Clone)]
pub struct ComponentPaths {
    root: PathBuf,
}

impl ComponentPaths {
    pub fn new(app: &AppHandle) -> Result<Self, AppError> {
        let root = app
            .path()
            .app_data_dir()
            .map_err(|_| AppError::new("コンポーネントの保存先を取得できませんでした。"))?
            .join("components");
        fs::create_dir_all(&root).map_err(|_| {
            AppError::new(
                "コンポーネントの保存先を作成できませんでした。書き込み権限を確認してください。",
            )
        })?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn install_directory(&self, artifact: &ComponentArtifact) -> PathBuf {
        self.root.join(&artifact.install_directory)
    }

    pub fn installed_entrypoint(&self, artifact: &ComponentArtifact) -> Option<PathBuf> {
        find_named_file(&self.install_directory(artifact), &artifact.entrypoint)
    }
}

pub fn find_named_file(root: &Path, file_name: &str) -> Option<PathBuf> {
    if !root.is_dir() {
        return None;
    }
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        let entries = fs::read_dir(directory).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(file_name))
            {
                return Some(path);
            }
        }
    }
    None
}
