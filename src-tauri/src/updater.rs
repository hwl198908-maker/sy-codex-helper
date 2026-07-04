use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifest {
    pub version: String,
    pub download_url: String,
    pub sha256: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: Option<u8>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstallResult {
    pub version: String,
    pub path: String,
    pub reused_cached_file: bool,
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

#[tauri::command]
pub fn download_and_install_update(
    app: AppHandle,
    manifest: UpdateManifest,
) -> Result<UpdateInstallResult, String> {
    let parsed = reqwest::Url::parse(manifest.download_url.trim())
        .map_err(|_| "新版下载地址无效，请检查更新清单。".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("新版下载地址无效，请使用完整的 http(s) 地址。".to_string());
    }

    let cache_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|err| format!("无法定位更新缓存目录：{err}"))?
        .join("updates");
    fs::create_dir_all(&cache_dir).map_err(|err| format!("无法创建更新缓存目录：{err}"))?;

    let target_path = cache_dir.join(update_cache_file_name(&manifest));
    if target_path.exists() && verify_cached_update(&target_path, manifest.sha256.as_deref())? {
        launch_installer(&target_path)?;
        return Ok(UpdateInstallResult {
            version: manifest.version,
            path: target_path.to_string_lossy().to_string(),
            reused_cached_file: true,
        });
    }

    let partial_path = target_path.with_extension("download");
    download_update_file(&app, parsed, &partial_path)?;
    verify_cached_update(&partial_path, manifest.sha256.as_deref())?;
    fs::rename(&partial_path, &target_path).map_err(|err| format!("保存新版安装包失败：{err}"))?;
    launch_installer(&target_path)?;

    Ok(UpdateInstallResult {
        version: manifest.version,
        path: target_path.to_string_lossy().to_string(),
        reused_cached_file: false,
    })
}

fn download_update_file(
    app: &AppHandle,
    url: reqwest::Url,
    target_path: &Path,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|err| format!("创建下载请求失败：{err}"))?;
    let mut response = client
        .get(url)
        .send()
        .map_err(|err| format!("无法下载新版安装包：{err}"))?;

    if !response.status().is_success() {
        return Err(format!("新版下载服务器返回异常状态：{}", response.status()));
    }

    let total = response.content_length();
    let mut file = fs::File::create(target_path).map_err(|err| format!("创建下载文件失败：{err}"))?;
    let mut buffer = [0_u8; 64 * 1024];
    let mut downloaded = 0_u64;

    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|err| format!("读取新版安装包失败：{err}"))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|err| format!("写入新版安装包失败：{err}"))?;
        downloaded += read as u64;
        emit_download_progress(app, downloaded, total);
    }

    file.flush()
        .map_err(|err| format!("保存新版安装包失败：{err}"))?;
    emit_download_progress(app, downloaded, total);
    Ok(())
}

fn emit_download_progress(app: &AppHandle, downloaded: u64, total: Option<u64>) {
    let percent = total
        .filter(|value| *value > 0)
        .map(|value| ((downloaded.saturating_mul(100)) / value).min(100) as u8);
    let _ = app.emit(
        "update-download-progress",
        UpdateDownloadProgress {
            downloaded,
            total,
            percent,
        },
    );
}

fn launch_installer(path: &Path) -> Result<(), String> {
    Command::new(path)
        .spawn()
        .map_err(|err| format!("启动新版安装包失败：{err}"))?;
    Ok(())
}

fn verify_cached_update(path: &Path, expected_sha256: Option<&str>) -> Result<bool, String> {
    let Some(expected_sha256) = expected_sha256 else {
        return Ok(path.exists());
    };

    let actual = file_sha256(path)?;
    Ok(actual.eq_ignore_ascii_case(expected_sha256.trim()))
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|err| format!("读取新版安装包失败：{err}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| format!("校验新版安装包失败：{err}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn update_cache_file_name(manifest: &UpdateManifest) -> String {
    let safe_version: String = manifest
        .version
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() || matches!(value, '.' | '-' | '_') {
                value
            } else {
                '_'
            }
        })
        .collect();
    format!("SY-Codex_{safe_version}_x64-setup.exe")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_camel_case_update_manifest() {
        let manifest: UpdateManifest = serde_json::from_str(
            r#"{"version":"0.1.1","downloadUrl":"https://example.com/app.exe","sha256":"abc123","notes":"UI update"}"#,
        )
        .expect("manifest");

        assert_eq!(manifest.version, "0.1.1");
        assert_eq!(manifest.download_url, "https://example.com/app.exe");
        assert_eq!(manifest.sha256.as_deref(), Some("abc123"));
        assert_eq!(manifest.notes.as_deref(), Some("UI update"));
    }

    #[test]
    fn builds_safe_update_cache_file_name() {
        let manifest = UpdateManifest {
            version: "0.1.9 beta/1".to_string(),
            download_url: "https://example.com/app.exe".to_string(),
            sha256: None,
            notes: None,
        };

        assert_eq!(
            update_cache_file_name(&manifest),
            "SY-Codex_0.1.9_beta_1_x64-setup.exe"
        );
    }

    #[test]
    fn verifies_update_sha256() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("update.exe");
        let mut file = fs::File::create(&path).expect("file");
        write!(file, "update payload").expect("write");

        let expected = file_sha256(&path).expect("sha");
        assert!(verify_cached_update(&path, Some(&expected)).expect("verify"));
        assert!(!verify_cached_update(&path, Some("bad")).expect("verify"));
    }
}
