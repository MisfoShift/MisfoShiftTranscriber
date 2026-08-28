import "./styles.css";
import {
  backend,
  chooseAudioFile,
  chooseFfmpeg,
  chooseModel,
  chooseTextDestination,
  chooseWhisperExecutable,
  listenTo,
} from "./services/backend";
import { appendTranscript, transcriptFileName } from "./lib/transcript";
import type {
  AppSettings,
  ComponentOverview,
  ComponentProgress,
  EngineStatus,
  ManagedComponentStatus,
  MicrophoneDevice,
} from "./types";

const root = document.querySelector<HTMLDivElement>("#app")!;
const brandIconUrl = new URL("../transcriber.png", import.meta.url).href;

root.innerHTML = `
  <div class="app-shell">
    <header class="app-header">
      <img class="brand-mark" src="${brandIconUrl}" alt="" aria-hidden="true" />
      <div class="brand-copy">
        <h1>MisfoShiftTranscriber</h1>
        <p>音声をPCの中だけで文字にします</p>
      </div>
      <span class="privacy-badge"><span aria-hidden="true">●</span> 外部送信なし</span>
    </header>

    <div id="error-banner" class="error-banner" role="alert" hidden>
      <span id="error-text"></span>
      <button id="dismiss-error" type="button" aria-label="エラーを閉じる">×</button>
    </div>
    <div id="notice" class="notice" role="status" hidden></div>

    <section id="engine-summary" class="card engine-summary" aria-label="文字起こし設定の概要">
      <div class="engine-summary-heading">
        <div><h2>文字起こし設定</h2><p id="engine-summary-message">ローカル環境を確認しています…</p></div>
        <span id="engine-badge" class="status-badge checking">確認中</span>
        <button id="open-settings" class="secondary" type="button">設定変更</button>
      </div>
      <div class="engine-summary-grid">
        <div><span>エンジン</span><strong id="summary-engine">確認中</strong></div>
        <div><span>モデル</span><strong id="summary-model">確認中</strong></div>
        <div><span>認識言語</span><strong id="summary-language">日本語</strong></div>
        <div><span>FFmpeg</span><strong id="summary-ffmpeg">確認中</strong></div>
      </div>
      <div id="initial-setup" class="initial-setup" hidden>
        <span>初期設定が必要です</span>
        <button id="setup-recommended-main" class="link-button" type="button">推奨構成をセットアップ</button>
      </div>
    </section>

    <section class="input-grid" aria-label="文字起こし入力">
      <article class="card input-card">
        <div class="card-heading">
          <div class="icon microphone-icon" aria-hidden="true">●</div>
          <div><h2>マイク</h2><p>話した内容をリアルタイム認識</p></div>
        </div>
        <label class="field-label" for="microphone-select">入力デバイス</label>
        <div class="inline-control">
          <select id="microphone-select"><option value="">マイクを確認中…</option></select>
          <button id="refresh-microphones" class="icon-button" type="button" title="マイク一覧を更新" aria-label="マイク一覧を更新">↻</button>
        </div>
        <div class="button-row">
          <button id="start-microphone" class="primary" type="button">文字起こし開始</button>
          <button id="stop-microphone" class="danger" type="button" disabled>停止</button>
        </div>
        <div id="recording-state" class="recording-state"><span></span><strong>待機中</strong></div>
      </article>

      <article class="card input-card">
        <div class="card-heading">
          <div class="icon file-icon" aria-hidden="true">♪</div>
          <div><h2>音声ファイル</h2><p>保存済みの音声・動画から認識</p></div>
        </div>
        <label class="field-label" for="audio-path">選択ファイル</label>
        <div class="file-picker">
          <input id="audio-path" type="text" readonly placeholder="ファイルが選択されていません" />
          <button id="choose-audio" class="secondary" type="button">ファイルを選択</button>
        </div>
        <button id="transcribe-file" class="primary wide" type="button" disabled>このファイルを文字起こし</button>
        <div id="file-ffmpeg-required" class="file-requirement" hidden>
          <span>音声ファイルの文字起こしにはFFmpegが必要です。</span>
          <button id="open-ffmpeg-settings" class="link-button" type="button">設定を開く</button>
        </div>
        <div id="file-state" class="process-state">待機中</div>
      </article>
    </section>

    <section class="card transcript-card">
      <div class="transcript-heading">
        <div><h2>文字起こし結果</h2><p>結果は編集してからコピー・保存できます</p></div>
        <span id="character-count">0文字</span>
      </div>
      <div id="partial-box" class="partial-box" hidden>
        <span>認識途中</span>
        <p id="partial-text"></p>
      </div>
      <textarea id="transcript" spellcheck="true" placeholder="ここに確定した文字起こし結果が蓄積されます"></textarea>
      <div class="result-actions">
        <button id="copy-result" class="secondary" type="button" disabled>結果をコピー</button>
        <button id="save-result" class="secondary" type="button" disabled>TXTで保存</button>
        <button id="clear-result" class="ghost" type="button" disabled>クリア</button>
      </div>
    </section>

    <footer>ローカルWhisper（whisper.cpp）で処理 · 音声データはこのPCから送信されません</footer>

    <dialog id="settings-dialog" class="settings-dialog" aria-labelledby="settings-title">
      <div class="settings-dialog-header">
        <div><h2 id="settings-title">文字起こしエンジン設定</h2><p>このPC内で使用する実行ファイルとモデルを設定します。</p></div>
        <button id="close-settings" class="dialog-close" type="button" aria-label="設定を閉じる">×</button>
      </div>
      <div class="settings-dialog-content">
        <p id="engine-message" class="engine-message">ローカル環境を確認しています…</p>
        <div id="component-progress" class="component-progress" hidden>
          <div><strong id="component-progress-message">ダウンロードを準備しています…</strong><span id="component-progress-percent"></span></div>
          <progress id="component-progress-bar" max="100" value="0"></progress>
        </div>

        <div id="recommended-setup" class="recommended-setup">
          <div><strong>推奨構成を自動セットアップ</strong><span>whisper.cpp・標準モデル・FFmpegを順番に準備します。</span></div>
          <button id="setup-recommended" class="primary" type="button">セットアップ</button>
        </div>

        <section class="component-section" aria-labelledby="whisper-component-title">
          <div class="component-heading">
            <div><h3 id="whisper-component-title">whisper.cpp</h3><p>ローカル文字起こしエンジン</p></div>
            <span id="whisper-component-state" class="component-state missing">未インストール</span>
          </div>
          <div class="component-meta"><span>推奨バージョン</span><strong id="whisper-component-version">確認中</strong></div>
          <div class="component-actions">
            <button id="install-whisper" class="secondary" type="button">ダウンロード</button>
            <button id="choose-whisper" class="ghost" type="button">手動選択</button>
          </div>
          <details class="technical-details">
            <summary>詳細情報</summary>
            <input id="whisper-path" type="text" readonly placeholder="whisper-cli.exeが未設定です" />
            <p id="whisper-component-details"></p>
          </details>
        </section>

        <section class="component-section" aria-labelledby="models-title">
          <div class="component-heading">
            <div><h3 id="models-title">Whisperモデル</h3><p>smallを標準モデルとして推奨します</p></div>
          </div>
          <div id="managed-models" class="managed-models"></div>
          <details class="technical-details manual-model-details">
            <summary>手元のモデルを使用</summary>
            <div class="manual-setting-row">
              <input id="model-path" type="text" readonly placeholder="ggmlモデルが未設定です" />
              <button id="choose-model" class="secondary" type="button">手動選択</button>
            </div>
          </details>
        </section>

        <section class="component-section" aria-labelledby="ffmpeg-component-title">
          <div class="component-heading">
            <div><h3 id="ffmpeg-component-title">FFmpeg</h3><p>音声ファイルの変換に使用</p></div>
            <span id="ffmpeg-component-state" class="component-state missing">未インストール</span>
          </div>
          <div class="component-meta"><span>推奨バージョン</span><strong id="ffmpeg-component-version">確認中</strong></div>
          <div class="component-actions">
            <button id="install-ffmpeg" class="secondary" type="button">ダウンロード</button>
            <button id="choose-ffmpeg" class="ghost" type="button">手動選択</button>
          </div>
          <details class="technical-details">
            <summary>詳細情報</summary>
            <input id="ffmpeg-path" type="text" readonly placeholder="ffmpeg.exeが未設定です" />
            <p id="ffmpeg-component-details"></p>
          </details>
        </section>

        <div class="language-row">
          <label for="language">認識言語</label>
          <select id="language">
            <option value="ja">日本語</option>
            <option value="auto">自動判定</option>
            <option value="en">英語</option>
          </select>
          <span id="resolved-paths" class="resolved-paths"></span>
        </div>
        <details class="storage-details">
          <summary>保存場所</summary>
          <p id="component-storage-path">確認中</p>
        </details>
      </div>
      <div class="settings-dialog-footer"><button id="close-settings-footer" class="primary" type="button">閉じる</button></div>
    </dialog>
  </div>
`;

const element = <T extends HTMLElement>(id: string) => document.querySelector<T>(`#${id}`)!;
const errorBanner = element<HTMLDivElement>("error-banner");
const errorText = element<HTMLSpanElement>("error-text");
const notice = element<HTMLDivElement>("notice");
const settingsDialog = element<HTMLDialogElement>("settings-dialog");
const engineBadge = element<HTMLSpanElement>("engine-badge");
const engineMessage = element<HTMLParagraphElement>("engine-message");
const resolvedPaths = element<HTMLSpanElement>("resolved-paths");
const engineSummaryMessage = element<HTMLParagraphElement>("engine-summary-message");
const summaryEngine = element<HTMLElement>("summary-engine");
const summaryModel = element<HTMLElement>("summary-model");
const summaryLanguage = element<HTMLElement>("summary-language");
const summaryFfmpeg = element<HTMLElement>("summary-ffmpeg");
const initialSetup = element<HTMLDivElement>("initial-setup");
const managedModels = element<HTMLDivElement>("managed-models");
const componentProgressPanel = element<HTMLDivElement>("component-progress");
const componentProgressMessage = element<HTMLElement>("component-progress-message");
const componentProgressPercent = element<HTMLElement>("component-progress-percent");
const componentProgressBar = element<HTMLProgressElement>("component-progress-bar");
const componentStoragePath = element<HTMLElement>("component-storage-path");
const recommendedSetup = element<HTMLDivElement>("recommended-setup");
const whisperPath = element<HTMLInputElement>("whisper-path");
const modelPath = element<HTMLInputElement>("model-path");
const ffmpegPath = element<HTMLInputElement>("ffmpeg-path");
const language = element<HTMLSelectElement>("language");
const microphoneSelect = element<HTMLSelectElement>("microphone-select");
const startMicrophone = element<HTMLButtonElement>("start-microphone");
const stopMicrophone = element<HTMLButtonElement>("stop-microphone");
const recordingState = element<HTMLDivElement>("recording-state");
const audioPath = element<HTMLInputElement>("audio-path");
const transcribeFile = element<HTMLButtonElement>("transcribe-file");
const fileFfmpegRequired = element<HTMLDivElement>("file-ffmpeg-required");
const fileState = element<HTMLDivElement>("file-state");
const partialBox = element<HTMLDivElement>("partial-box");
const partialText = element<HTMLParagraphElement>("partial-text");
const transcript = element<HTMLTextAreaElement>("transcript");
const characterCount = element<HTMLSpanElement>("character-count");
const copyResult = element<HTMLButtonElement>("copy-result");
const saveResult = element<HTMLButtonElement>("save-result");
const clearResult = element<HTMLButtonElement>("clear-result");

let settings: AppSettings = { whisper_path: "", model_path: "", ffmpeg_path: "", language: "ja" };
let engine: EngineStatus | null = null;
let componentOverview: ComponentOverview | null = null;
let componentProgress: ComponentProgress | null = null;
let componentBusy = false;
let microphones: MicrophoneDevice[] = [];
let chosenAudio = "";
let finalText = "";
let interimText = "";
let recording = false;
let stopping = false;
let fileBusy = false;
let fileProgress = "";
let noticeTimer: number | undefined;

function messageOf(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "予期しないエラーが発生しました。";
}

function showError(error: unknown): void {
  errorText.textContent = messageOf(error);
  errorBanner.hidden = false;
}

function showNotice(message: string): void {
  window.clearTimeout(noticeTimer);
  notice.textContent = message;
  notice.hidden = false;
  noticeTimer = window.setTimeout(() => { notice.hidden = true; }, 3200);
}

function modelDisplayName(path: string | null | undefined): string {
  const name = path?.toLowerCase() ?? "";
  if (name.includes("large")) return "高性能PC向け（large）";
  if (name.includes("medium")) return "高精度（medium）";
  if (name.includes("small")) return "標準（small）";
  if (name.includes("base")) return "軽量（base）";
  return path ? "カスタムモデル" : "未設定";
}

function languageDisplayName(value: string): string {
  if (value === "auto") return "自動判定";
  if (value === "en") return "英語";
  return "日本語";
}

function componentById(id: string): ManagedComponentStatus | undefined {
  return componentOverview?.components.find((component) => component.id === id);
}

function updateComponentState(prefix: "whisper" | "ffmpeg", component?: ManagedComponentStatus): void {
  const state = element<HTMLElement>(`${prefix}-component-state`);
  const version = element<HTMLElement>(`${prefix}-component-version`);
  const details = element<HTMLElement>(`${prefix}-component-details`);
  const installButton = element<HTMLButtonElement>(`install-${prefix}`);
  state.textContent = component?.installed ? "インストール済み" : "未インストール";
  state.className = `component-state ${component?.installed ? "installed" : "missing"}`;
  version.textContent = component?.version ?? "確認中";
  installButton.textContent = component?.managed ? "再取得" : "ダウンロード";
  installButton.disabled = componentBusy || !component;
  details.textContent = component
    ? `${component.file_name} · ${component.installation_source} · ${component.license} · ${component.download_size_label} · 配布元: ${component.source_url}`
    : "コンポーネント情報を確認しています…";
}

function modelActionButton(label: string, className: string, action: () => void): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = className;
  button.textContent = label;
  button.disabled = componentBusy;
  button.addEventListener("click", action);
  return button;
}

function renderManagedModels(): void {
  managedModels.replaceChildren();
  const models = componentOverview?.components.filter((component) => component.kind === "model") ?? [];
  if (!models.length) {
    const loading = document.createElement("p");
    loading.className = "component-loading";
    loading.textContent = "モデル情報を確認しています…";
    managedModels.append(loading);
    return;
  }
  for (const model of models) {
    const row = document.createElement("div");
    row.className = `model-option${model.current ? " current" : ""}`;

    const indicator = document.createElement("span");
    indicator.className = "model-indicator";
    indicator.textContent = model.current ? "●" : "○";

    const description = document.createElement("div");
    description.className = "model-description";
    const name = document.createElement("strong");
    name.textContent = model.display_name;
    const meta = document.createElement("span");
    const state = model.installed ? "インストール済み" : "未インストール";
    meta.textContent = `${state}${model.current ? " / 使用中" : ""} · ${model.download_size_label}`;
    const technical = document.createElement("details");
    technical.className = "model-technical";
    const technicalSummary = document.createElement("summary");
    technicalSummary.textContent = "詳細";
    const technicalText = document.createElement("small");
    technicalText.textContent = `${model.file_name} · ${model.license} · 配布元: ${model.source_url}`;
    technical.append(technicalSummary, technicalText);
    description.append(name, meta, technical);

    const actions = document.createElement("div");
    actions.className = "model-actions";
    if (!model.managed) {
      actions.append(modelActionButton("ダウンロード", "secondary", () => void installComponent(model.id)));
    } else {
      if (!model.current) {
        actions.append(modelActionButton("使用する", "secondary", () => void selectModel(model.id)));
      }
      actions.append(modelActionButton("再取得", "ghost", () => void installComponent(model.id)));
      actions.append(modelActionButton("削除", "ghost danger-text", () => void deleteModel(model)));
    }
    row.append(indicator, description, actions);
    managedModels.append(row);
  }
}

function renderComponentManager(): void {
  updateComponentState("whisper", componentById("whisper"));
  updateComponentState("ffmpeg", componentById("ffmpeg"));
  renderManagedModels();
  componentStoragePath.textContent = componentOverview?.storage_directory ?? "確認中";
  recommendedSetup.hidden = engine?.file_ready ?? false;
  element<HTMLButtonElement>("setup-recommended").disabled = componentBusy;
  element<HTMLButtonElement>("setup-recommended-main").disabled = componentBusy;
  element<HTMLButtonElement>("choose-whisper").disabled = componentBusy;
  element<HTMLButtonElement>("choose-model").disabled = componentBusy;
  element<HTMLButtonElement>("choose-ffmpeg").disabled = componentBusy;

  componentProgressPanel.hidden = !componentBusy && !componentProgress;
  if (componentProgress) {
    componentProgressMessage.textContent = componentProgress.message;
    const hasTotal = componentProgress.total_bytes > 0;
    componentProgressPercent.textContent = hasTotal ? `${Math.round(componentProgress.percent)}%` : "";
    if (hasTotal) {
      componentProgressBar.value = componentProgress.percent;
    } else {
      componentProgressBar.removeAttribute("value");
    }
  } else if (componentBusy) {
    componentProgressMessage.textContent = "セットアップを準備しています…";
    componentProgressPercent.textContent = "";
    componentProgressBar.removeAttribute("value");
  }
}

function renderSettings(): void {
  whisperPath.value = settings.whisper_path;
  modelPath.value = settings.model_path;
  ffmpegPath.value = settings.ffmpeg_path;
  language.value = settings.language;
  const configurationReady = engine?.ready ?? false;
  engineBadge.textContent = engine ? (configurationReady ? "利用可能" : "要設定") : "確認中";
  engineBadge.className = `status-badge ${engine ? (configurationReady ? "ready" : "missing") : "checking"}`;
  engineMessage.textContent = engine?.message ?? "ローカル環境を確認しています…";
  const detected = [
    engine?.whisper_path ? "Whisper検出済み" : "",
    engine?.model_path ? "モデル検出済み" : "",
    engine?.ffmpeg_path ? "FFmpeg検出済み" : "",
  ].filter(Boolean);
  resolvedPaths.textContent = detected.join(" · ");

  const needsSetup = !configurationReady;
  engineSummaryMessage.textContent = needsSetup
    ? "初期設定を完了すると、ローカル文字起こしを利用できます。"
    : engine?.ffmpeg_path
      ? "このPC内でローカル文字起こしを実行します。"
      : "マイク文字起こしを利用できます。音声ファイルにはFFmpegが必要です。";
  summaryEngine.textContent = engine?.whisper_path ? "whisper.cpp" : "未設定";
  summaryModel.textContent = modelDisplayName(engine?.model_path ?? settings.model_path);
  summaryLanguage.textContent = languageDisplayName(settings.language);
  summaryFfmpeg.textContent = engine?.ffmpeg_path ? "設定済み" : "未設定";
  initialSetup.hidden = !needsSetup;
  renderComponentManager();
}

function renderMicrophones(): void {
  const previous = microphoneSelect.value;
  microphoneSelect.replaceChildren();
  for (const microphone of microphones) {
    const option = document.createElement("option");
    option.value = microphone.id;
    option.textContent = microphone.is_default ? `${microphone.name}（既定）` : microphone.name;
    microphoneSelect.append(option);
  }
  const selected = microphones.find((item) => item.id === previous)
    ?? microphones.find((item) => item.is_default)
    ?? microphones[0];
  if (selected) microphoneSelect.value = selected.id;
  if (!selected) {
    const option = document.createElement("option");
    option.textContent = "利用できるマイクがありません";
    option.value = "";
    microphoneSelect.append(option);
  }
}

function render(): void {
  renderSettings();
  const hasEngine = engine?.ready ?? false;
  const hasText = finalText.trim().length > 0;
  const hasMicrophone = Boolean(microphoneSelect.value);
  startMicrophone.disabled = recording || fileBusy || !hasEngine || !hasMicrophone;
  stopMicrophone.disabled = !recording || stopping;
  stopMicrophone.textContent = stopping ? "停止処理中…" : "停止";
  microphoneSelect.disabled = recording || fileBusy;
  element<HTMLButtonElement>("refresh-microphones").disabled = recording || fileBusy;
  recordingState.classList.toggle("active", recording);
  recordingState.querySelector("strong")!.textContent = recording
    ? (stopping ? "残りの音声を確定中" : "録音・認識中")
    : "待機中";

  audioPath.value = chosenAudio;
  element<HTMLButtonElement>("choose-audio").disabled = recording || fileBusy;
  transcribeFile.disabled = !chosenAudio || recording || fileBusy || !(engine?.file_ready ?? false);
  fileFfmpegRequired.hidden = !chosenAudio || !(engine?.ready ?? false) || Boolean(engine?.ffmpeg_path);
  transcribeFile.textContent = fileBusy ? "文字起こし中…" : "このファイルを文字起こし";
  fileState.textContent = fileProgress || "待機中";
  fileState.classList.toggle("active", fileBusy);

  transcript.value = finalText;
  characterCount.textContent = `${finalText.length.toLocaleString("ja-JP")}文字`;
  partialBox.hidden = !interimText.trim();
  partialText.textContent = interimText;
  copyResult.disabled = !hasText;
  saveResult.disabled = !hasText;
  clearResult.disabled = !hasText && !interimText;
}

async function persistSettings(): Promise<void> {
  engine = await backend.saveSettings(settings);
  render();
}

async function refreshConfiguration(): Promise<void> {
  const [loadedSettings, loadedEngine, loadedComponents] = await Promise.all([
    backend.loadSettings(),
    backend.engineStatus(),
    backend.componentOverview(),
  ]);
  settings = loadedSettings;
  engine = loadedEngine;
  componentOverview = loadedComponents;
}

async function runComponentOperation(
  operation: () => Promise<ComponentOverview>,
  successMessage: string,
): Promise<void> {
  if (componentBusy) return;
  componentBusy = true;
  componentProgress = null;
  errorBanner.hidden = true;
  render();
  try {
    componentOverview = await operation();
    const [loadedSettings, loadedEngine] = await Promise.all([
      backend.loadSettings(),
      backend.engineStatus(),
    ]);
    settings = loadedSettings;
    engine = loadedEngine;
    showNotice(successMessage);
  } catch (error) {
    showError(error);
    try {
      await refreshConfiguration();
    } catch {
      // The original actionable error is more useful than a secondary refresh failure.
    }
  } finally {
    componentBusy = false;
    componentProgress = null;
    render();
  }
}

async function installComponent(componentId: string): Promise<void> {
  await runComponentOperation(
    () => backend.installComponent(componentId),
    "コンポーネントのインストールが完了しました。",
  );
}

async function setupRecommended(): Promise<void> {
  await runComponentOperation(
    () => backend.setupRecommendedComponents(),
    "推奨構成のセットアップが完了しました。オフラインで文字起こしを利用できます。",
  );
}

async function selectModel(componentId: string): Promise<void> {
  await runComponentOperation(
    () => backend.selectManagedModel(componentId),
    "使用するモデルを変更しました。",
  );
}

async function deleteModel(model: ManagedComponentStatus): Promise<void> {
  if (!window.confirm(`${model.display_name}をこのPCから削除しますか？`)) return;
  await runComponentOperation(
    () => backend.deleteManagedModel(model.id),
    "モデルを削除しました。",
  );
}

async function chooseSetting(
  key: "whisper_path" | "model_path" | "ffmpeg_path",
  chooser: () => Promise<string | null>,
): Promise<void> {
  try {
    const path = await chooser();
    if (!path) return;
    settings = { ...settings, [key]: path };
    await persistSettings();
    componentOverview = await backend.componentOverview();
    render();
    showNotice("設定を保存しました。");
  } catch (error) {
    showError(error);
  }
}

async function refreshMicrophones(): Promise<void> {
  try {
    microphones = await backend.listMicrophones();
    renderMicrophones();
    render();
  } catch (error) {
    microphones = [];
    renderMicrophones();
    render();
    showError(error);
  }
}

element("dismiss-error").addEventListener("click", () => { errorBanner.hidden = true; });
const openSettings = () => settingsDialog.showModal();
element("open-settings").addEventListener("click", openSettings);
element("open-ffmpeg-settings").addEventListener("click", openSettings);
element("setup-recommended-main").addEventListener("click", () => {
  openSettings();
  void setupRecommended();
});
element("close-settings").addEventListener("click", () => settingsDialog.close());
element("close-settings-footer").addEventListener("click", () => settingsDialog.close());
settingsDialog.addEventListener("click", (event) => {
  if (event.target === settingsDialog) settingsDialog.close();
});
element("choose-whisper").addEventListener("click", () => void chooseSetting("whisper_path", chooseWhisperExecutable));
element("choose-model").addEventListener("click", () => void chooseSetting("model_path", chooseModel));
element("choose-ffmpeg").addEventListener("click", () => void chooseSetting("ffmpeg_path", chooseFfmpeg));
element("install-whisper").addEventListener("click", () => void installComponent("whisper"));
element("install-ffmpeg").addEventListener("click", () => void installComponent("ffmpeg"));
element("setup-recommended").addEventListener("click", () => void setupRecommended());
element("refresh-microphones").addEventListener("click", () => void refreshMicrophones());

language.addEventListener("change", () => {
  settings = { ...settings, language: language.value };
  void persistSettings().catch(showError);
});

startMicrophone.addEventListener("click", async () => {
  try {
    errorBanner.hidden = true;
    await backend.startMicrophone(microphoneSelect.value);
  } catch (error) {
    recording = false;
    stopping = false;
    showError(error);
    render();
  }
});

stopMicrophone.addEventListener("click", async () => {
  try {
    stopping = true;
    render();
    await backend.stopMicrophone();
  } catch (error) {
    stopping = false;
    showError(error);
    render();
  }
});

element("choose-audio").addEventListener("click", async () => {
  try {
    const path = await chooseAudioFile();
    if (path) chosenAudio = path;
    render();
  } catch (error) {
    showError(error);
  }
});

transcribeFile.addEventListener("click", async () => {
  try {
    fileBusy = true;
    fileProgress = "ローカルで音声を処理しています…";
    errorBanner.hidden = true;
    render();
    const result = await backend.transcribeFile(chosenAudio);
    if (result.trim()) {
      finalText = appendTranscript(finalText, result);
      showNotice("音声ファイルの文字起こしが完了しました。");
    } else {
      showNotice("認識できる音声は見つかりませんでした。");
    }
  } catch (error) {
    showError(error);
  } finally {
    fileBusy = false;
    fileProgress = "";
    render();
  }
});

transcript.addEventListener("input", () => {
  finalText = transcript.value;
  render();
});

copyResult.addEventListener("click", async () => {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(finalText);
    } else {
      transcript.focus();
      transcript.select();
      if (!document.execCommand("copy")) throw new Error("コピーできませんでした。");
    }
    showNotice("文字起こし結果をコピーしました。");
  } catch (error) {
    showError(error);
  }
});

saveResult.addEventListener("click", async () => {
  try {
    const path = await chooseTextDestination(transcriptFileName());
    if (!path) return;
    await backend.saveText(path, finalText);
    showNotice("TXTファイルを保存しました。");
  } catch (error) {
    showError(error);
  }
});

clearResult.addEventListener("click", () => {
  finalText = "";
  interimText = "";
  render();
  showNotice("文字起こし結果をクリアしました。");
});

async function initialize(): Promise<void> {
  const unlisteners = await Promise.all([
    listenTo<string>("transcript-partial", (text) => {
      interimText = text;
      render();
    }),
    listenTo<string>("transcript-final", (text) => {
      finalText = appendTranscript(finalText, text);
      interimText = "";
      render();
    }),
    listenTo<boolean>("mic-state", (value) => {
      recording = value;
      stopping = false;
      if (!value) interimText = "";
      render();
    }),
    listenTo<string>("file-progress", (text) => {
      fileProgress = text;
      render();
    }),
    listenTo<string>("app-error", showError),
    listenTo<ComponentProgress>("component-progress", (progress) => {
      componentProgress = progress;
      render();
    }),
  ]);
  window.addEventListener("beforeunload", () => unlisteners.forEach((unlisten) => unlisten()));

  try {
    await refreshConfiguration();
    render();
  } catch (error) {
    showError(error);
  }
  await refreshMicrophones();
}

renderMicrophones();
render();
void initialize();
