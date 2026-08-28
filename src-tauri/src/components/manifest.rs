use serde::Deserialize;
use std::{collections::HashSet, path::Path};

use crate::error::AppError;

const MANIFEST_JSON: &str = include_str!("../../components.json");

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentManifest {
    pub schema_version: u32,
    pub components: Vec<ComponentArtifact>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentKind {
    Whisper,
    Model,
    Ffmpeg,
}

impl ComponentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Whisper => "whisper",
            Self::Model => "model",
            Self::Ffmpeg => "ffmpeg",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveType {
    File,
    Zip,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentArtifact {
    pub id: String,
    pub kind: ComponentKind,
    pub display_name: String,
    pub version: String,
    pub recommended: bool,
    pub download_url: String,
    pub sha256: String,
    pub archive_type: ArchiveType,
    pub entrypoint: String,
    pub install_directory: String,
    pub download_size_bytes: u64,
    pub required_space_bytes: u64,
    pub source_url: String,
    pub license: String,
}

impl ComponentManifest {
    pub fn load() -> Result<Self, AppError> {
        let manifest: Self = serde_json::from_str(MANIFEST_JSON)
            .map_err(|_| AppError::new("コンポーネント定義を読み取れませんでした。"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn find(&self, id: &str) -> Result<&ComponentArtifact, AppError> {
        self.components
            .iter()
            .find(|component| component.id == id)
            .ok_or_else(|| AppError::new("指定されたコンポーネントは管理対象ではありません。"))
    }

    fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != 1 {
            return Err(AppError::new("未対応のコンポーネント定義です。"));
        }
        let mut ids = HashSet::new();
        for component in &self.components {
            let safe_directory = Path::new(&component.install_directory)
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_)));
            let safe_entrypoint = Path::new(&component.entrypoint)
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_)));
            if !ids.insert(&component.id)
                || !component.download_url.starts_with("https://")
                || !component.source_url.starts_with("https://")
                || component.sha256.len() != 64
                || !component
                    .sha256
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
                || !safe_directory
                || !safe_entrypoint
                || component.download_size_bytes == 0
                || component.required_space_bytes < component.download_size_bytes
            {
                return Err(AppError::new(
                    "コンポーネント定義の安全性を確認できませんでした。",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentKind, ComponentManifest};

    #[test]
    fn bundled_manifest_is_valid_and_has_recommended_set() {
        let manifest = ComponentManifest::load().unwrap();
        assert!(manifest.find("whisper").is_ok());
        assert!(manifest.find("ffmpeg").is_ok());
        assert!(manifest
            .components
            .iter()
            .any(|item| item.kind == ComponentKind::Model && item.recommended));
    }
}
