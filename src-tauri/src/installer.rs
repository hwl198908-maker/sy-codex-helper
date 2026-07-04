use std::time::Duration;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct InstallStatus {
    pub installed: bool,
    pub installed_version: Option<String>,
    pub available_version: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MirrorManifest {
    pub tools: Vec<MirrorToolPackage>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorToolPackage {
    pub tool_id: String,
    pub version: String,
    pub platform: String,
    pub package_url: String,
    pub checksum_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<String>,
}

pub fn select_codex_windows_package(manifest: &MirrorManifest) -> Option<&MirrorToolPackage> {
    manifest
        .tools
        .iter()
        .find(|tool| tool.tool_id == "codex" && tool.platform == "windows-x64")
}

#[tauri::command]
pub fn get_install_status() -> Result<InstallStatus, String> {
    Ok(InstallStatus {
        installed: false,
        installed_version: None,
        available_version: None,
        message: "尚未检测本地安装状态。".to_string(),
    })
}

#[tauri::command]
pub fn read_mirror_manifest(base_url: String) -> Result<MirrorManifest, String> {
    let manifest_url = build_manifest_url(&base_url)?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|err| format!("创建网络请求失败：{err}"))?;
    let response = client
        .get(manifest_url)
        .send()
        .map_err(|err| format!("无法连接镜像源：{err}"))?;

    if !response.status().is_success() {
        return Err(format!("镜像源返回异常状态：{}", response.status()));
    }

    response
        .json::<MirrorManifest>()
        .map_err(|err| format!("镜像清单格式无效：{err}"))
}

fn build_manifest_url(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    let parsed = reqwest::Url::parse(trimmed)
        .map_err(|_| "镜像地址无效，请填写完整的 http(s) 地址。".to_string())?;

    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("镜像地址无效，请填写完整的 http(s) 地址。".to_string());
    }

    Ok(format!("{trimmed}/manifest.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_codex_windows_package_from_manifest() {
        let manifest = MirrorManifest {
            tools: vec![
                MirrorToolPackage {
                    tool_id: "openclaw".to_string(),
                    version: "1.0.0".to_string(),
                    platform: "windows-x64".to_string(),
                    package_url: "https://mirror.test/openclaw.zip".to_string(),
                    checksum_sha256: "openclaw-checksum".to_string(),
                    release_notes: Some("OpenClaw".to_string()),
                },
                MirrorToolPackage {
                    tool_id: "codex".to_string(),
                    version: "2.3.4".to_string(),
                    platform: "windows-x64".to_string(),
                    package_url: "https://mirror.test/codex-2.3.4.zip".to_string(),
                    checksum_sha256: "codex-checksum".to_string(),
                    release_notes: Some("Codex release".to_string()),
                },
            ],
        };

        let package = select_codex_windows_package(&manifest).expect("codex package");

        assert_eq!(package.version, "2.3.4");
        assert_eq!(package.package_url, "https://mirror.test/codex-2.3.4.zip");
    }

    #[test]
    fn parses_camel_case_manifest_fields() {
        let manifest: MirrorManifest = serde_json::from_str(
            r#"{
                "tools": [
                    {
                        "toolId": "codex",
                        "version": "2.3.4",
                        "platform": "windows-x64",
                        "packageUrl": "https://mirror.test/codex-2.3.4.zip",
                        "checksumSha256": "codex-checksum",
                        "releaseNotes": "Codex release"
                    }
                ]
            }"#,
        )
        .expect("camelCase manifest");

        let package = select_codex_windows_package(&manifest).expect("codex package");

        assert_eq!(package.version, "2.3.4");
        assert_eq!(package.package_url, "https://mirror.test/codex-2.3.4.zip");

        let serialized = serde_json::to_value(&manifest).expect("serialized manifest");

        assert_eq!(
            serialized["tools"][0]["packageUrl"],
            "https://mirror.test/codex-2.3.4.zip"
        );
        assert!(serialized["tools"][0]["package_url"].is_null());
    }
}
