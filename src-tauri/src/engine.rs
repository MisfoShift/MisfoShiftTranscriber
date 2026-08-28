use serde::Serialize;
use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use crate::{error::AppError, settings::AppSettings};

const TARGET_SAMPLE_RATE: u32 = 16_000;

#[derive(Debug, Clone, Serialize)]
pub struct EngineStatus {
    pub ready: bool,
    pub file_ready: bool,
    pub whisper_path: Option<String>,
    pub model_path: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone)]
struct ResolvedEngine {
    whisper: PathBuf,
    model: PathBuf,
    ffmpeg: Option<PathBuf>,
    language: String,
}

#[cfg(windows)]
fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
fn hide_console(_command: &mut Command) {}

fn existing_file(value: &str) -> Option<PathBuf> {
    if value.trim().is_empty() {
        return None;
    }
    let path = PathBuf::from(value.trim());
    path.is_file().then_some(path)
}

fn search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(current) = env::current_dir() {
        roots.push(current);
    }
    if let Ok(executable) = env::current_exe() {
        if let Some(parent) = executable.parent() {
            roots.push(parent.to_path_buf());
        }
    }
    roots
}

fn find_relative(candidates: &[&str]) -> Option<PathBuf> {
    search_roots()
        .into_iter()
        .flat_map(|root| candidates.iter().map(move |candidate| root.join(candidate)))
        .find(|path| path.is_file())
}

fn find_on_path(names: &[&str]) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    for directory in env::split_paths(&paths) {
        for name in names {
            let candidate = directory.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn resolve_whisper(settings: &AppSettings) -> Option<PathBuf> {
    existing_file(&settings.whisper_path)
        .or_else(|| {
            find_relative(&[
                "tools/whisper-cli.exe",
                "tools/main.exe",
                "whisper-cli.exe",
                "whisper.cpp/build/bin/Release/whisper-cli.exe",
            ])
        })
        .or_else(|| find_on_path(&["whisper-cli.exe", "whisper-cli", "main.exe", "main"]))
}

fn resolve_model(settings: &AppSettings) -> Option<PathBuf> {
    existing_file(&settings.model_path).or_else(|| {
        find_relative(&[
            "models/ggml-small.bin",
            "models/ggml-base.bin",
            "models/ggml-medium.bin",
            "ggml-small.bin",
        ])
    })
}

fn resolve_ffmpeg(settings: &AppSettings) -> Option<PathBuf> {
    existing_file(&settings.ffmpeg_path)
        .or_else(|| {
            find_relative(&[
                "tools/ffmpeg.exe",
                "src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe",
                "ffmpeg-x86_64-pc-windows-msvc.exe",
                "ffmpeg.exe",
            ])
        })
        .or_else(|| find_on_path(&["ffmpeg.exe", "ffmpeg"]))
}

pub fn status(settings: &AppSettings) -> EngineStatus {
    let whisper = resolve_whisper(settings);
    let model = resolve_model(settings);
    let ffmpeg = resolve_ffmpeg(settings);
    let mut missing = Vec::new();
    if whisper.is_none() {
        missing.push("whisper-cli.exe");
    }
    if model.is_none() {
        missing.push("Whisperモデル");
    }
    let ready = whisper.is_some() && model.is_some();
    let file_ready = ready && ffmpeg.is_some();
    let message = if missing.is_empty() {
        if ffmpeg.is_some() {
            "ローカル文字起こしを利用できます。".into()
        } else {
            "マイク文字起こしを利用できます。音声ファイルにはFFmpegの設定が必要です。".into()
        }
    } else {
        format!("{}を設定してください。", missing.join(" と "))
    };
    EngineStatus {
        ready,
        file_ready,
        whisper_path: whisper.map(|path| path.to_string_lossy().into_owned()),
        model_path: model.map(|path| path.to_string_lossy().into_owned()),
        ffmpeg_path: ffmpeg.map(|path| path.to_string_lossy().into_owned()),
        message,
    }
}

fn resolve(settings: &AppSettings, needs_ffmpeg: bool) -> Result<ResolvedEngine, AppError> {
    let whisper = resolve_whisper(settings).ok_or_else(|| {
        AppError::new("whisper-cli.exeが見つかりません。設定から実行ファイルを選択してください。")
    })?;
    let model = resolve_model(settings).ok_or_else(|| {
        AppError::new("Whisperモデルが見つかりません。設定からggmlモデルを選択してください。")
    })?;
    let ffmpeg = resolve_ffmpeg(settings);
    if needs_ffmpeg && ffmpeg.is_none() {
        return Err(AppError::new(
            "FFmpegが見つかりません。音声ファイルを変換するため、設定からffmpeg.exeを選択してください。",
        ));
    }
    Ok(ResolvedEngine {
        whisper,
        model,
        ffmpeg,
        language: if settings.language.trim().is_empty() {
            "ja".into()
        } else {
            settings.language.trim().into()
        },
    })
}

fn command_output(program: &OsStr, arguments: &[&OsStr]) -> Result<Output, AppError> {
    let mut command = Command::new(program);
    hide_console(&mut command);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|_| AppError::new("ローカル文字起こしプログラムを起動できませんでした。"))
}

fn error_tail(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let compact = text
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    compact.chars().take(240).collect()
}

fn clean_transcript(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(|line| {
            let mut text = line.trim();
            if text.starts_with('[') && text.contains(" --> ") {
                text = text
                    .split_once(']')
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
            }
            if text.is_empty()
                || text.starts_with("whisper_")
                || text.starts_with("main:")
                || text.starts_with("system_info:")
            {
                None
            } else {
                Some(text)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn run_whisper(engine: &ResolvedEngine, wav_path: &Path) -> Result<String, AppError> {
    let output = command_output(
        engine.whisper.as_os_str(),
        &[
            OsStr::new("-m"),
            engine.model.as_os_str(),
            OsStr::new("-f"),
            wav_path.as_os_str(),
            OsStr::new("-l"),
            OsStr::new(&engine.language),
            OsStr::new("-nt"),
        ],
    )?;
    if !output.status.success() {
        let detail = error_tail(&output.stderr);
        return Err(AppError::new(if detail.is_empty() {
            "Whisperによる文字起こしに失敗しました。".into()
        } else {
            format!("Whisperによる文字起こしに失敗しました: {detail}")
        }));
    }
    Ok(clean_transcript(&output.stdout))
}

fn convert_audio(engine: &ResolvedEngine, input: &Path, output: &Path) -> Result<(), AppError> {
    let ffmpeg = engine
        .ffmpeg
        .as_ref()
        .ok_or_else(|| AppError::new("FFmpegが見つかりません。"))?;
    let result = command_output(
        ffmpeg.as_os_str(),
        &[
            OsStr::new("-y"),
            OsStr::new("-v"),
            OsStr::new("error"),
            OsStr::new("-i"),
            input.as_os_str(),
            OsStr::new("-ar"),
            OsStr::new("16000"),
            OsStr::new("-ac"),
            OsStr::new("1"),
            OsStr::new("-c:a"),
            OsStr::new("pcm_s16le"),
            output.as_os_str(),
        ],
    )?;
    if result.status.success() {
        Ok(())
    } else {
        let detail = error_tail(&result.stderr);
        Err(AppError::new(if detail.is_empty() {
            "音声ファイルを読み込めませんでした。".into()
        } else {
            format!("音声ファイルを変換できませんでした: {detail}")
        }))
    }
}

pub fn transcribe_file(settings: &AppSettings, input: &Path) -> Result<String, AppError> {
    if !input.is_file() {
        return Err(AppError::new("選択した音声ファイルが見つかりません。"));
    }
    let engine = resolve(settings, true)?;
    let temporary =
        tempfile::tempdir().map_err(|_| AppError::new("一時ファイルを作成できませんでした。"))?;
    let wav_path = temporary.path().join("input.wav");
    convert_audio(&engine, input, &wav_path)?;
    run_whisper(&engine, &wav_path)
}

fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Vec<f32> {
    if samples.is_empty() || source_rate == 0 {
        return Vec::new();
    }
    if source_rate == TARGET_SAMPLE_RATE {
        return samples.to_vec();
    }
    let output_len =
        ((samples.len() as u64 * TARGET_SAMPLE_RATE as u64) / source_rate as u64) as usize;
    let scale = source_rate as f64 / TARGET_SAMPLE_RATE as f64;
    (0..output_len)
        .map(|index| {
            let position = index as f64 * scale;
            let left = position.floor() as usize;
            let right = (left + 1).min(samples.len() - 1);
            let fraction = (position - left as f64) as f32;
            samples[left] * (1.0 - fraction) + samples[right] * fraction
        })
        .collect()
}

fn write_wav(path: &Path, samples: &[f32]) -> Result<(), AppError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|_| AppError::new("マイク音声の一時ファイルを作成できませんでした。"))?;
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer
            .write_sample(value)
            .map_err(|_| AppError::new("マイク音声を一時ファイルへ書き込めませんでした。"))?;
    }
    writer
        .finalize()
        .map_err(|_| AppError::new("マイク音声の一時ファイルを確定できませんでした。"))
}

pub fn transcribe_samples(
    settings: &AppSettings,
    samples: &[f32],
    source_rate: u32,
) -> Result<String, AppError> {
    let engine = resolve(settings, false)?;
    let samples = resample_to_16khz(samples, source_rate);
    let temporary =
        tempfile::tempdir().map_err(|_| AppError::new("一時ファイルを作成できませんでした。"))?;
    let wav_path = temporary.path().join("microphone.wav");
    write_wav(&wav_path, &samples)?;
    run_whisper(&engine, &wav_path)
}

pub fn has_audible_signal(samples: &[f32]) -> bool {
    if samples.is_empty() {
        return false;
    }
    let power = samples
        .iter()
        .map(|sample| (*sample as f64) * (*sample as f64))
        .sum::<f64>()
        / samples.len() as f64;
    power.sqrt() >= 0.004
}

#[cfg(test)]
mod tests {
    use super::{clean_transcript, has_audible_signal, resample_to_16khz};

    #[test]
    fn removes_whisper_timestamps() {
        let text = clean_transcript(b"[00:00:00.000 --> 00:00:01.000]  hello\n");
        assert_eq!(text, "hello");
    }

    #[test]
    fn resamples_audio_to_sixteen_kilohertz() {
        let source = vec![0.0; 48_000];
        assert_eq!(resample_to_16khz(&source, 48_000).len(), 16_000);
    }

    #[test]
    fn ignores_quiet_microphone_chunks() {
        assert!(!has_audible_signal(&vec![0.001; 16_000]));
        assert!(has_audible_signal(&vec![0.1; 16_000]));
    }
}
