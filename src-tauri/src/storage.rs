use crate::config_writer::CodexProviderConfig;
use keyring::Entry;
use std::{fs, path::PathBuf};

const SERVICE_NAME: &str = "codex-manager";
const METADATA_FILE: &str = "providers.json";

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ProviderMetadata {
    name: String,
    base_url: String,
    protocol: String,
    default_model: Option<String>,
    user_agent: String,
}

impl From<&CodexProviderConfig> for ProviderMetadata {
    fn from(provider: &CodexProviderConfig) -> Self {
        Self {
            name: provider.name.clone(),
            base_url: provider.base_url.clone(),
            protocol: provider.protocol.clone(),
            default_model: provider.default_model.clone(),
            user_agent: provider.user_agent.clone(),
        }
    }
}

impl ProviderMetadata {
    fn into_provider(self, api_key: String) -> CodexProviderConfig {
        CodexProviderConfig {
            name: self.name,
            base_url: self.base_url,
            api_key,
            protocol: self.protocol,
            default_model: self.default_model,
            user_agent: self.user_agent,
        }
    }
}

pub fn save_provider(provider: &CodexProviderConfig) -> Result<(), String> {
    keyring_entry(&provider.name)?
        .set_password(&provider.api_key)
        .map_err(|err| format!("保存 API Key 到系统凭据失败: {err}"))?;

    let mut records = read_metadata_records()?;
    let metadata = ProviderMetadata::from(provider);
    if let Some(existing) = records
        .iter_mut()
        .find(|record| record.name == provider.name)
    {
        *existing = metadata;
    } else {
        records.push(metadata);
    }
    write_metadata_records(&records)
}

pub fn load_provider(name: &str) -> Result<Option<CodexProviderConfig>, String> {
    let Some(metadata) = read_metadata_records()?
        .into_iter()
        .find(|record| record.name == name)
    else {
        return Ok(None);
    };

    let api_key = keyring_entry(name)?
        .get_password()
        .map_err(|err| format!("读取 API Key 失败: {err}"))?;
    Ok(Some(metadata.into_provider(api_key)))
}

fn keyring_entry(provider_name: &str) -> Result<Entry, String> {
    Entry::new(SERVICE_NAME, &format!("provider:{provider_name}"))
        .map_err(|err| format!("打开系统凭据失败: {err}"))
}

fn metadata_path() -> Result<PathBuf, String> {
    let base_dir = dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .ok_or_else(|| "无法找到应用数据目录".to_string())?;
    Ok(base_dir.join(SERVICE_NAME).join(METADATA_FILE))
}

fn read_metadata_records() -> Result<Vec<ProviderMetadata>, String> {
    let path = metadata_path()?;
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(format!("读取 Provider 元数据失败: {err}")),
    };
    serde_json::from_str(&content).map_err(|err| format!("解析 Provider 元数据失败: {err}"))
}

fn write_metadata_records(records: &[ProviderMetadata]) -> Result<(), String> {
    let path = metadata_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定应用数据目录".to_string())?;
    fs::create_dir_all(parent).map_err(|err| format!("创建应用数据目录失败: {err}"))?;
    let content = serde_json::to_string_pretty(records)
        .map_err(|err| format!("序列化 Provider 元数据失败: {err}"))?;
    fs::write(path, content).map_err(|err| format!("写入 Provider 元数据失败: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_provider() -> CodexProviderConfig {
        CodexProviderConfig {
            name: "Unit Test Provider".to_string(),
            base_url: "https://unit.test/v1".to_string(),
            api_key: "unit-test-key".to_string(),
            protocol: "responses".to_string(),
            default_model: Some("unit-model".to_string()),
            user_agent: "CodexManager/1.0".to_string(),
        }
    }

    #[test]
    fn provider_metadata_serialization_excludes_api_key() {
        let metadata = ProviderMetadata::from(&test_provider());
        let json = serde_json::to_string(&metadata).expect("serialize metadata");

        assert!(json.contains("Unit Test Provider"));
        assert!(json.contains("https://unit.test/v1"));
        assert!(!json.contains("api_key"));
        assert!(!json.contains("unit-test-key"));
    }

    #[test]
    #[ignore = "requires an OS credential store"]
    fn keyring_round_trip_uses_provider_name_entry() {
        let provider_name = "Unit Test Provider Keyring";
        let api_key = "unit-test-key";
        let entry = keyring_entry(provider_name).expect("keyring entry");
        let _ = entry.delete_credential();

        entry.set_password(api_key).expect("set test credential");

        assert_eq!(entry.get_password().expect("get test credential"), api_key);

        entry.delete_credential().expect("delete test credential");
    }
}
