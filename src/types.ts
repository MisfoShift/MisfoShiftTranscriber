export interface AppSettings {
  whisper_path: string;
  model_path: string;
  ffmpeg_path: string;
  language: string;
}

export interface EngineStatus {
  ready: boolean;
  file_ready: boolean;
  whisper_path: string | null;
  model_path: string | null;
  ffmpeg_path: string | null;
  message: string;
}

export interface MicrophoneDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export interface ManagedComponentStatus {
  id: string;
  kind: "whisper" | "model" | "ffmpeg";
  display_name: string;
  version: string;
  recommended: boolean;
  installed: boolean;
  managed: boolean;
  current: boolean;
  download_size_bytes: number;
  download_size_label: string;
  file_name: string;
  source_url: string;
  license: string;
  installation_source: string;
}

export interface ComponentOverview {
  components: ManagedComponentStatus[];
  storage_directory: string;
  busy_component: string | null;
}

export interface ComponentProgress {
  component_id: string;
  stage: "connecting" | "downloading" | "verifying" | "installing" | "complete";
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
  message: string;
}
