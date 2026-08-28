use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};

use crate::{engine, error::AppError, settings::AppSettings};

const PARTIAL_SECONDS: usize = 2;
const FINAL_SECONDS: usize = 8;

#[derive(Debug, Clone, Serialize)]
pub struct MicrophoneDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub struct MicrophoneManager {
    running: Arc<AtomicBool>,
    stop_requested: Arc<AtomicBool>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
}

impl Default for MicrophoneManager {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            stop_requested: Arc::new(AtomicBool::new(false)),
            worker: Mutex::new(None),
        }
    }
}

impl MicrophoneManager {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn start(
        &self,
        app: AppHandle,
        device_id: String,
        settings: AppSettings,
    ) -> Result<(), AppError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(AppError::new("マイク文字起こしはすでに実行中です。"));
        }

        if let Some(previous) = self.worker.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = previous.join();
        }
        self.stop_requested.store(false, Ordering::SeqCst);

        let stop_requested = Arc::clone(&self.stop_requested);
        let running = Arc::clone(&self.running);
        let worker_app = app.clone();
        let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<(), String>>(1);
        let worker = thread::spawn(move || {
            let result = capture_loop(
                worker_app.clone(),
                &device_id,
                settings,
                stop_requested,
                &ready_tx,
            );
            if let Err(error) = result {
                let message = error.to_string();
                let _ = ready_tx.try_send(Err(message.clone()));
                let _ = worker_app.emit("app-error", message);
            }
            running.store(false, Ordering::SeqCst);
            let _ = worker_app.emit("mic-state", false);
        });

        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => {
                *self.worker.lock().unwrap_or_else(|e| e.into_inner()) = Some(worker);
                let _ = app.emit("mic-state", true);
                Ok(())
            }
            Ok(Err(message)) => {
                let _ = worker.join();
                self.running.store(false, Ordering::SeqCst);
                Err(AppError::new(message))
            }
            Err(_) => {
                self.stop_requested.store(true, Ordering::SeqCst);
                let _ = worker.join();
                self.running.store(false, Ordering::SeqCst);
                Err(AppError::new("マイクの開始がタイムアウトしました。"))
            }
        }
    }

    pub fn stop(&self) -> Result<(), AppError> {
        if !self.is_running() {
            return Err(AppError::new("マイク文字起こしは実行されていません。"));
        }
        self.stop_requested.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub fn list_input_devices() -> Result<Vec<MicrophoneDevice>, AppError> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|device| device.name().ok());
    let devices = host
        .input_devices()
        .map_err(|_| AppError::new("マイクデバイスの一覧を取得できませんでした。"))?;
    let result = devices
        .enumerate()
        .filter_map(|(index, device)| {
            let name = device.name().ok()?;
            Some(MicrophoneDevice {
                id: index.to_string(),
                is_default: default_name.as_deref() == Some(name.as_str()),
                name,
            })
        })
        .collect::<Vec<_>>();
    if result.is_empty() {
        Err(AppError::new("利用できるマイクが見つかりませんでした。"))
    } else {
        Ok(result)
    }
}

fn selected_device(device_id: &str) -> Result<cpal::Device, AppError> {
    let index = device_id
        .parse::<usize>()
        .map_err(|_| AppError::new("選択したマイクが正しくありません。"))?;
    cpal::default_host()
        .input_devices()
        .map_err(|_| AppError::new("マイクデバイスを開けませんでした。"))?
        .nth(index)
        .ok_or_else(|| {
            AppError::new("選択したマイクが見つかりません。マイク一覧を更新してください。")
        })
}

fn append_mono(buffer: &Arc<Mutex<Vec<f32>>>, values: impl Iterator<Item = f32>) {
    if let Ok(mut target) = buffer.lock() {
        target.extend(
            values
                .filter(|value| value.is_finite())
                .map(|value| value.clamp(-1.0, 1.0)),
        );
    }
}

fn push_f32(buffer: &Arc<Mutex<Vec<f32>>>, data: &[f32], channels: usize) {
    append_mono(
        buffer,
        data.chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32),
    );
}

fn push_i16(buffer: &Arc<Mutex<Vec<f32>>>, data: &[i16], channels: usize) {
    append_mono(
        buffer,
        data.chunks(channels).map(|frame| {
            frame
                .iter()
                .map(|value| *value as f32 / i16::MAX as f32)
                .sum::<f32>()
                / frame.len() as f32
        }),
    );
}

fn push_u16(buffer: &Arc<Mutex<Vec<f32>>>, data: &[u16], channels: usize) {
    append_mono(
        buffer,
        data.chunks(channels).map(|frame| {
            frame
                .iter()
                .map(|value| (*value as f32 / u16::MAX as f32) * 2.0 - 1.0)
                .sum::<f32>()
                / frame.len() as f32
        }),
    );
}

fn emit_error(app: &AppHandle, message: impl Into<String>) {
    let _ = app.emit("app-error", message.into());
}

fn transcribe_snapshot(
    app: &AppHandle,
    settings: &AppSettings,
    samples: &[f32],
    sample_rate: u32,
    final_result: bool,
) {
    if !engine::has_audible_signal(samples) {
        if !final_result {
            let _ = app.emit("transcript-partial", "");
        }
        return;
    }
    match engine::transcribe_samples(settings, samples, sample_rate) {
        Ok(text) if final_result && !text.trim().is_empty() => {
            let _ = app.emit("transcript-final", text);
            let _ = app.emit("transcript-partial", "");
        }
        Ok(text) if !final_result => {
            let _ = app.emit("transcript-partial", text);
        }
        Ok(_) => {}
        Err(error) => emit_error(app, error.to_string()),
    }
}

fn capture_loop(
    app: AppHandle,
    device_id: &str,
    settings: AppSettings,
    stop_requested: Arc<AtomicBool>,
    ready: &mpsc::SyncSender<Result<(), String>>,
) -> Result<(), AppError> {
    let device = selected_device(device_id)?;
    let supported = device
        .default_input_config()
        .map_err(|_| AppError::new("選択したマイクの録音形式を取得できませんでした。"))?;
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    let sample_rate = config.sample_rate.0;
    let channels = config.channels as usize;
    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let callback_buffer = Arc::clone(&buffer);
    let error_app = app.clone();
    let error_callback = move |error: cpal::StreamError| {
        emit_error(&error_app, format!("マイク入力エラー: {error}"));
    };

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| push_f32(&callback_buffer, data, channels),
            error_callback,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| push_i16(&callback_buffer, data, channels),
            error_callback,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _| push_u16(&callback_buffer, data, channels),
            error_callback,
            None,
        ),
        _ => return Err(AppError::new("このマイクの録音形式には対応していません。")),
    }
    .map_err(|_| AppError::new("選択したマイクを開始できませんでした。ほかのアプリが使用していないか確認してください。"))?;

    stream
        .play()
        .map_err(|_| AppError::new("マイク録音を開始できませんでした。"))?;
    let _ = ready.send(Ok(()));

    let partial_samples = sample_rate as usize * PARTIAL_SECONDS;
    let final_samples = sample_rate as usize * FINAL_SECONDS;
    let mut last_partial = Instant::now();

    while !stop_requested.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(200));
        let count = buffer.lock().map(|samples| samples.len()).unwrap_or(0);
        if count >= final_samples {
            let snapshot = buffer
                .lock()
                .map(|samples| samples.clone())
                .unwrap_or_default();
            transcribe_snapshot(&app, &settings, &snapshot, sample_rate, true);
            if let Ok(mut samples) = buffer.lock() {
                let consumed = snapshot.len().min(samples.len());
                samples.drain(..consumed);
            }
            last_partial = Instant::now();
        } else if count >= partial_samples
            && last_partial.elapsed() >= Duration::from_secs(PARTIAL_SECONDS as u64)
        {
            let snapshot = buffer
                .lock()
                .map(|samples| samples.clone())
                .unwrap_or_default();
            transcribe_snapshot(&app, &settings, &snapshot, sample_rate, false);
            last_partial = Instant::now();
        }
    }

    drop(stream);
    let remaining = buffer
        .lock()
        .map(|samples| samples.clone())
        .unwrap_or_default();
    if remaining.len() >= sample_rate as usize / 2 {
        transcribe_snapshot(&app, &settings, &remaining, sample_rate, true);
    }
    Ok(())
}
