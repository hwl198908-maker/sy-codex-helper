// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod config_writer;
pub mod diagnostics;
pub mod installer;
pub mod native_menu;
pub mod storage;
pub mod updater;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn write_provider_config(provider: config_writer::CodexProviderConfig) -> Result<(), String> {
    let config_dir = config_writer::default_codex_dir()?;
    config_writer::write_codex_config(&config_dir, &provider)
}

#[tauri::command]
fn save_provider_record(provider: config_writer::CodexProviderConfig) -> Result<(), String> {
    storage::save_provider(&provider)
}

#[tauri::command]
fn get_diagnostic_log_path() -> String {
    diagnostics::log_path().to_string_lossy().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_diagnostic_log_path,
            write_provider_config,
            save_provider_record,
            installer::download_and_open_codex,
            installer::get_install_status,
            installer::open_codex,
            installer::read_mirror_manifest,
            updater::check_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
