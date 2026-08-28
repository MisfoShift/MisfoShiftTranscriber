use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub whisper_path: String,
    pub model_path: String,
    pub ffmpeg_path: String,
    pub language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            whisper_path: String::new(),
            model_path: String::new(),
            ffmpeg_path: String::new(),
            language: "ja".into(),
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::new("設定の保存先を取得できませんでした。"))?;
    fs::create_dir_all(&directory)
        .map_err(|_| AppError::new("設定の保存先を作成できませんでした。"))?;
    Ok(directory.join("settings.json"))
}

pub fn load(app: &AppHandle) -> Result<AppSettings, AppError> {
    let path = settings_path(app)?;
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(|_| {
            AppError::new("設定ファイルを読み取れませんでした。設定をリセットしてください。")
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(AppSettings::default()),
        Err(_) => Err(AppError::new("設定ファイルを開けませんでした。")),
    }
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(settings)
        .map_err(|_| AppError::new("設定を変換できませんでした。"))?;
    fs::write(settings_path(app)?, content)
        .map_err(|_| AppError::new("設定を保存できませんでした。"))
}

pub fn clear(app: &AppHandle) -> Result<AppSettings, AppError> {
    let defaults = AppSettings::default();
    save(app, &defaults)?;
    Ok(defaults)
}
