import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { AppSettings, ComponentOverview, EngineStatus, MicrophoneDevice } from "../types";

export const backend = {
  loadSettings: () => invoke<AppSettings>("load_settings"),
  saveSettings: (settings: AppSettings) =>
    invoke<EngineStatus>("save_settings", { settings }),
  engineStatus: () => invoke<EngineStatus>("engine_status"),
  componentOverview: () => invoke<ComponentOverview>("component_overview"),
  installComponent: (componentId: string) =>
    invoke<ComponentOverview>("install_managed_component", { componentId }),
  setupRecommendedComponents: () =>
    invoke<ComponentOverview>("setup_recommended_components"),
  selectManagedModel: (componentId: string) =>
    invoke<ComponentOverview>("select_managed_model", { componentId }),
  deleteManagedModel: (componentId: string) =>
    invoke<ComponentOverview>("delete_managed_model", { componentId }),
  listMicrophones: () => invoke<MicrophoneDevice[]>("list_microphones"),
  startMicrophone: (deviceId: string) =>
    invoke<void>("start_microphone", { deviceId }),
  stopMicrophone: () => invoke<void>("stop_microphone"),
  transcribeFile: (path: string) =>
    invoke<string>("transcribe_audio_file", { path }),
  saveText: (path: string, text: string) =>
    invoke<void>("save_text_file", { path, text }),
};

export async function chooseWhisperExecutable(): Promise<string | null> {
  return open({
    multiple: false,
    title: "whisper.cppの実行ファイルを選択",
    filters: [{ name: "実行ファイル", extensions: ["exe"] }],
  });
}

export async function chooseModel(): Promise<string | null> {
  return open({
    multiple: false,
    title: "Whisper ggmlモデルを選択",
    filters: [{ name: "Whisperモデル", extensions: ["bin"] }],
  });
}

export async function chooseFfmpeg(): Promise<string | null> {
  return open({
    multiple: false,
    title: "ffmpeg.exeを選択",
    filters: [{ name: "実行ファイル", extensions: ["exe"] }],
  });
}

export async function chooseAudioFile(): Promise<string | null> {
  return open({
    multiple: false,
    title: "文字起こしする音声ファイルを選択",
    filters: [
      {
        name: "音声ファイル",
        extensions: ["wav", "mp3", "m4a", "aac", "flac", "ogg", "wma", "mp4", "webm"],
      },
    ],
  });
}

export async function chooseTextDestination(defaultPath: string): Promise<string | null> {
  return save({
    title: "文字起こし結果を保存",
    defaultPath,
    filters: [{ name: "テキストファイル", extensions: ["txt"] }],
  });
}

export async function listenTo<T>(event: string, callback: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, ({ payload }) => callback(payload));
}
