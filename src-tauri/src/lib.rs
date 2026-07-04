// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod config_writer;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn write_provider_config(
    config_dir: Option<String>,
    provider: config_writer::CodexProviderConfig,
) -> Result<(), String> {
    let config_dir = resolve_config_dir(config_dir)?;
    config_writer::write_codex_config(&config_dir, &provider)
}

fn resolve_config_dir(config_dir: Option<String>) -> Result<std::path::PathBuf, String> {
    resolve_config_dir_with_default(config_dir, config_writer::default_codex_dir)
}

fn resolve_config_dir_with_default(
    config_dir: Option<String>,
    default_codex_dir: impl FnOnce() -> Result<std::path::PathBuf, String>,
) -> Result<std::path::PathBuf, String> {
    match config_dir {
        Some(path) if !path.trim().is_empty() => Ok(std::path::PathBuf::from(path)),
        _ => default_codex_dir(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn resolve_config_dir_uses_non_blank_custom_path() {
        let resolved =
            resolve_config_dir_with_default(Some("C:\\custom\\.codex".to_string()), || {
                panic!("default path should not be used")
            })
            .expect("custom path");

        assert_eq!(resolved, PathBuf::from("C:\\custom\\.codex"));
    }

    #[test]
    fn resolve_config_dir_uses_default_for_blank_custom_path() {
        let default_dir = PathBuf::from("C:\\fake-user\\.codex");
        let resolved =
            resolve_config_dir_with_default(Some("  ".to_string()), || Ok(default_dir.clone()))
                .expect("default path");

        assert_eq!(resolved, default_dir);
    }

    #[test]
    fn resolve_config_dir_uses_default_when_custom_path_is_absent() {
        let default_dir = PathBuf::from("C:\\fake-user\\.codex");
        let resolved = resolve_config_dir_with_default(None, || Ok(default_dir.clone()))
            .expect("default path");

        assert_eq!(resolved, default_dir);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, write_provider_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
