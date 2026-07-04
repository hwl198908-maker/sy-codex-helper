use std::{fs, process::Command, thread, time::Duration};

const LOCALIZATION_RETRIES: usize = 20;
const LOCALIZATION_RETRY_DELAY: Duration = Duration::from_millis(500);

#[derive(serde::Deserialize)]
struct InspectorTarget {
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    #[serde(rename = "type")]
    target_type: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: Option<String>,
}

pub fn spawn_native_menu_localizer(inspector_port: u16) {
    spawn_localizer(
        "native-menu",
        inspector_port,
        target_is_node,
        native_menu_localizer_script,
        false,
    );
}

pub fn spawn_renderer_locale_localizer(debug_port: u16) {
    spawn_localizer(
        "renderer-locale",
        debug_port,
        target_is_page,
        renderer_locale_localizer_script,
        true,
    );
}

fn spawn_localizer(
    name: &'static str,
    port: u16,
    target_filter: fn(&InspectorTarget) -> bool,
    script_builder: fn() -> String,
    register_new_document: bool,
) {
    thread::spawn(move || {
        for attempt in 1..=LOCALIZATION_RETRIES {
            match install_localizer(port, target_filter, script_builder, register_new_document) {
                Ok(()) => {
                    crate::diagnostics::append(
                        "codex_localizer.success",
                        serde_json::json!({ "name": name, "port": port, "attempt": attempt }),
                    );
                    return;
                }
                Err(error) => {
                    crate::diagnostics::append(
                        "codex_localizer.retry",
                        serde_json::json!({
                            "name": name,
                            "port": port,
                            "attempt": attempt,
                            "message": error,
                        }),
                    );
                }
            }
            thread::sleep(LOCALIZATION_RETRY_DELAY);
        }
        crate::diagnostics::append(
            "codex_localizer.failed",
            serde_json::json!({ "name": name, "port": port, "attempts": LOCALIZATION_RETRIES }),
        );
    });
}

fn install_localizer(
    port: u16,
    target_filter: fn(&InspectorTarget) -> bool,
    script_builder: fn() -> String,
    register_new_document: bool,
) -> Result<(), String> {
    let websocket_url = find_websocket_url(port, target_filter)?;
    evaluate_script(&websocket_url, &script_builder(), register_new_document)
}

fn target_is_node(target: &InspectorTarget) -> bool {
    target.target_type == "node"
}

fn target_is_page(target: &InspectorTarget) -> bool {
    target.target_type == "page"
}

fn find_websocket_url(
    port: u16,
    target_filter: fn(&InspectorTarget) -> bool,
) -> Result<String, String> {
    let targets: Vec<InspectorTarget> = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|err| format!("创建 Codex 调试客户端失败: {err}"))?
        .get(format!("http://127.0.0.1:{port}/json/list"))
        .send()
        .map_err(|err| format!("连接 Codex 调试端口失败: {err}"))?
        .json()
        .map_err(|err| format!("读取 Codex 调试目标失败: {err}"))?;

    crate::diagnostics::append(
        "codex_localizer.targets",
        serde_json::json!({
            "port": port,
            "targets": targets.iter().map(|target| {
                serde_json::json!({
                    "type": target.target_type,
                    "title": target.title,
                    "url": target.url,
                    "hasWebSocket": target.web_socket_debugger_url.as_deref().is_some_and(|url| !url.is_empty()),
                })
            }).collect::<Vec<_>>()
        }),
    );

    targets
        .iter()
        .find(|target| {
            target_filter(target)
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
        .ok_or_else(|| "没有找到 Codex 调试目标。".to_string())
}

fn evaluate_script(
    websocket_url: &str,
    script: &str,
    register_new_document: bool,
) -> Result<(), String> {
    let script_path = std::env::temp_dir().join("sy-codex-localizer.js");
    fs::write(&script_path, script).map_err(|err| format!("准备 Codex 汉化脚本失败: {err}"))?;

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            POWERSHELL_CDP_EVALUATE,
        ])
        .env("SY_CODEX_WS_URL", websocket_url)
        .env("SY_CODEX_SCRIPT_PATH", script_path)
        .env(
            "SY_CODEX_REGISTER_DOCUMENT",
            if register_new_document { "1" } else { "0" },
        )
        .output()
        .map_err(|err| format!("启动 Codex 汉化脚本失败: {err}"))?;

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
$scriptPath = $env:SY_CODEX_SCRIPT_PATH
$script = [IO.File]::ReadAllText($scriptPath, [Text.Encoding]::UTF8)
$client = [Net.WebSockets.ClientWebSocket]::new()
$token = [Threading.CancellationToken]::None
$client.ConnectAsync([Uri]$wsUrl, $token).GetAwaiter().GetResult()

function Send-CdpCommand($id, $method, $params) {
  $payload = @{
    id = $id
    method = $method
    params = $params
  } | ConvertTo-Json -Depth 8 -Compress
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
    if ($text -match ('"id"\s*:\s*' + $id)) {
      if ($text -match '"exceptionDetails"' -or $text -match '"error"') { exit 2 }
      return
    }
  }
  exit 3
}

if ($env:SY_CODEX_REGISTER_DOCUMENT -eq '1') {
  Send-CdpCommand 1 'Page.addScriptToEvaluateOnNewDocument' @{ source = $script }
}
Send-CdpCommand 2 'Runtime.evaluate' @{
  expression = $script
  awaitPromise = $true
  returnByValue = $true
  allowUnsafeEvalBlockedByCSP = $true
}
exit 0
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

pub fn renderer_locale_localizer_script() -> String {
    r#"
(() => {
  const locale = "zh-CN";
  if (window.__syCodexForceChineseLocaleInstalled === "1") {
    return JSON.stringify({ status: "already-installed", locale });
  }
  window.__syCodexForceChineseLocaleInstalled = "1";
  const languages = [locale, "zh", "en-US", "en"];

  const defineNavigatorGetter = (name, value) => {
    try {
      Object.defineProperty(Navigator.prototype, name, {
        configurable: true,
        get: () => value,
      });
    } catch {
      try {
        Object.defineProperty(navigator, name, {
          configurable: true,
          get: () => value,
        });
      } catch {}
    }
  };

  defineNavigatorGetter("language", locale);
  defineNavigatorGetter("languages", languages);

  const patchI18nConfig = (dynamicConfig) => {
    if (!dynamicConfig || typeof dynamicConfig !== "object") return dynamicConfig;
    const value = dynamicConfig.value && typeof dynamicConfig.value === "object" ? dynamicConfig.value : {};
    try {
      dynamicConfig.value = {
        ...value,
        enable_i18n: true,
        locale_source: "SYSTEM",
      };
    } catch {}
    if (typeof dynamicConfig.get === "function" && !dynamicConfig.__syCodexForceChineseLocaleGetPatched) {
      const originalGet = dynamicConfig.get.bind(dynamicConfig);
      dynamicConfig.get = (key, fallback) => {
        if (key === "enable_i18n") return true;
        if (key === "locale_source") return "SYSTEM";
        return originalGet(key, fallback);
      };
      dynamicConfig.__syCodexForceChineseLocaleGetPatched = true;
    }
    return dynamicConfig;
  };

  const statsigClients = () => {
    const root = window.__STATSIG__ || globalThis.__STATSIG__;
    if (!root || typeof root !== "object") return [];
    const clients = [root.firstInstance, typeof root.instance === "function" ? root.instance() : null];
    if (root.instances && typeof root.instances === "object") clients.push(...Object.values(root.instances));
    return clients.filter((client, index, array) => client && typeof client === "object" && array.indexOf(client) === index);
  };

  const patchStatsigClient = (client) => {
    if (!client || typeof client !== "object" || typeof client.getDynamicConfig !== "function") return;
    if (!client.__syCodexForceChineseLocalePatched) {
      const originalGetDynamicConfig = client.getDynamicConfig.bind(client);
      client.getDynamicConfig = (name, options) => {
        const result = originalGetDynamicConfig(name, options);
        return name === "72216192" ? patchI18nConfig(result) : result;
      };
      client.__syCodexForceChineseLocalePatched = true;
    }
    try {
      patchI18nConfig(client.getDynamicConfig("72216192", { disableExposureLog: true }));
    } catch {}
  };

  const patchStatsigRoot = (root) => {
    if (!root || typeof root !== "object" || root.__syCodexForceChineseLocaleRootPatched) return;
    root.__syCodexForceChineseLocaleRootPatched = true;
    ["firstInstance", "instance"].forEach((key) => {
      let current;
      try {
        current = root[key];
      } catch {
        return;
      }
      patchStatsigClient(typeof current === "function" && key === "instance" ? current.call(root) : current);
      try {
        Object.defineProperty(root, key, {
          configurable: true,
          get: () => current,
          set: (next) => {
            current = next;
            patchStatsigClient(typeof next === "function" && key === "instance" ? next.call(root) : next);
          },
        });
      } catch {}
    });
  };

  const installStatsigRootSetter = () => {
    const descriptor = Object.getOwnPropertyDescriptor(window, "__STATSIG__");
    if (descriptor && descriptor.configurable === false) return;
    let currentRoot = window.__STATSIG__;
    patchStatsigRoot(currentRoot);
    try {
      Object.defineProperty(window, "__STATSIG__", {
        configurable: true,
        get: () => currentRoot,
        set: (next) => {
          currentRoot = next;
          patchStatsigRoot(next);
          statsigClients().forEach(patchStatsigClient);
        },
      });
    } catch {}
  };

  const patchStatsigI18nConfig = () => {
    installStatsigRootSetter();
    const root = window.__STATSIG__ || globalThis.__STATSIG__;
    patchStatsigRoot(root);
    statsigClients().forEach(patchStatsigClient);
  };

  patchStatsigI18nConfig();
  const startedAt = Date.now();
  const timer = window.setInterval(() => {
    patchStatsigI18nConfig();
    if (Date.now() - startedAt > 5000) window.clearInterval(timer);
  }, 50);

  const navigationTranslations = new Map([
    ["Chats", "对话"],
    ["Projects", "项目"],
    ["Settings", "设置"],
    ["Automations", "自动化"],
    ["Skills", "技能"],
    ["New Chat", "新建对话"],
    ["Search chats", "搜索对话"],
    ["Search Chats", "搜索对话"],
    ["Archived conversations", "已归档对话"],
    ["Local Environments", "本地环境"],
    ["Worktrees", "工作树"],
    ["Model Context Protocol", "模型上下文协议"],
    ["Troubleshooting", "故障排查"],
    ["What's new", "更新内容"],
  ]);

  const translateTextNode = (node) => {
    const text = node.nodeValue;
    if (!text) return 0;
    const trimmed = text.trim();
    const translated = navigationTranslations.get(trimmed);
    if (!translated || trimmed === translated) return 0;
    node.nodeValue = text.replace(trimmed, translated);
    return 1;
  };

  const translateAttributes = (element) => {
    let changed = 0;
    for (const attr of ["aria-label", "title", "placeholder", "data-app-action-sidebar-section-heading"]) {
      const value = element.getAttribute?.(attr);
      const translated = value ? navigationTranslations.get(value.trim()) : null;
      if (translated && value !== translated) {
        element.setAttribute(attr, translated);
        changed += 1;
      }
    }
    return changed;
  };

  const translateNavigationText = (root = document.body || document.documentElement) => {
    if (!root) return 0;
    let changed = 0;
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: (node) => {
        const parent = node.parentElement;
        if (!parent || parent.closest("script,style,textarea,input,[contenteditable=true]")) {
          return NodeFilter.FILTER_REJECT;
        }
        return navigationTranslations.has((node.nodeValue || "").trim())
          ? NodeFilter.FILTER_ACCEPT
          : NodeFilter.FILTER_REJECT;
      },
    });
    const nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    nodes.forEach((node) => { changed += translateTextNode(node); });
    root.querySelectorAll?.("[aria-label], [title], [placeholder], [data-app-action-sidebar-section-heading]")
      .forEach((element) => { changed += translateAttributes(element); });
    window.__syCodexNavigationTranslationCount = (window.__syCodexNavigationTranslationCount || 0) + changed;
    return changed;
  };

  const scheduleNavigationTranslation = () => {
    window.clearTimeout(window.__syCodexNavigationTranslationTimer);
    window.__syCodexNavigationTranslationTimer = window.setTimeout(() => translateNavigationText(), 80);
  };

  translateNavigationText();
  if (!window.__syCodexNavigationObserver) {
    window.__syCodexNavigationObserver = new MutationObserver(scheduleNavigationTranslation);
    window.__syCodexNavigationObserver.observe(document.documentElement, {
      childList: true,
      subtree: true,
      characterData: true,
      attributes: true,
      attributeFilter: ["aria-label", "title", "placeholder", "data-app-action-sidebar-section-heading"],
    });
  }

  return JSON.stringify({ status: "ok", locale, navigationFallback: true });
})()
"#
    .to_string()
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

    #[test]
    fn renderer_locale_localizer_forces_codex_i18n_config() {
        let script = renderer_locale_localizer_script();

        assert!(script.contains("zh-CN"));
        assert!(script.contains("enable_i18n"));
        assert!(script.contains("locale_source"));
        assert!(script.contains("72216192"));
        assert!(script.contains("navigationFallback"));
        assert!(script.contains("MutationObserver"));
        assert!(script.contains("data-app-action-sidebar-section-heading"));
        assert!(!script.contains("app.asar"));
    }

    #[test]
    fn cdp_evaluator_registers_renderer_script_for_new_documents() {
        assert!(POWERSHELL_CDP_EVALUATE.contains("Page.addScriptToEvaluateOnNewDocument"));
        assert!(POWERSHELL_CDP_EVALUATE.contains("Runtime.evaluate"));
        assert!(POWERSHELL_CDP_EVALUATE.contains("SY_CODEX_REGISTER_DOCUMENT"));
    }
}
