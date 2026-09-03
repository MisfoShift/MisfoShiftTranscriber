# MisfoShiftTranscriber

MisfoShiftTranscriberは、Windows上で音声をローカル文字起こしするデスクトップアプリです。音声ファイルとマイク入力に対応し、文字起こし対象の音声を外部APIへ送信しません。

アプリ本体には、原則としてwhisper.cpp、Whisperモデル、FFmpegなどの音声認識用コンポーネントを同梱しません。必要なものは初回セットアップまたは設定画面から取得します。

## 主な機能

- Windows GUIからマイクデバイスの一覧取得・選択
- マイク文字起こしの開始・停止、途中結果と確定結果の表示
- WAV / MP3 / M4A / AAC / FLAC / OGG / WMA / MP4 / WebMのファイル文字起こし
- 結果の編集、コピー、UTF-8 TXT保存、クリア
- whisper.cpp、Whisperモデル、FFmpegのアプリ内セットアップ
- ダウンロード進捗、SHA-256検証、一時保存後の正式配置
- small / medium / large-v3モデルのインストール、切替、削除
- 既存ファイルを選ぶ手動設定

## 初回セットアップ

初回起動時に「初期設定が必要です」と表示されたら、「推奨構成をセットアップ」を押します。次の構成が不足している場合だけ、アプリが順番に取得します。

- whisper.cpp 1.9.0
- 標準モデル（small）
- FFmpeg n8.1.2ベースの固定ビルド

各ファイルはHTTPSで取得し、[components.json](src-tauri/components.json)に記録したSHA-256と一致した場合だけ正式配置します。接続切断、破損、展開失敗時の一時ファイルは処理終了時に削除されます。

設定ダイアログでは、各コンポーネントの個別取得・再取得、モデルの切替・削除、既存ファイルの手動指定もできます。

推奨モデルは標準（small）です。一度必要なコンポーネントを取得すれば、文字起こし処理はローカルで完結し、通常利用時はオフラインで動作します。

## 保存場所

取得したコンポーネントをGitリポジトリやインストール先へ書き込むことはありません。Tauriが提供するWindowsのアプリデータ領域を使います。

```text
%APPDATA%\com.misfoshift.transcriber\
  settings.json
  components\
    whisper\<version>\
    models\<model>\
    ffmpeg\<version>\
```

実際の保存場所は設定ダイアログの「保存場所」で確認できます。ダウンロード中の一時ディレクトリも`components`内に作成され、成功後は原子的に正式ディレクトリへ置き換えられます。

初回Publicリリース前にアプリ識別子とWindows PublisherをMisfoShift名義へ変更したため、以前の開発版が作成した設定やコンポーネント保存領域は自動移行しません。開発版を利用していた環境では、旧開発版を先にアンインストールし、必要に応じてコンポーネントを再設定または再取得してください。

## プライバシーとオフライン利用

文字起こしはローカルのwhisper.cppで実行します。本アプリに音声アップロード処理や文字起こしAPI呼び出しはありません。

外部通信は、ユーザーが設定画面でダウンロードまたは再取得を実行したときだけ発生します。通常起動時の更新確認やバックグラウンド通信は行いません。一度セットアップすれば、文字起こしはオフラインで利用できます。

## 管理対象コンポーネント

取得URL、バージョン、SHA-256、アーカイブ形式、必要容量、配置先はすべて[components.json](src-tauri/components.json)で一元管理しています。更新時はURLだけでなく、公式配布物を取得してハッシュ、ライセンス、展開後の実行ファイル名を再確認してください。

| コンポーネント | 取得元 | ライセンス | 通常の扱い |
| --- | --- | --- | --- |
| whisper.cpp 1.9.0 | ggml-org公式GitHub Releases | MIT | 不足時に自動取得 |
| ggml-small / medium / large-v3 | ggerganov/whisper.cpp on Hugging Face | MIT | ユーザー選択で自動取得 |
| FFmpeg / BtbN Windows x64 LGPL build | BtbN公式GitHub Releases | LGPL-3.0-or-later | 既存版を自動検出し、不足時にユーザー操作で取得 |

詳細な固定バージョン、ハッシュ、Public配布時の注意点は[docs/COMPONENTS.md](docs/COMPONENTS.md)と[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)を参照してください。

## 開発環境

- Windows 10 / 11 x64
- Node.js 20以降
- Rust stable（MSVC toolchain）
- Visual Studio 2022 Build Tools
  - `Desktop development with C++`ワークロード
- Microsoft Edge WebView2 Runtime

依存関係をインストールします。

```powershell
npm install
```

開発版を起動します。

```powershell
npm run tauri:dev:windows
```

## テストとビルド

```powershell
npm test
npm run build
cd src-tauri
cargo test
cd ..
npm run tauri:build:windows
```

標準のWindows配布物には、whisper.cpp、Whisperモデル、FFmpegを含めません。配布物にはアプリ本体、UI、コンポーネント取得定義、第三者ライセンス情報、アプリアイコンを含めます。`transcriber.png`をアイコンの元画像として保持し、`src-tauri/icons/`の生成済みアイコンをWindows実行ファイルとインストーラーのビルドに使用します。

MSI / NSISにはアプリ本体の`LICENSE`と`THIRD_PARTY_NOTICES.md`を含めます。リリース前の監査、SHA-256生成、配布確認は[リリース手順](docs/RELEASING.md)と[手動確認チェックリスト](docs/RELEASE_CHECKLIST.md)を参照してください。Windowsコード署名は現在未導入で、将来の導入ポイントは[Windowsコード署名手順](docs/WINDOWS_CODE_SIGNING.md)に整理しています。

## 構成

```text
src/
  main.ts                    GUIと画面状態
  services/backend.ts        Tauriコマンドとの境界

src-tauri/
  components.json            取得物の固定マニフェスト
  src/components/
    manifest.rs              マニフェスト読込・検証
    paths.rs                 アプリデータ保存場所
    download.rs              HTTPS取得・進捗・SHA-256・安全な展開
    mod.rs                   状態判定・設定接続・モデル管理
  src/engine.rs              既存whisper.cpp / FFmpegアダプター
  src/audio.rs               マイク入力
  src/settings.rs            設定の永続化
  src/storage.rs             TXT保存
```

ダウンロード処理と既存文字起こしエンジンは、保存された実行パスを境界に分離されています。

## 現在の制限

- ダウンロードの一時停止・再開・キャンセルは未実装です。失敗後は再取得してください。
- 自動更新確認は行いません。推奨版の更新は`components.json`とライセンス文書を更新してアプリをリリースする方式です。
- Windows x64用配布物のみを管理しています。ARM64対応には別アーティファクトとハッシュが必要です。
- カタカナ語、専門用語、固有名詞などは、音質や使用モデルによって誤認識することがあります。
- smallは軽量な標準モデルです。medium / large-v3は精度向上を期待できますが、必要容量と処理負荷が増えます。
- リアルタイム認識は録音チャンクを繰り返し推論する方式のため、数秒の遅延があります。
- Windows配布物は現在未署名のため、初回実行時にSmartScreenの警告が表示される場合があります。

## ライセンス

MisfoShift名義で公開するアプリのソースコードは[LICENSE](LICENSE)を参照してください。オリジナル画像`transcriber.png`と、その画像から生成した`src-tauri/icons/`内のアプリアイコンもアプリ本体と同じMITライセンスで公開します。第三者コンポーネントにはそれぞれのライセンスが適用されます。標準配布ではFFmpegバイナリを同梱しません。将来同梱へ変更する場合は、[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)に記載したLGPL条件を満たしてください。
