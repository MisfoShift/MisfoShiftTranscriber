mod audio;
mod commands;
mod components;
mod engine;
mod error;
mod settings;
mod storage;

use audio::MicrophoneManager;
use commands::{
    clear_saved_settings, component_overview, delete_managed_model, engine_status,
    install_managed_component, list_microphones, load_settings, save_settings, save_text_file,
    select_managed_model, setup_recommended_components, start_microphone, stop_microphone,
    transcribe_audio_file,
};
use components::ComponentManager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(MicrophoneManager::default())
        .manage(ComponentManager::default())
        .invoke_handler(tauri::generate_handler![
            component_overview,
            install_managed_component,
            setup_recommended_components,
            select_managed_model,
            delete_managed_model,
            list_microphones,
            load_settings,
            save_settings,
            clear_saved_settings,
            engine_status,
            start_microphone,
            stop_microphone,
            transcribe_audio_file,
            save_text_file,
        ])
        .run(tauri::generate_context!())
        .expect("MisfoShiftTranscriberの起動中にエラーが発生しました");
}
