use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

use crate::error::AppError;

use super::{
    manifest::{ArchiveType, ComponentArtifact},
    paths::{find_named_file, ComponentPaths},
};

const MIN_DOWNLOAD_TOLERANCE_BYTES: u64 = 8 * 1024 * 1024;
const DOWNLOAD_TOLERANCE_DIVISOR: u64 = 20;

#[derive(Debug, Clone, Serialize)]
pub struct ComponentProgress {
    pub component_id: String,
    pub stage: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub message: String,
}

fn emit_progress(
    app: &AppHandle,
    artifact: &ComponentArtifact,
    stage: &str,
    downloaded: u64,
    total: u64,
    message: impl Into<String>,
) {
    let percent = if total > 0 {
        (downloaded as f64 / total as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let _ = app.emit(
        "component-progress",
        ComponentProgress {
            component_id: artifact.id.clone(),
            stage: stage.into(),
            downloaded_bytes: downloaded,
            total_bytes: total,
            percent,
            message: message.into(),
        },
    );
}

pub async fn install(
    app: &AppHandle,
    paths: &ComponentPaths,
    artifact: &ComponentArtifact,
) -> Result<PathBuf, AppError> {
    let available = fs2::available_space(paths.root())
        .map_err(|_| AppError::new("保存先の空き容量を確認できませんでした。"))?;
    if available < artifact.required_space_bytes {
        return Err(AppError::new(format!(
            "{}の保存に必要な空き容量が不足しています。空き容量を増やしてから再試行してください。",
            artifact.display_name
        )));
    }

    let temporary = tempfile::Builder::new()
        .prefix(".install-")
        .tempdir_in(paths.root())
        .map_err(|_| {
            AppError::new("一時保存領域を作成できませんでした。書き込み権限を確認してください。")
        })?;
    let download_path = temporary.path().join("download.part");
    download(app, artifact, &download_path).await?;

    emit_progress(
        app,
        artifact,
        "verifying",
        0,
        0,
        "ファイルの整合性を確認しています…",
    );
    verify_sha256(&download_path, &artifact.sha256)?;

    let payload = temporary.path().join("payload");
    fs::create_dir_all(&payload).map_err(|_| AppError::new("展開先を準備できませんでした。"))?;
    emit_progress(app, artifact, "installing", 0, 0, "安全に配置しています…");
    match artifact.archive_type {
        ArchiveType::File => {
            fs::rename(&download_path, payload.join(&artifact.entrypoint))
                .map_err(|_| AppError::new("ダウンロード済みファイルを配置できませんでした。"))?;
        }
        ArchiveType::Zip => {
            let archive_path = download_path.clone();
            let output_path = payload.clone();
            let maximum_size = artifact.required_space_bytes;
            tauri::async_runtime::spawn_blocking(move || {
                extract_zip(&archive_path, &output_path, maximum_size)
            })
            .await
            .map_err(|_| AppError::new("アーカイブの展開処理を完了できませんでした。"))??;
        }
    }

    let entrypoint = find_named_file(&payload, &artifact.entrypoint).ok_or_else(|| {
        AppError::new(format!(
            "取得したファイルに{}が含まれていません。コンポーネント定義の更新が必要です。",
            artifact.entrypoint
        ))
    })?;
    let relative_entrypoint = entrypoint
        .strip_prefix(&payload)
        .map_err(|_| AppError::new("配置するファイルを確認できませんでした。"))?
        .to_path_buf();

    let marker = serde_json::json!({
        "id": artifact.id,
        "version": artifact.version,
        "sha256": artifact.sha256,
        "source": artifact.download_url,
    });
    fs::write(
        payload.join(".component.json"),
        serde_json::to_vec_pretty(&marker).unwrap_or_default(),
    )
    .map_err(|_| AppError::new("インストール情報を書き込めませんでした。"))?;

    let destination = paths.install_directory(artifact);
    replace_directory(&payload, &destination, paths.root(), &artifact.id)?;
    let installed = destination.join(relative_entrypoint);
    emit_progress(
        app,
        artifact,
        "complete",
        1,
        1,
        "インストールが完了しました。",
    );
    Ok(installed)
}

async fn download(
    app: &AppHandle,
    artifact: &ComponentArtifact,
    destination: &Path,
) -> Result<(), AppError> {
    emit_progress(app, artifact, "connecting", 0, 0, "配布元へ接続しています…");
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("MisfoShiftTranscriber/0.1 component-manager")
        .build()
        .map_err(|_| AppError::new("ダウンロード機能を初期化できませんでした。"))?;
    let response = client
        .get(&artifact.download_url)
        .send()
        .await
        .map_err(|_| {
            AppError::new(
                "配布元へ接続できませんでした。インターネット接続を確認して再試行してください。",
            )
        })?;
    if !response.status().is_success() {
        return Err(AppError::new(format!(
            "ダウンロードに失敗しました（HTTP {}）。時間を置いて再試行してください。",
            response.status().as_u16()
        )));
    }
    let maximum_download_size = download_size_limit(artifact);
    if response
        .content_length()
        .is_some_and(|content_length| content_length > maximum_download_size)
    {
        return Err(download_size_error(maximum_download_size));
    }
    let total = response
        .content_length()
        .unwrap_or(artifact.download_size_bytes);
    let mut file = tokio::fs::File::create(destination).await.map_err(|_| {
        AppError::new(
            "一時ファイルを作成できませんでした。書き込み権限と空き容量を確認してください。",
        )
    })?;
    let mut stream = response.bytes_stream();
    let mut downloaded = 0_u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|_| AppError::new("ダウンロード中に接続が切れました。再試行してください。"))?;
        downloaded = match checked_downloaded_size(downloaded, chunk.len(), maximum_download_size) {
            Ok(size) => size,
            Err(error) => {
                drop(file);
                let _ = tokio::fs::remove_file(destination).await;
                return Err(error);
            }
        };
        file.write_all(&chunk).await.map_err(|_| {
            AppError::new(
                "ダウンロードを書き込めませんでした。空き容量と書き込み権限を確認してください。",
            )
        })?;
        emit_progress(
            app,
            artifact,
            "downloading",
            downloaded,
            total,
            format!("{}をダウンロードしています…", artifact.display_name),
        );
    }
    file.flush()
        .await
        .map_err(|_| AppError::new("ダウンロード済みファイルを確定できませんでした。"))?;
    Ok(())
}

fn download_size_limit(artifact: &ComponentArtifact) -> u64 {
    let tolerance = (artifact.download_size_bytes / DOWNLOAD_TOLERANCE_DIVISOR)
        .max(MIN_DOWNLOAD_TOLERANCE_BYTES);
    artifact
        .download_size_bytes
        .saturating_add(tolerance)
        .min(artifact.required_space_bytes)
}

fn checked_downloaded_size(
    downloaded: u64,
    chunk_size: usize,
    maximum_download_size: u64,
) -> Result<u64, AppError> {
    let chunk_size =
        u64::try_from(chunk_size).map_err(|_| download_size_error(maximum_download_size))?;
    let next = downloaded
        .checked_add(chunk_size)
        .ok_or_else(|| download_size_error(maximum_download_size))?;
    if next > maximum_download_size {
        Err(download_size_error(maximum_download_size))
    } else {
        Ok(next)
    }
}

fn download_size_error(maximum_download_size: u64) -> AppError {
    AppError::new(format!(
        "ダウンロード容量が安全上限（約{} MB）を超えたため中止しました。コンポーネント定義が最新か確認して再試行してください。",
        maximum_download_size / 1_000_000
    ))
}

fn verify_sha256(path: &Path, expected: &str) -> Result<(), AppError> {
    let mut file = fs::File::open(path)
        .map_err(|_| AppError::new("ダウンロード済みファイルを確認できませんでした。"))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| AppError::new("ダウンロード済みファイルを検査できませんでした。"))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    let actual = format!("{:x}", digest.finalize());
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(AppError::new(
            "ダウンロードしたファイルの整合性を確認できませんでした。破損または配布元の更新が考えられるため、使用せず削除しました。",
        ))
    }
}

fn extract_zip(archive_path: &Path, destination: &Path, maximum_size: u64) -> Result<(), AppError> {
    let file = fs::File::open(archive_path)
        .map_err(|_| AppError::new("ダウンロードしたZIPファイルを開けませんでした。"))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|_| AppError::new("ダウンロードしたZIPファイルが壊れています。"))?;
    let mut extracted_size = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| AppError::new("ZIPファイルの内容を読み取れませんでした。"))?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| AppError::new("安全でないパスを含むZIPファイルを拒否しました。"))?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(AppError::new(
                "シンボリックリンクを含むZIPファイルを拒否しました。",
            ));
        }
        extracted_size = extracted_size.saturating_add(entry.size());
        if extracted_size > maximum_size {
            return Err(AppError::new(
                "ZIPファイルの展開サイズが安全上限を超えています。",
            ));
        }
        let output = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&output)
                .map_err(|_| AppError::new("ZIPファイルの展開先を作成できませんでした。"))?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| AppError::new("ZIPファイルの展開先を作成できませんでした。"))?;
        }
        let mut output_file = fs::File::create(output)
            .map_err(|_| AppError::new("ZIPファイルを展開できませんでした。"))?;
        std::io::copy(&mut entry, &mut output_file)
            .map_err(|_| AppError::new("ZIPファイルの展開中に書き込みエラーが発生しました。"))?;
        output_file
            .flush()
            .map_err(|_| AppError::new("展開したファイルを確定できませんでした。"))?;
    }
    Ok(())
}

fn replace_directory(
    source: &Path,
    destination: &Path,
    component_root: &Path,
    id: &str,
) -> Result<(), AppError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| AppError::new("コンポーネントの配置先を作成できませんでした。"))?;
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let backup = component_root.join(format!(".backup-{id}-{timestamp}"));
    let had_existing = destination.exists();
    if had_existing {
        fs::rename(destination, &backup).map_err(|_| {
            AppError::new(
                "既存コンポーネントを更新準備できませんでした。使用中でないか確認してください。",
            )
        })?;
    }
    if let Err(error) = fs::rename(source, destination) {
        if had_existing {
            let _ = fs::rename(&backup, destination);
        }
        return Err(AppError::new(format!(
            "コンポーネントを正式配置できませんでした: {error}"
        )));
    }
    if had_existing {
        let _ = fs::remove_dir_all(backup);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{checked_downloaded_size, download_size_limit, verify_sha256};
    use crate::components::manifest::{ArchiveType, ComponentArtifact, ComponentKind};
    use std::io::Write;

    fn test_artifact(download_size_bytes: u64, required_space_bytes: u64) -> ComponentArtifact {
        ComponentArtifact {
            id: "test".into(),
            kind: ComponentKind::Ffmpeg,
            display_name: "Test".into(),
            version: "1".into(),
            recommended: false,
            download_url: "https://example.com/test.zip".into(),
            sha256: "0".repeat(64),
            archive_type: ArchiveType::Zip,
            entrypoint: "test.exe".into(),
            install_directory: "test/1".into(),
            download_size_bytes,
            required_space_bytes,
            source_url: "https://example.com".into(),
            license: "MIT".into(),
        }
    }

    #[test]
    fn verifies_sha256_before_installing() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"abc").unwrap();
        assert!(verify_sha256(
            file.path(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        )
        .is_ok());
        assert!(verify_sha256(file.path(), &"0".repeat(64)).is_err());
    }

    #[test]
    fn download_limit_allows_tolerance_but_not_required_space_overrun() {
        let small = test_artifact(100_000_000, 500_000_000);
        assert_eq!(download_size_limit(&small), 108_388_608);

        let constrained = test_artifact(100_000_000, 104_000_000);
        assert_eq!(download_size_limit(&constrained), 104_000_000);
    }

    #[test]
    fn rejects_chunk_that_crosses_download_limit() {
        assert_eq!(checked_downloaded_size(90, 10, 100).unwrap(), 100);
        assert!(checked_downloaded_size(100, 1, 100).is_err());
    }
}
