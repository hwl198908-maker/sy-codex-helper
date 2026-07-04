use serde_json::{Map, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CodexProviderConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub protocol: String,
    pub default_model: Option<String>,
    pub user_agent: String,
}

pub fn default_codex_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| "无法找到用户主目录".to_string())?;
    Ok(PathBuf::from(home).join(".codex"))
}

pub fn write_codex_config(config_dir: &Path, provider: &CodexProviderConfig) -> Result<(), String> {
    fs::create_dir_all(config_dir).map_err(|err| format!("创建 Codex 配置目录失败: {err}"))?;

    let config_path = config_dir.join("config.toml");
    let auth_path = config_dir.join("auth.json");
    backup_if_exists(&config_path)?;
    backup_if_exists(&auth_path)?;

    fs::write(&config_path, build_config_toml(provider))
        .map_err(|err| format!("写入 config.toml 失败: {err}"))?;
    fs::write(&auth_path, build_auth_json(&auth_path, &provider.api_key)?)
        .map_err(|err| format!("写入 auth.json 失败: {err}"))?;

    Ok(())
}

fn backup_if_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let backup_path = next_backup_path(path);
    fs::copy(path, backup_path).map_err(|err| {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("配置文件");
        format!("备份 {name} 失败: {err}")
    })?;
    Ok(())
}

fn next_backup_path(path: &Path) -> PathBuf {
    let first = PathBuf::from(format!("{}.bak", path.display()));
    if !first.exists() {
        return first;
    }

    for index in 1.. {
        let candidate = PathBuf::from(format!("{}.bak.{index}", path.display()));
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("backup index loop should always return")
}

fn build_config_toml(provider: &CodexProviderConfig) -> String {
    let model = provider
        .default_model
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or("gpt-5");

    format!(
        concat!(
            "model = {}\n",
            "model_provider = \"custom\"\n",
            "cli_auth_credentials_store = \"file\"\n",
            "\n",
            "[model_providers.custom]\n",
            "name = {}\n",
            "base_url = {}\n",
            "wire_api = \"responses\"\n",
            "requires_openai_auth = true\n"
        ),
        toml_string(model),
        toml_string(&provider.name),
        toml_string(&provider.base_url)
    )
}

fn build_auth_json(auth_path: &Path, api_key: &str) -> Result<String, String> {
    let mut auth = match fs::read_to_string(auth_path) {
        Ok(content) => match serde_json::from_str::<Value>(&content)
            .map_err(|err| format!("读取 auth.json 失败: {err}"))?
        {
            Value::Object(object) => object,
            _ => Map::new(),
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Map::new(),
        Err(err) => return Err(format!("读取 auth.json 失败: {err}")),
    };

    auth.insert(
        "OPENAI_API_KEY".to_string(),
        Value::String(api_key.to_string()),
    );

    serde_json::to_string_pretty(&Value::Object(auth))
        .map_err(|err| format!("写入 auth.json 失败: {err}"))
}

fn toml_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn writes_config_and_auth_with_backups() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let config_dir = temp_dir.path();
        fs::write(config_dir.join("config.toml"), "old_config = true\n").expect("old config");
        fs::write(
            config_dir.join("auth.json"),
            r#"{"OTHER_FIELD":"keep-me","OPENAI_API_KEY":"old-key"}"#,
        )
        .expect("old auth");

        let provider = CodexProviderConfig {
            name: "Proxy Test".to_string(),
            base_url: "https://proxy.test/v1".to_string(),
            api_key: "test-key".to_string(),
            protocol: "chat_completions".to_string(),
            default_model: Some("gpt-test".to_string()),
            user_agent: "CodexManager/1.0".to_string(),
        };

        write_codex_config(config_dir, &provider).expect("write config");

        assert_eq!(
            fs::read_to_string(config_dir.join("config.toml.bak")).expect("config backup"),
            "old_config = true\n"
        );
        assert_eq!(
            fs::read_to_string(config_dir.join("auth.json.bak")).expect("auth backup"),
            r#"{"OTHER_FIELD":"keep-me","OPENAI_API_KEY":"old-key"}"#
        );

        let config_toml = fs::read_to_string(config_dir.join("config.toml")).expect("new config");
        assert!(config_toml.contains(r#"cli_auth_credentials_store = "file""#));
        assert!(config_toml.contains(r#"model_provider = "custom""#));
        assert!(config_toml.contains(r#"base_url = "https://proxy.test/v1""#));
        assert!(config_toml.contains(r#"wire_api = "responses""#));
        assert!(config_toml.contains("requires_openai_auth = true"));
        assert!(!config_toml.contains("User-Agent"));
        assert!(!config_toml.contains("CodexManager/1.0"));

        let auth_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(config_dir.join("auth.json")).expect("auth"))
                .expect("valid auth json");
        assert_eq!(auth_json["OPENAI_API_KEY"], "test-key");
        assert_eq!(auth_json["OTHER_FIELD"], "keep-me");
    }

    #[test]
    fn does_not_overwrite_existing_backup_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let config_dir = temp_dir.path();
        fs::write(config_dir.join("config.toml"), "old_config = true\n").expect("old config");
        fs::write(config_dir.join("config.toml.bak"), "first backup\n").expect("first backup");

        let provider = CodexProviderConfig {
            name: "Proxy Test".to_string(),
            base_url: "https://proxy.test/v1".to_string(),
            api_key: "test-key".to_string(),
            protocol: "responses".to_string(),
            default_model: Some("gpt-test".to_string()),
            user_agent: "CodexManager/1.0".to_string(),
        };

        write_codex_config(config_dir, &provider).expect("write config");

        assert_eq!(
            fs::read_to_string(config_dir.join("config.toml.bak")).expect("first backup"),
            "first backup\n"
        );
        assert_eq!(
            fs::read_to_string(config_dir.join("config.toml.bak.1")).expect("numbered backup"),
            "old_config = true\n"
        );
    }
}
