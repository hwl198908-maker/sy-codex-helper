use std::time::Duration;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifest {
    pub version: String,
    pub download_url: String,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn check_update(manifest_url: String) -> Result<UpdateManifest, String> {
    let parsed = reqwest::Url::parse(manifest_url.trim())
        .map_err(|_| "更新地址无效，请检查版本清单地址。".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("更新地址无效，请使用完整的 http(s) 地址。".to_string());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|err| format!("创建更新请求失败：{err}"))?;
    let response = client
        .get(parsed)
        .send()
        .map_err(|err| format!("无法连接更新服务器：{err}"))?;

    if !response.status().is_success() {
        return Err(format!("更新服务器返回异常状态：{}", response.status()));
    }

    response
        .json::<UpdateManifest>()
        .map_err(|err| format!("更新清单格式无效：{err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_camel_case_update_manifest() {
        let manifest: UpdateManifest = serde_json::from_str(
            r#"{"version":"0.1.1","downloadUrl":"https://example.com/app.exe","notes":"UI update"}"#,
        )
        .expect("manifest");

        assert_eq!(manifest.version, "0.1.1");
        assert_eq!(manifest.download_url, "https://example.com/app.exe");
        assert_eq!(manifest.notes.as_deref(), Some("UI update"));
    }
}
