use std::{fs, process::Command, thread, time::Duration};

const MENU_LOCALIZATION_RETRIES: usize = 20;
const MENU_LOCALIZATION_RETRY_DELAY: Duration = Duration::from_millis(500);

#[derive(serde::Deserialize)]
struct InspectorTarget {
    #[serde(rename = "type")]
    target_type: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: Option<String>,
}

pub fn spawn_native_menu_localizer(inspector_port: u16) {
    thread::spawn(move || {
        for _ in 0..MENU_LOCALIZATION_RETRIES {
            if install_native_menu_localizer(inspector_port).is_ok() {
                return;
            }
            thread::sleep(MENU_LOCALIZATION_RETRY_DELAY);
        }
    });
}

fn install_native_menu_localizer(inspector_port: u16) -> Result<(), String> {
    let websocket_url = find_main_process_websocket_url(inspector_port)?;
    evaluate_script(&websocket_url, &native_menu_localizer_script())
}

fn find_main_process_websocket_url(inspector_port: u16) -> Result<String, String> {
    let targets: Vec<InspectorTarget> =
        reqwest::blocking::get(format!("http://127.0.0.1:{inspector_port}/json/list"))
            .map_err(|err| format!("连接 Codex 菜单调试端口失败: {err}"))?
            .json()
            .map_err(|err| format!("读取 Codex 菜单调试目标失败: {err}"))?;

    targets
        .iter()
        .find(|target| {
            target.target_type == "node"
                && target
                    .web_socket_debugger_url
                    .as_deref()
                    .is_some_and(|url| !url.is_empty())
        })
        .or_else(|| {
            targets.iter().find(|target| {
                target
                    .web_socket_debugger_url
                    .as_deref()
                    .is_some_and(|url| !url.is_empty())
            })
        })
        .and_then(|target| target.web_socket_debugger_url.clone())
        .ok_or_else(|| "没有找到 Codex Electron 主进程调试目标。".to_string())
}

fn evaluate_script(websocket_url: &str, script: &str) -> Result<(), String> {
    let script_path = std::env::temp_dir().join("sy-codex-native-menu-localizer.js");
    fs::write(&script_path, script).map_err(|err| format!("准备 Codex 菜单汉化脚本失败: {err}"))?;

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            POWERSHELL_CDP_EVALUATE,
        ])
        .env("SY_CODEX_WS_URL", websocket_url)
        .env("SY_CODEX_MENU_SCRIPT_PATH", script_path)
        .output()
        .map_err(|err| format!("启动 Codex 菜单汉化脚本失败: {err}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Err(if stderr.is_empty() { stdout } else { stderr })
}

const POWERSHELL_CDP_EVALUATE: &str = r#"
$ErrorActionPreference = 'Stop'
$wsUrl = $env:SY_CODEX_WS_URL
$scriptPath = $env:SY_CODEX_MENU_SCRIPT_PATH
$script = [IO.File]::ReadAllText($scriptPath, [Text.Encoding]::UTF8)
$payload = @{
  id = 1
  method = 'Runtime.evaluate'
  params = @{
    expression = $script
    awaitPromise = $true
    returnByValue = $true
  }
} | ConvertTo-Json -Depth 8 -Compress
$client = [Net.WebSockets.ClientWebSocket]::new()
$token = [Threading.CancellationToken]::None
$client.ConnectAsync([Uri]$wsUrl, $token).GetAwaiter().GetResult()
$bytes = [Text.Encoding]::UTF8.GetBytes($payload)
$client.SendAsync([ArraySegment[byte]]::new($bytes), [Net.WebSockets.WebSocketMessageType]::Text, $true, $token).GetAwaiter().GetResult()
for ($i = 0; $i -lt 20; $i++) {
  $buffer = New-Object byte[] 65536
  $segment = [ArraySegment[byte]]::new($buffer)
  $text = ''
  do {
    $result = $client.ReceiveAsync($segment, $token).GetAwaiter().GetResult()
    $text += [Text.Encoding]::UTF8.GetString($buffer, 0, $result.Count)
  } until ($result.EndOfMessage)
  if ($text -match '"id"\s*:\s*1') {
    if ($text -match '"exceptionDetails"') { exit 2 }
    exit 0
  }
}
exit 3
"#;

pub fn native_menu_localizer_script() -> String {
    let translations = serde_json::to_string(&menu_label_translations()).unwrap_or_default();
    format!(
        r#"
(() => {{
  const translations = new Map({translations});
  const electron = process.mainModule?.require?.("electron");
  if (!electron?.Menu) return JSON.stringify({{ status: "skipped" }});
  const Menu = electron.Menu;
  let changed = 0;
  const translateItem = (item) => {{
    if (!item) return;
    const nextLabel = translations.get(item.label);
    if (nextLabel && item.label !== nextLabel) {{
      item.label = nextLabel;
      changed += 1;
    }}
    if (item.submenu?.items) {{
      for (const child of item.submenu.items) translateItem(child);
    }}
  }};
  const translateMenu = (menu) => {{
    if (!menu?.items) return menu;
    for (const item of menu.items) translateItem(item);
    return menu;
  }};
  if (!globalThis.__syCodexNativeMenuLocalizerInstalled) {{
    globalThis.__syCodexNativeMenuLocalizerInstalled = true;
    const originalSetApplicationMenu = Menu.setApplicationMenu.bind(Menu);
    Menu.setApplicationMenu = (menu) => {{
      try {{ translateMenu(menu); }} catch {{}}
      return originalSetApplicationMenu(menu);
    }};
  }}
  const menu = Menu.getApplicationMenu();
  if (menu) {{
    translateMenu(menu);
    Menu.setApplicationMenu(menu);
  }}
  return JSON.stringify({{ status: "ok", changed }});
}})()
"#
    )
}

fn menu_label_translations() -> Vec<(&'static str, &'static str)> {
    vec![
        ("File", "文件"),
        ("Edit", "编辑"),
        ("View", "视图"),
        ("Window", "窗口"),
        ("Help", "帮助"),
        ("Undo", "撤销"),
        ("Redo", "重做"),
        ("Cut", "剪切"),
        ("Copy", "复制"),
        ("Paste", "粘贴"),
        ("Delete", "删除"),
        ("Select All", "全选"),
        ("New Window", "新建窗口"),
        ("New Chat", "新建对话"),
        ("Quick Chat", "快速对话"),
        ("Settings…", "设置..."),
        ("Keyboard Shortcuts", "键盘快捷键"),
        ("Open Folder…", "打开文件夹..."),
        ("Toggle Sidebar", "切换边栏"),
        ("Open Terminal", "打开终端"),
        ("Find", "查找"),
        ("Back", "后退"),
        ("Forward", "前进"),
        ("Log Out", "退出登录"),
        ("Reload Window", "重新加载窗口"),
        ("Zoom In", "放大"),
        ("Zoom Out", "缩小"),
        ("Actual Size", "实际大小"),
        ("Toggle Full Screen", "切换全屏"),
        ("Codex Documentation", "Codex 文档"),
        ("What's new", "更新内容"),
        ("Automations", "自动化"),
        ("Local Environments", "本地环境"),
        ("Worktrees", "工作树"),
        ("Skills", "技能"),
        ("Model Context Protocol", "模型上下文协议"),
        ("Troubleshooting", "故障排查"),
        ("Send Feedback", "发送反馈"),
        ("Check for Updates…", "检查更新..."),
        ("Updates Unavailable", "更新不可用"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_menu_localizer_uses_runtime_menu_patch() {
        let script = native_menu_localizer_script();

        assert!(script.contains("Menu.setApplicationMenu"));
        assert!(script.contains("Toggle Sidebar"));
        assert!(script.contains("切换边栏"));
        assert!(!script.contains("app.asar"));
    }
}
