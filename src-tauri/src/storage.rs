use std::{fs, path::Path};

use crate::error::AppError;

pub fn save_text(path: &Path, text: &str) -> Result<(), AppError> {
    if text.trim().is_empty() {
        return Err(AppError::new("保存する文字起こし結果がありません。"));
    }
    fs::write(path, text.as_bytes())
        .map_err(|_| AppError::new("TXTファイルを保存できませんでした。保存先を確認してください。"))
}
