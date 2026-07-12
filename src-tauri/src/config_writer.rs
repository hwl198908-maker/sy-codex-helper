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
    let existing_config = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(format!("读取 config.toml 失败: {err}")),
    };
    backup_if_exists(&config_path)?;
    backup_if_exists(&auth_path)?;

    fs::write(&config_path, build_config_toml(&existing_config, provider))
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

fn build_config_toml(existing_config: &str, provider: &CodexProviderConfig) -> String {
    let preserved_config = preserve_unmanaged_config(existing_config);
    let managed_config = build_managed_config_toml(provider);

    if preserved_config.trim().is_empty() {
        managed_config
    } else {
        format!("{}\n\n{}", managed_config.trim_end(), preserved_config.trim_start())
    }
}

fn preserve_unmanaged_config(existing_config: &str) -> String {
    let mut preserved = Vec::new();
    let mut in_managed_provider = false;

    for line in existing_config.lines() {
        let trimmed = line.trim();

        if is_toml_header(trimmed) {
            in_managed_provider = matches!(
                trimmed,
                "[model_providers.custom]" | "[model_providers.openai]"
            );
        }

        if in_managed_provider {
            continue;
        }

        if is_managed_top_level_key(trimmed) {
            continue;
        }

        preserved.push(line);
    }

    preserved.join("\n")
}

fn is_toml_header(trimmed_line: &str) -> bool {
    trimmed_line.starts_with('[') && trimmed_line.ends_with(']')
}

fn is_managed_top_level_key(trimmed_line: &str) -> bool {
    let Some((key, _)) = trimmed_line.split_once('=') else {
        return false;
    };

    matches!(
        key.trim(),
        "model" | "model_provider" | "cli_auth_credentials_store"
    )
}

fn build_managed_config_toml(provider: &CodexProviderConfig) -> String {
    let model = provider
        .default_model
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or("gpt-5");
    let wire_api = codex_wire_api(&provider.protocol);
    let provider_key = managed_provider_key(provider);

    format!(
        concat!(
            "model = {}\n",
            "model_provider = {}\n",
            "cli_auth_credentials_store = \"file\"\n",
            "\n",
            "[model_providers.{}]\n",
            "name = {}\n",
            "base_url = {}\n",
            "wire_api = {}\n",
            "requires_openai_auth = true\n"
        ),
        toml_string(model),
        toml_string(provider_key),
        provider_key,
        toml_string(&provider.name),
        toml_string(&provider.base_url),
        toml_string(wire_api)
    )
}

fn managed_provider_key(provider: &CodexProviderConfig) -> &'static str {
    if provider.base_url.trim_end_matches('/') == "https://api.openai.com/v1" {
        "openai"
    } else {
        "custom"
    }
}

fn codex_wire_api(protocol: &str) -> &str {
    match protocol {
        "chat_completions" => "chat",
        _ => "responses",
    }
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
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

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
        assert!(config_toml.contains(r#"wire_api = "chat""#));
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
    fn default_codex_dir_falls_back_to_home_for_mac_style_paths() {
        let _guard = env_lock().lock().expect("env lock");
        let old_userprofile = std::env::var_os("USERPROFILE");
        let old_home = std::env::var_os("HOME");
        let temp_dir = tempfile::tempdir().expect("temp dir");

        std::env::remove_var("USERPROFILE");
        std::env::set_var("HOME", temp_dir.path());

        let codex_dir = default_codex_dir().expect("codex dir");

        assert_eq!(codex_dir, temp_dir.path().join(".codex"));

        match old_userprofile {
            Some(value) => std::env::set_var("USERPROFILE", value),
            None => std::env::remove_var("USERPROFILE"),
        }
        match old_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
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

    #[test]
    fn preserves_unrelated_existing_config() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let config_dir = temp_dir.path();
        fs::write(
            config_dir.join("config.toml"),
            concat!(
                "notify = [\"tool\"]\n",
                "\n",
                "[features]\n",
                "memories = true\n",
                "\n",
                "[model_providers.other]\n",
                "name = \"other\"\n",
                "base_url = \"https://other.test/v1\"\n"
            ),
        )
        .expect("old config");

        let provider = CodexProviderConfig {
            name: "Proxy Test".to_string(),
            base_url: "https://proxy.test/v1".to_string(),
            api_key: "test-key".to_string(),
            protocol: "responses".to_string(),
            default_model: Some("gpt-test".to_string()),
            user_agent: "CodexManager/1.0".to_string(),
        };

        write_codex_config(config_dir, &provider).expect("write config");

        let config_toml = fs::read_to_string(config_dir.join("config.toml")).expect("new config");
        assert!(config_toml.contains("notify = [\"tool\"]"));
        assert!(config_toml.contains("[features]\nmemories = true"));
        assert!(config_toml.contains("[model_providers.other]\nname = \"other\""));
        assert!(config_toml.contains(r#"model = "gpt-test""#));
        assert!(config_toml.contains(r#"model_provider = "custom""#));
        assert!(config_toml.contains(r#"cli_auth_credentials_store = "file""#));
        assert!(config_toml.contains("[model_providers.custom]"));
        assert!(config_toml.contains(r#"base_url = "https://proxy.test/v1""#));
    }

    #[test]
    fn replaces_existing_custom_provider_block_and_managed_keys() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let config_dir = temp_dir.path();
        fs::write(
            config_dir.join("config.toml"),
            concat!(
                "model = \"old-model\"\n",
                "model_provider = \"old-provider\"\n",
                "cli_auth_credentials_store = \"old-store\"\n",
                "keep_me = true\n",
                "\n",
                "[model_providers.custom]\n",
                "name = \"old custom\"\n",
                "base_url = \"https://old.test/v1\"\n",
                "wire_api = \"chat\"\n",
                "requires_openai_auth = false\n",
                "\n",
                "[features]\n",
                "memories = true\n"
            ),
        )
        .expect("old config");

        let provider = CodexProviderConfig {
            name: "New Proxy".to_string(),
            base_url: "https://new.test/v1".to_string(),
            api_key: "test-key".to_string(),
            protocol: "responses".to_string(),
            default_model: Some("gpt-new".to_string()),
            user_agent: "CodexManager/1.0".to_string(),
        };

        write_codex_config(config_dir, &provider).expect("write config");

        let config_toml = fs::read_to_string(config_dir.join("config.toml")).expect("new config");
        assert!(config_toml.contains("keep_me = true"));
        assert!(config_toml.contains("[features]\nmemories = true"));
        assert!(config_toml.contains(r#"model = "gpt-new""#));
        assert!(config_toml.contains(r#"name = "New Proxy""#));
        assert!(config_toml.contains(r#"base_url = "https://new.test/v1""#));
        assert!(config_toml.contains(r#"wire_api = "responses""#));
        assert!(config_toml.contains("requires_openai_auth = true"));
        assert!(!config_toml.contains("old-model"));
        assert!(!config_toml.contains("old-provider"));
        assert!(!config_toml.contains("old-store"));
        assert!(!config_toml.contains("old custom"));
        assert!(!config_toml.contains("https://old.test/v1"));
        assert_eq!(config_toml.matches("[model_providers.custom]").count(), 1);
    }

    #[test]
    fn writes_responses_protocol_for_responses_provider() {
        let provider = CodexProviderConfig {
            name: "Proxy Test".to_string(),
            base_url: "https://proxy.test/v1".to_string(),
            api_key: "test-key".to_string(),
            protocol: "responses".to_string(),
            default_model: Some("gpt-test".to_string()),
            user_agent: "CodexManager/1.0".to_string(),
        };

        let config_toml = build_managed_config_toml(&provider);

        assert!(config_toml.contains(r#"wire_api = "responses""#));
    }

    #[test]
    fn uses_the_official_openai_provider_for_the_openai_api_url() {
        let provider = CodexProviderConfig {
            name: "OpenAI 官方".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "test-key".to_string(),
            protocol: "responses".to_string(),
            default_model: Some("gpt-5.5".to_string()),
            user_agent: "CodexManager/1.0".to_string(),
        };

        let config_toml = build_managed_config_toml(&provider);

        assert!(config_toml.contains(r#"model_provider = "openai""#));
        assert!(config_toml.contains("[model_providers.openai]"));
        assert!(!config_toml.contains("[model_providers.custom]"));
    }

    #[test]
    fn repeated_writes_keep_managed_config_at_top_without_duplicates() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let config_dir = temp_dir.path();
        fs::write(
            config_dir.join("config.toml"),
            concat!(
                "model = \"old-model\"\n",
                "model_provider = \"old-provider\"\n",
                "\n",
                "[model_providers.custom]\n",
                "name = \"old custom\"\n",
                "wire_api = \"responses\"\n",
                "requires_openai_auth = true\n",
                "base_url = \"https://old.test/v1\"\n",
                "\n",
                "[features]\n",
                "memories = true\n",
                "\n",
                "[desktop]\n",
                "followUpQueueMode = \"steer\"\n",
                "\n",
                "[projects.'d:\\codex+']\n",
                "trust_level = \"trusted\"\n",
                "\n",
                "[windows]\n",
                "sandbox = \"elevated\"\n"
            ),
        )
        .expect("old config");

        let provider = CodexProviderConfig {
            name: "custom".to_string(),
            base_url: "https://proxy.test/v1".to_string(),
            api_key: "test-key".to_string(),
            protocol: "responses".to_string(),
            default_model: Some("gpt-5.5".to_string()),
            user_agent: "CodexManager/1.0".to_string(),
        };

        write_codex_config(config_dir, &provider).expect("first write");
        write_codex_config(config_dir, &provider).expect("second write");

        let config_toml = fs::read_to_string(config_dir.join("config.toml")).expect("new config");
        assert!(config_toml.starts_with("model = \"gpt-5.5\"\n"));
        assert_eq!(config_toml.matches("\nmodel = ").count(), 0);
        assert_eq!(config_toml.matches("model_provider = ").count(), 1);
        assert_eq!(config_toml.matches("[model_providers.custom]").count(), 1);
        assert!(config_toml.contains("[windows]\nsandbox = \"elevated\""));
        assert!(config_toml.contains("[projects.'d:\\codex+']\ntrust_level = \"trusted\""));
    }
}
