use std::{net::TcpStream, thread, time::Duration};

use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

const LOCALIZATION_RETRIES: usize = 30;
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
    let (mut socket, _) = tungstenite::connect(websocket_url)
        .map_err(|err| format!("连接 Codex 调试 WebSocket 失败: {err}"))?;

    if register_new_document {
        send_cdp_command(
            &mut socket,
            1,
            "Page.addScriptToEvaluateOnNewDocument",
            serde_json::json!({ "source": script }),
        )?;
    }

    send_cdp_command(
        &mut socket,
        2,
        "Runtime.evaluate",
        serde_json::json!({
            "expression": script,
            "awaitPromise": true,
            "returnByValue": true,
            "allowUnsafeEvalBlockedByCSP": true,
        }),
    )
}

fn send_cdp_command(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    id: u64,
    method: &str,
    params: serde_json::Value,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "id": id,
        "method": method,
        "params": params,
    })
    .to_string();
    socket
        .send(Message::Text(payload.into()))
        .map_err(|err| format!("发送 Codex 汉化命令失败: {err}"))?;

    for _ in 0..50 {
        let message = socket
            .read()
            .map_err(|err| format!("读取 Codex 汉化响应失败: {err}"))?;
        let Message::Text(text) = message else {
            continue;
        };
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| format!("解析 Codex 汉化响应失败: {err}"))?;
        if value.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
            continue;
        }
        if let Some(error) = value.get("error") {
            return Err(format!("Codex 汉化命令返回错误: {error}"));
        }
        if value
            .get("result")
            .and_then(|result| result.get("exceptionDetails"))
            .is_some()
        {
            return Err("Codex 汉化脚本执行异常。".to_string());
        }
        return Ok(());
    }

    Err("等待 Codex 汉化响应超时。".to_string())
}

pub fn native_menu_localizer_script() -> String {
    let translations = serde_json::to_string(&menu_label_translations()).unwrap_or_default();
    format!(
        r#"
(() => {{
  const translations = new Map({translations});
  const electron = process.mainModule?.require?.("electron") || require?.("electron");
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
    let translations = serde_json::to_string(&renderer_text_translations()).unwrap_or_default();
    format!(
        r#"
(() => {{
  const locale = "zh-CN";
  const alreadyInstalled = window.__syCodexForceChineseLocaleInstalled === "1";
  window.__syCodexForceChineseLocaleInstalled = "1";
  const languages = [locale, "zh", "en-US", "en"];

  const defineNavigatorGetter = (name, value) => {{
    try {{
      Object.defineProperty(Navigator.prototype, name, {{
        configurable: true,
        get: () => value,
      }});
    }} catch {{
      try {{
        Object.defineProperty(navigator, name, {{
          configurable: true,
          get: () => value,
        }});
      }} catch {{}}
    }}
  }};

  defineNavigatorGetter("language", locale);
  defineNavigatorGetter("languages", languages);

  const patchI18nConfig = (dynamicConfig) => {{
    if (!dynamicConfig || typeof dynamicConfig !== "object") return dynamicConfig;
    const value = dynamicConfig.value && typeof dynamicConfig.value === "object" ? dynamicConfig.value : {{}};
    try {{
      dynamicConfig.value = {{
        ...value,
        enable_i18n: true,
        locale_source: "SYSTEM",
      }};
    }} catch {{}}
    if (typeof dynamicConfig.get === "function" && !dynamicConfig.__syCodexForceChineseLocaleGetPatched) {{
      const originalGet = dynamicConfig.get.bind(dynamicConfig);
      dynamicConfig.get = (key, fallback) => {{
        if (key === "enable_i18n") return true;
        if (key === "locale_source") return "SYSTEM";
        return originalGet(key, fallback);
      }};
      dynamicConfig.__syCodexForceChineseLocaleGetPatched = true;
    }}
    return dynamicConfig;
  }};

  const statsigClients = () => {{
    const root = window.__STATSIG__ || globalThis.__STATSIG__;
    if (!root || typeof root !== "object") return [];
    const clients = [root.firstInstance, typeof root.instance === "function" ? root.instance() : null];
    if (root.instances && typeof root.instances === "object") clients.push(...Object.values(root.instances));
    return clients.filter((client, index, array) => client && typeof client === "object" && array.indexOf(client) === index);
  }};

  const patchStatsigClient = (client) => {{
    if (!client || typeof client !== "object" || typeof client.getDynamicConfig !== "function") return;
    if (!client.__syCodexForceChineseLocalePatched) {{
      const originalGetDynamicConfig = client.getDynamicConfig.bind(client);
      client.getDynamicConfig = (name, options) => {{
        const result = originalGetDynamicConfig(name, options);
        return name === "72216192" ? patchI18nConfig(result) : result;
      }};
      client.__syCodexForceChineseLocalePatched = true;
    }}
    try {{
      patchI18nConfig(client.getDynamicConfig("72216192", {{ disableExposureLog: true }}));
    }} catch {{}}
  }};

  const patchStatsigRoot = (root) => {{
    if (!root || typeof root !== "object" || root.__syCodexForceChineseLocaleRootPatched) return;
    root.__syCodexForceChineseLocaleRootPatched = true;
    ["firstInstance", "instance"].forEach((key) => {{
      let current;
      try {{
        current = root[key];
      }} catch {{
        return;
      }}
      patchStatsigClient(typeof current === "function" && key === "instance" ? current.call(root) : current);
      try {{
        Object.defineProperty(root, key, {{
          configurable: true,
          get: () => current,
          set: (next) => {{
            current = next;
            patchStatsigClient(typeof next === "function" && key === "instance" ? next.call(root) : next);
          }},
        }});
      }} catch {{}}
    }});
  }};

  const installStatsigRootSetter = () => {{
    const descriptor = Object.getOwnPropertyDescriptor(window, "__STATSIG__");
    if (descriptor && descriptor.configurable === false) return;
    let currentRoot = window.__STATSIG__;
    patchStatsigRoot(currentRoot);
    try {{
      Object.defineProperty(window, "__STATSIG__", {{
        configurable: true,
        get: () => currentRoot,
        set: (next) => {{
          currentRoot = next;
          patchStatsigRoot(next);
          statsigClients().forEach(patchStatsigClient);
        }},
      }});
    }} catch {{}}
  }};

  const patchStatsigI18nConfig = () => {{
    installStatsigRootSetter();
    const root = window.__STATSIG__ || globalThis.__STATSIG__;
    patchStatsigRoot(root);
    statsigClients().forEach(patchStatsigClient);
  }};

  patchStatsigI18nConfig();
  const startedAt = Date.now();
  const timer = window.setInterval(() => {{
    patchStatsigI18nConfig();
    if (Date.now() - startedAt > 8000) window.clearInterval(timer);
  }}, 50);

  const translations = new Map({translations});

  const translateTextNode = (node) => {{
    const text = node.nodeValue;
    if (!text) return 0;
    const trimmed = text.trim();
    const translated = translations.get(trimmed);
    if (!translated || trimmed === translated) return 0;
    node.nodeValue = text.replace(trimmed, translated);
    return 1;
  }};

  const translateAttributes = (element) => {{
    let changed = 0;
    for (const attr of ["aria-label", "title", "placeholder", "data-app-action-sidebar-section-heading"]) {{
      const value = element.getAttribute?.(attr);
      const translated = value ? translations.get(value.trim()) : null;
      if (translated && value !== translated) {{
        element.setAttribute(attr, translated);
        changed += 1;
      }}
    }}
    return changed;
  }};

  const translateNavigationText = (root = document.body || document.documentElement) => {{
    if (!root) return 0;
    let changed = 0;
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {{
      acceptNode: (node) => {{
        const parent = node.parentElement;
        if (!parent || parent.closest("script,style,textarea,input,[contenteditable=true]")) {{
          return NodeFilter.FILTER_REJECT;
        }}
        return translations.has((node.nodeValue || "").trim())
          ? NodeFilter.FILTER_ACCEPT
          : NodeFilter.FILTER_REJECT;
      }},
    }});
    const nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    nodes.forEach((node) => {{ changed += translateTextNode(node); }});
    root.querySelectorAll?.("[aria-label], [title], [placeholder], [data-app-action-sidebar-section-heading]")
      .forEach((element) => {{ changed += translateAttributes(element); }});
    window.__syCodexNavigationTranslationCount = (window.__syCodexNavigationTranslationCount || 0) + changed;
    return changed;
  }};

  const runNavigationTranslation = () => {{
    translateNavigationText();
    console.info("[SY Codex] navigation translations applied", window.__syCodexNavigationTranslationCount || 0);
  }};

  const scheduleNavigationTranslation = () => {{
    window.clearTimeout(window.__syCodexNavigationTranslationTimer);
    window.__syCodexNavigationTranslationTimer = window.setTimeout(runNavigationTranslation, 80);
  }};

  const installNavigationObserver = () => {{
    if (window.__syCodexNavigationObserver || !document.documentElement) return;
    window.__syCodexNavigationObserver = new MutationObserver(scheduleNavigationTranslation);
    window.__syCodexNavigationObserver.observe(document.documentElement, {{
      childList: true,
      subtree: true,
      characterData: true,
      attributes: true,
      attributeFilter: ["aria-label", "title", "placeholder", "data-app-action-sidebar-section-heading"],
    }});
  }};

  const activateNavigationTranslation = () => {{
    installNavigationObserver();
    runNavigationTranslation();
  }};

  if (document.readyState === "loading") {{
    document.addEventListener("DOMContentLoaded", activateNavigationTranslation, {{ once: true }});
  }} else {{
    activateNavigationTranslation();
  }}

  return JSON.stringify({{ status: alreadyInstalled ? "reapplied" : "ok", locale, navigationFallback: true }});
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
        ("Settings...", "设置..."),
        ("Settings…", "设置..."),
        ("Keyboard Shortcuts", "键盘快捷键"),
        ("Open Folder...", "打开文件夹..."),
        ("Open Folder…", "打开文件夹..."),
        ("Toggle Sidebar", "切换侧边栏"),
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
        ("Check for Updates...", "检查更新..."),
        ("Check for Updates…", "检查更新..."),
        ("Updates Unavailable", "更新不可用"),
        ("Hide Codex", "隐藏 Codex"),
        ("Hide Others", "隐藏其他"),
        ("Show All", "全部显示"),
        ("Quit Codex", "退出 Codex"),
        ("Services", "服务"),
        ("Minimize", "最小化"),
        ("Close Window", "关闭窗口"),
        ("Bring All to Front", "全部置于前台"),
    ]
}

fn renderer_text_translations() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Chats", "对话"),
        ("Projects", "项目"),
        ("Settings", "设置"),
        ("Automations", "自动化"),
        ("Skills", "技能"),
        ("Plugins", "插件"),
        ("Plugin", "插件"),
        ("New chat", "新建对话"),
        ("New Chat", "新建对话"),
        ("Search", "搜索"),
        ("Search chats", "搜索对话"),
        ("Search Chats", "搜索对话"),
        ("Search plugins", "搜索插件"),
        ("Archived conversations", "已归档对话"),
        ("Local Environments", "本地环境"),
        ("Worktrees", "工作树"),
        ("Model Context Protocol", "模型上下文协议"),
        ("Troubleshooting", "故障排查"),
        ("What's new", "更新内容"),
        ("What should we get done?", "今天要完成什么？"),
        ("Do anything", "输入你的任务"),
        ("Install", "安装"),
        ("Back to app", "返回应用"),
        ("General", "通用"),
        ("Appearance", "外观"),
        ("Configuration", "配置"),
        ("Personalization", "个性化"),
        ("Keyboard shortcuts", "键盘快捷键"),
        ("Integrations", "集成"),
        ("MCP servers", "MCP 服务器"),
        ("Browser", "浏览器"),
        ("Computer use", "电脑使用"),
        ("Coding", "编程"),
        ("Hooks", "钩子"),
        ("Git", "Git"),
        ("Environments", "环境"),
        ("Archived", "已归档"),
        ("Archived chats", "已归档对话"),
        ("Chat Settings", "聊天设置"),
        ("Work mode", "工作模式"),
        ("Choose how much technical detail Codex shows", "选择 Codex 显示多少技术细节"),
        ("For coding", "用于编程"),
        ("More technical responses and control", "更多技术细节和控制"),
        ("For everyday work", "用于日常工作"),
        ("Same power, less technical detail", "同样能力，更少技术细节"),
        ("Permissions", "权限"),
        ("Default permissions", "默认权限"),
        ("By default, Codex can read and edit files in its workspace. It can ask for additional access when needed", "默认情况下，Codex 可以读取和编辑工作区文件，需要时会请求额外权限"),
        ("Full access", "完全访问"),
        ("When Codex runs with full access, it can edit any file on your computer and run commands with network, without your approval. This significantly increases the risk of data loss, leaks, or unexpected behavior.", "开启完全访问后，Codex 可以编辑电脑上的任意文件并运行联网命令，且不再需要你的确认。这会明显增加数据丢失、泄露或异常行为的风险。"),
        ("Learn more", "了解更多"),
        ("Default file open destination", "默认文件打开位置"),
        ("Where files and folders open by default", "文件和文件夹默认打开的位置"),
        ("No targets found", "未找到目标"),
        ("Integrated terminal shell", "集成终端 Shell"),
        ("Choose which shell opens in the integrated terminal.", "选择集成终端默认打开的 Shell。"),
        ("By OpenAI", "OpenAI 提供"),
        ("By your workspace", "工作区提供"),
        ("Personal", "个人"),
        ("Creativity", "创意"),
        ("Data & Analytics", "数据分析"),
        ("Developer Tools", "开发工具"),
        ("Work with Codex across your favorite tools", "让 Codex 连接你常用的工具"),
        ("No chats", "暂无对话"),
        ("Account", "账号"),
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
        assert!(script.contains("切换侧边栏"));
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
        assert!(script.contains("Plugins"));
        assert!(script.contains("What should we get done?"));
        assert!(script.contains("Do anything"));
        assert!(script.contains("Back to app"));
        assert!(script.contains("Work mode"));
        assert!(script.contains("Default permissions"));
        assert!(script.contains("默认权限"));
        assert!(!script.contains("app.asar"));
    }

    #[test]
    fn renderer_localizer_runs_again_after_the_document_is_ready() {
        let script = renderer_locale_localizer_script();

        assert!(script.contains("DOMContentLoaded"));
        assert!(script.contains("runNavigationTranslation"));
    }

    #[test]
    fn cdp_evaluator_uses_rust_websocket_instead_of_powershell() {
        assert!(
            renderer_locale_localizer_script().contains("Page.addScriptToEvaluateOnNewDocument")
                == false
        );
        assert!(
            !std::any::type_name::<WebSocket<MaybeTlsStream<TcpStream>>>().contains("PowerShell")
        );
    }
}
