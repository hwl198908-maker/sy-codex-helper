use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, process::Command, time::Duration};
use tauri::Emitter;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct InstallStatus {
    pub installed: bool,
    pub installed_version: Option<String>,
    pub available_version: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub phase: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
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
pub fn download_and_open_codex(
    app: tauri::AppHandle,
    base_url: String,
) -> Result<InstallStatus, String> {
    let manifest = read_mirror_manifest(base_url)?;
    let package = select_codex_windows_package(&manifest)
        .ok_or_else(|| "镜像清单中没有 Codex Windows 安装包。".to_string())?;
    emit_download_progress(&app, "准备下载", 0, None);
    let target_path = download_package(package, Some(&app))?;
    emit_download_progress(&app, "正在安装", 0, None);
    open_installer(package, &target_path)?;
    emit_download_progress(&app, "安装完成", 0, None);

    Ok(InstallStatus {
        installed: false,
        installed_version: None,
        available_version: Some(package.version.clone()),
        message: format!(
            "已下载 Codex {} 并打开安装程序，请按安装向导完成安装。",
            package.version
        ),
    })
}

#[tauri::command]
pub fn open_codex() -> Result<(), String> {
    let executable = find_codex_executable()
        .ok_or_else(|| "没有找到 Codex 可执行文件，请先完成安装。".to_string())?;
    Command::new(executable)
        .spawn()
        .map_err(|err| format!("打开 Codex 失败：{err}"))?;
    Ok(())
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
    if let Some(manifest) = read_local_manifest(&base_url)? {
        return Ok(manifest);
    }
    if let Some(manifest) = direct_package_manifest(&base_url)? {
        return Ok(manifest);
    }

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

fn download_package(
    package: &MirrorToolPackage,
    app: Option<&tauri::AppHandle>,
) -> Result<std::path::PathBuf, String> {
    let local_path = Path::new(&package.package_url);
    if local_path.exists() {
        return Ok(local_path.to_path_buf());
    }

    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| "无法定位下载缓存目录。".to_string())?
        .join("codex-manager")
        .join("downloads");
    fs::create_dir_all(&cache_dir).map_err(|err| format!("创建下载缓存目录失败：{err}"))?;

    let target_path = cache_dir.join(download_file_name(package));
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|err| format!("创建下载请求失败：{err}"))?;
    let mut response = client
        .get(&package.package_url)
        .send()
        .map_err(|err| format!("下载 Codex 安装包失败：{err}"))?;

    if !response.status().is_success() {
        return Err(format!("安装包下载地址返回异常状态：{}", response.status()));
    }

    let total_bytes = response.content_length();
    let mut file =
        fs::File::create(&target_path).map_err(|err| format!("创建安装包文件失败：{err}"))?;
    let mut downloaded_bytes = 0_u64;
    let mut last_emitted_bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|err| format!("下载安装包失败：{err}"))?;
        if read == 0 {
            break;
        }

        std::io::Write::write_all(&mut file, &buffer[..read])
            .map_err(|err| format!("保存安装包失败：{err}"))?;
        downloaded_bytes += read as u64;

        if downloaded_bytes == total_bytes.unwrap_or(downloaded_bytes)
            || downloaded_bytes.saturating_sub(last_emitted_bytes) >= 1024 * 1024
        {
            if let Some(app) = app {
                emit_download_progress(app, "正在下载", downloaded_bytes, total_bytes);
            }
            last_emitted_bytes = downloaded_bytes;
        }
    }
    drop(file);

    if let Some(app) = app {
        emit_download_progress(app, "正在校验", downloaded_bytes, total_bytes);
    }
    verify_package_checksum(package, &target_path)?;

    Ok(target_path)
}

fn emit_download_progress(
    app: &tauri::AppHandle,
    phase: &str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
) {
    let _ = app.emit(
        "codex-download-progress",
        DownloadProgress {
            phase: phase.to_string(),
            downloaded_bytes,
            total_bytes,
        },
    );
}

fn verify_package_checksum(package: &MirrorToolPackage, path: &Path) -> Result<(), String> {
    if matches!(
        package.checksum_sha256.as_str(),
        "local-file" | "mirror-direct"
    ) {
        return Ok(());
    }

    let expected = package.checksum_sha256.to_ascii_lowercase();
    let actual = file_sha256(path)?;
    if actual != expected {
        return Err("安装包校验失败，请检查镜像文件是否完整。".to_string());
    }

    Ok(())
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|err| format!("读取安装包失败：{err}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| format!("读取安装包失败：{err}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn read_local_manifest(base_url: &str) -> Result<Option<MirrorManifest>, String> {
    let path = Path::new(base_url.trim());
    if !path.exists() {
        return Ok(None);
    }

    if path.is_file() {
        return Ok(Some(MirrorManifest {
            tools: vec![local_package(path)?],
        }));
    }

    let codex_package = fs::read_dir(path)
        .map_err(|err| format!("读取本地镜像目录失败：{err}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|entry_path| entry_path.is_file() && is_supported_install_package(entry_path))
        .ok_or_else(|| "本地镜像目录中没有找到 Windows 安装包。".to_string())?;

    Ok(Some(MirrorManifest {
        tools: vec![local_package(&codex_package)?],
    }))
}

fn local_package(path: &Path) -> Result<MirrorToolPackage, String> {
    let package_url = path
        .to_str()
        .ok_or_else(|| "本地安装包路径包含无法识别的字符。".to_string())?
        .to_string();

    Ok(MirrorToolPackage {
        tool_id: "codex".to_string(),
        version: "local".to_string(),
        platform: "windows-x64".to_string(),
        package_url,
        checksum_sha256: "local-file".to_string(),
        release_notes: Some("本地默认安装包".to_string()),
    })
}

fn download_file_name(package: &MirrorToolPackage) -> String {
    if package
        .package_url
        .eq_ignore_ascii_case("https://codexapp.agentsmirror.com/latest/win")
    {
        return "codex-latest-windows-x64.msix".to_string();
    }

    reqwest::Url::parse(&package.package_url)
        .ok()
        .and_then(|url| {
            url.path_segments()
                .and_then(|mut segments| segments.next_back().map(str::to_string))
        })
        .filter(|name| name.contains('.'))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| format!("codex-{}-{}.exe", package.version, package.platform))
}

fn direct_package_manifest(base_url: &str) -> Result<Option<MirrorManifest>, String> {
    let trimmed = base_url.trim();
    if !is_direct_package_url(trimmed) {
        return Ok(None);
    }
    let parsed = reqwest::Url::parse(trimmed)
        .map_err(|_| "安装包地址无效，请填写完整的 http(s) 地址。".to_string())?;

    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("安装包地址无效，请填写完整的 http(s) 地址。".to_string());
    }

    Ok(Some(MirrorManifest {
        tools: vec![MirrorToolPackage {
            tool_id: "codex".to_string(),
            version: "latest".to_string(),
            platform: "windows-x64".to_string(),
            package_url: trimmed.to_string(),
            checksum_sha256: "mirror-direct".to_string(),
            release_notes: Some("Codex App 镜像最新 Windows 包".to_string()),
        }],
    }))
}

fn is_direct_package_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.ends_with(".exe")
        || lower.ends_with(".msix")
        || lower.ends_with(".msixbundle")
        || lower.ends_with(".zip")
        || lower == "https://codexapp.agentsmirror.com/latest/win"
}

fn is_supported_install_package(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("exe")
                || extension.eq_ignore_ascii_case("msix")
                || extension.eq_ignore_ascii_case("msixbundle")
                || extension.eq_ignore_ascii_case("zip")
        })
}

fn open_installer(package: &MirrorToolPackage, path: &std::path::Path) -> Result<(), String> {
    if package.package_url.to_ascii_lowercase().ends_with(".zip")
        || path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        return install_zip_package(path);
    }

    Command::new("cmd")
        .args(["/C", "start", "", &path.to_string_lossy()])
        .spawn()
        .map_err(|err| format!("打开安装程序失败：{err}"))?;
    Ok(())
}

fn install_zip_package(path: &std::path::Path) -> Result<(), String> {
    let install_dir = dirs::data_local_dir()
        .ok_or_else(|| "无法定位本地 Codex 安装目录。".to_string())?
        .join("OpenAI")
        .join("Codex");
    fs::create_dir_all(&install_dir).map_err(|err| format!("创建 Codex 安装目录失败：{err}"))?;

    extract_zip_package(path, &install_dir)?;

    if find_codex_in_openai_install().is_none() {
        return Err("ZIP 包已解压，但没有找到 codex.exe。".to_string());
    }

    Ok(())
}

fn extract_zip_package(
    path: &std::path::Path,
    install_dir: &std::path::Path,
) -> Result<(), String> {
    let tar_result = Command::new("tar")
        .arg("-xf")
        .arg(path)
        .arg("-C")
        .arg(install_dir)
        .output();

    match tar_result {
        Ok(output) if output.status.success() => return Ok(()),
        Ok(output) => {
            let tar_error = output_to_string(&output);
            match extract_zip_with_powershell(path, install_dir) {
                Ok(()) => return Ok(()),
                Err(message) => {
                    return Err(format!(
                        "{}\n{}",
                        format_extract_failure("tar", &tar_error),
                        message
                    ))
                }
            }
        }
        Err(err) => match extract_zip_with_powershell(path, install_dir) {
            Ok(()) => return Ok(()),
            Err(message) => {
                return Err(format!(
                    "{}\n{}",
                    format_extract_failure("tar", &err.to_string()),
                    message
                ))
            }
        },
    }
}

fn extract_zip_with_powershell(
    path: &std::path::Path,
    install_dir: &std::path::Path,
) -> Result<(), String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Expand-Archive -LiteralPath $args[0] -DestinationPath $args[1] -Force",
        ])
        .arg(path)
        .arg(install_dir)
        .output()
        .map_err(|err| format_extract_failure("PowerShell Expand-Archive", &err.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    Err(format_extract_failure(
        "PowerShell Expand-Archive",
        &output_to_string(&output),
    ))
}

fn output_to_string(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn format_extract_failure(tool: &str, detail: &str) -> String {
    let detail = detail.trim();
    if detail.is_empty() {
        return format!("解压 Codex ZIP 包失败：{tool} 未返回详细错误。");
    }

    format!("解压 Codex ZIP 包失败：{tool}：{detail}")
}

fn find_codex_executable() -> Option<std::path::PathBuf> {
    std::env::var_os("CODEX_CLI_PATH")
        .map(std::path::PathBuf::from)
        .filter(|path| path.exists())
        .or_else(find_codex_in_openai_install)
        .or_else(|| find_codex_on_path())
}

fn find_codex_in_openai_install() -> Option<std::path::PathBuf> {
    let base_dir = dirs::data_local_dir()?
        .join("OpenAI")
        .join("Codex")
        .join("bin");
    find_codex_under_bin_dir(&base_dir)
}

fn find_codex_under_bin_dir(base_dir: &Path) -> Option<std::path::PathBuf> {
    fs::read_dir(base_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("codex.exe"))
        .find(|path| path.exists())
}

fn find_codex_on_path() -> Option<std::path::PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join("codex.exe"))
        .find(|path| path.exists())
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

    #[test]
    fn builds_safe_download_file_name() {
        let package = MirrorToolPackage {
            tool_id: "codex".to_string(),
            version: "2.3.4".to_string(),
            platform: "windows-x64".to_string(),
            package_url: "https://mirror.test/releases/codex-2.3.4.exe".to_string(),
            checksum_sha256: "codex-checksum".to_string(),
            release_notes: None,
        };

        assert_eq!(download_file_name(&package), "codex-2.3.4.exe");
    }

    #[test]
    fn falls_back_to_versioned_download_file_name() {
        let package = MirrorToolPackage {
            tool_id: "codex".to_string(),
            version: "2.3.4".to_string(),
            platform: "windows-x64".to_string(),
            package_url: "https://mirror.test/releases/".to_string(),
            checksum_sha256: "codex-checksum".to_string(),
            release_notes: None,
        };

        assert_eq!(download_file_name(&package), "codex-2.3.4-windows-x64.exe");
    }

    #[test]
    fn treats_agentsmirror_latest_win_as_direct_package() {
        let manifest = direct_package_manifest("https://codexapp.agentsmirror.com/latest/win")
            .expect("direct package")
            .expect("manifest");
        let package = select_codex_windows_package(&manifest).expect("codex package");

        assert_eq!(package.tool_id, "codex");
        assert_eq!(package.version, "latest");
        assert_eq!(
            package.package_url,
            "https://codexapp.agentsmirror.com/latest/win"
        );
    }

    #[test]
    fn gives_direct_latest_win_a_msix_file_name() {
        let package = MirrorToolPackage {
            tool_id: "codex".to_string(),
            version: "latest".to_string(),
            platform: "windows-x64".to_string(),
            package_url: "https://codexapp.agentsmirror.com/latest/win".to_string(),
            checksum_sha256: "mirror-direct".to_string(),
            release_notes: None,
        };

        assert_eq!(
            download_file_name(&package),
            "codex-latest-windows-x64.msix"
        );
    }

    #[test]
    fn treats_zip_url_as_direct_package() {
        let manifest = direct_package_manifest("https://mirror.test/codex-clean.zip")
            .expect("direct package")
            .expect("manifest");
        let package = select_codex_windows_package(&manifest).expect("codex package");

        assert_eq!(package.package_url, "https://mirror.test/codex-clean.zip");
        assert_eq!(download_file_name(package), "codex-clean.zip");
    }

    #[test]
    fn calculates_file_sha256() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let package_path = temp_dir.path().join("package.zip");
        fs::write(&package_path, b"codex package").expect("package");

        assert_eq!(
            file_sha256(&package_path).expect("sha256"),
            "ecec675b44cd12878ff1d953dc4f7f2df6a3761f01ba66ba8a39106fcc0ad114"
        );
    }

    #[test]
    fn formats_extract_failure_with_tool_output() {
        let message = format_extract_failure("tar", "cannot create file");

        assert!(message.contains("tar"));
        assert!(message.contains("cannot create file"));
    }

    #[test]
    fn reads_local_directory_as_manifest() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let package_path = temp_dir.path().join("Codex Installer.zip");
        fs::write(&package_path, "fake installer").expect("package");

        let manifest = read_local_manifest(temp_dir.path().to_str().expect("path"))
            .expect("local manifest")
            .expect("manifest");
        let package = select_codex_windows_package(&manifest).expect("codex package");

        assert_eq!(package.version, "local");
        assert_eq!(package.package_url, package_path.to_str().expect("path"));
    }

    #[test]
    fn finds_codex_under_versioned_bin_dir() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let codex_dir = temp_dir.path().join("abc123");
        fs::create_dir_all(&codex_dir).expect("codex dir");
        let codex_path = codex_dir.join("codex.exe");
        fs::write(&codex_path, "fake exe").expect("codex exe");

        assert_eq!(find_codex_under_bin_dir(temp_dir.path()), Some(codex_path));
    }
}
