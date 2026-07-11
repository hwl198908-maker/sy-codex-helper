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
                Ok(report) => {
                    crate::diagnostics::append(
                        "codex_localizer.success",
                        serde_json::json!({
                            "name": name,
                            "port": port,
                            "attempt": attempt,
                            "report": localization_report(&report),
                        }),
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
) -> Result<serde_json::Value, String> {
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
) -> Result<serde_json::Value, String> {
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
) -> Result<serde_json::Value, String> {
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
        return Ok(value);
    }

    Err("等待 Codex 汉化响应超时。".to_string())
}

fn localization_report(response: &serde_json::Value) -> serde_json::Value {
    response
        .pointer("/result/result/value")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| serde_json::from_str(value).ok())
        .unwrap_or_else(|| serde_json::json!({ "status": "no-report" }))
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
    const changed = translateNavigationText();
    console.info("[SY Codex] navigation translations applied", window.__syCodexNavigationTranslationCount || 0);
    return changed;
  }};

  const collectUntranslatedSystemUi = () => {{
    if (!document.body) return [];
    const visible = (element) => {{
      if (!element || element.closest("textarea,input,[contenteditable=true],pre,code")) return false;
      const style = window.getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
    }};
    const values = new Set();
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    while (walker.nextNode()) {{
      const node = walker.currentNode;
      const text = (node.nodeValue || "").replace(/\s+/g, " ").trim();
      if (text && text.length <= 120 && /[A-Za-z]/.test(text) && visible(node.parentElement)) values.add(text);
    }}
    document.querySelectorAll("[aria-label],[title],[placeholder]").forEach((element) => {{
      if (!visible(element)) return;
      for (const attr of ["aria-label", "title", "placeholder"]) {{
        const text = (element.getAttribute(attr) || "").trim();
        if (text && text.length <= 120 && /[A-Za-z]/.test(text)) values.add(text);
      }}
    }});
    return [...values].slice(0, 40);
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

  const translatedCount = runNavigationTranslation();
  return JSON.stringify({{
    status: alreadyInstalled ? "reapplied" : "ok",
    locale,
    navigationFallback: true,
    translatedCount,
    untranslatedSystemUi: collectUntranslatedSystemUi(),
  }});
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
        ("New Task", "新建任务"),
        ("New Projectless Task", "新建无项目任务"),
        ("New Chat", "新建对话"),
        ("Quick Chat", "快速对话"),
        ("Settings...", "设置..."),
        ("Settings…", "设置..."),
        ("Keyboard Shortcuts", "键盘快捷键"),
        ("Open Folder...", "打开文件夹..."),
        ("Open Folder…", "打开文件夹..."),
        ("Toggle Sidebar", "切换侧边栏"),
        ("Toggle Bottom Panel", "切换底部面板"),
        ("Toggle Pinned Summary", "切换置顶摘要"),
        ("Toggle File Tree", "切换文件树"),
        ("Toggle Side Panel", "切换侧边面板"),
        ("Open Terminal", "打开终端"),
        ("Open Browser Tab", "打开浏览器标签页"),
        ("Focus Browser Address Bar", "聚焦浏览器地址栏"),
        ("Reload Browser Page", "重新加载浏览器页面"),
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
        ("Documentation", "文档"),
        ("What's New", "更新内容"),
        ("System Status", "系统状态"),
        ("Start Performance Trace", "开始性能跟踪"),
        ("About ChatGPT", "关于 ChatGPT"),
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
        ("File", "文件"),
        ("Edit", "编辑"),
        ("View", "视图"),
        ("Window", "窗口"),
        ("Help", "帮助"),
        ("Chats", "对话"),
        ("Projects", "项目"),
        ("Settings", "设置"),
        ("Automations", "自动化"),
        ("Skills", "技能"),
        ("Plugins", "插件"),
        ("Plugin", "插件"),
        ("New chat", "新建对话"),
        ("New Chat", "新建对话"),
        ("New Task", "新建任务"),
        ("New Projectless Task", "新建无项目任务"),
        ("Documentation", "文档"),
        ("Keyboard shortcuts", "键盘快捷键"),
        ("What's New", "更新内容"),
        ("System Status", "系统状态"),
        ("Send feedback", "发送反馈"),
        ("Start Performance Trace", "开始性能跟踪"),
        ("About ChatGPT", "关于 ChatGPT"),
        ("Toggle Bottom Panel", "切换底部面板"),
        ("Toggle Pinned Summary", "切换置顶摘要"),
        ("Toggle File Tree", "切换文件树"),
        ("Toggle Side Panel", "切换侧边面板"),
        ("Open Browser Tab", "打开浏览器标签页"),
        ("Focus Browser Address Bar", "聚焦浏览器地址栏"),
        ("Reload Browser Page", "重新加载浏览器页面"),
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
        ("Unpin task", "取消置顶任务"),
        ("Rename task", "重命名任务"),
        ("Archive task", "归档任务"),
        ("Mark as unread", "标记为未读"),
        ("Open in Explorer", "在资源管理器中打开"),
        ("Copy working directory", "复制工作目录"),
        ("Copy session ID", "复制会话 ID"),
        ("Copy deeplink", "复制深层链接"),
        ("Continue in new task", "在新任务中继续"),
        ("Language", "语言"),
        ("Language for the app UI", "应用界面语言"),
        ("Auto detect", "自动检测"),
        ("Bottom panel", "底部面板"),
        ("Show the bottom panel control in the app header", "在应用顶部显示底部面板控制按钮"),
        ("Import work from other AI apps", "从其他 AI 应用导入工作内容"),
        ("Bring over your setup, projects, and recent chats", "导入你的设置、项目和最近对话"),
        ("No data detected", "未检测到数据"),
        ("Open source licenses", "开源许可证"),
        ("Third-party notices for bundled dependencies", "随附依赖项的第三方声明"),
        ("View", "查看"),
        ("Composer", "输入区"),
        ("Show context window usage", "显示上下文窗口用量"),
        ("Send shortcut", "发送快捷键"),
        ("Choose when Enter sends a prompt or inserts a new line", "选择 Enter 是发送提示词还是插入换行"),
        ("Follow-up behavior", "后续操作方式"),
        ("Queue follow-ups while ChatGPT runs or steer the current run. Press Ctrl+Enter to do the opposite for one message", "ChatGPT 运行时可将后续消息排队，或引导当前任务。按 Ctrl+Enter 可对单条消息执行相反操作"),
        ("Queue", "排队"),
        ("Steer", "引导"),
        ("Notifications", "通知"),
        ("Turn completion notifications", "任务完成通知"),
        ("Set when ChatGPT alerts you that it's finished", "设置 ChatGPT 完成任务时的提醒方式"),
        ("Only when unfocused", "仅在未聚焦时"),
        ("Enable permission notifications", "启用权限通知"),
        ("Show alerts when notification permissions are required", "需要通知权限时显示提醒"),
        ("Enable question notifications", "启用提问通知"),
        ("Show alerts when input is needed to continue", "需要输入以继续时显示提醒"),
        ("Light theme", "浅色主题"),
        ("Dark theme", "深色主题"),
        ("Import", "导入"),
        ("Copy theme", "复制主题"),
        ("Accent", "强调色"),
        ("Background", "背景"),
        ("Foreground", "前景色"),
        ("UI font", "界面字体"),
        ("Contrast", "对比度"),
        ("Preferences", "偏好设置"),
        ("Use pointer cursors", "使用指针光标"),
        ("Change the cursor to a pointer when hovering over interactive elements", "悬停在可交互元素上时将光标改为指针"),
        ("Reduce motion", "减少动画效果"),
        ("Reduce animations or match your system", "减少动画效果或跟随系统设置"),
        ("System", "系统"),
        ("On", "开启"),
        ("Off", "关闭"),
        ("UI font size", "界面字体大小"),
        ("Adjust the base size used for the ChatGPT UI", "调整 ChatGPT 界面的基础字体大小"),
        ("Diff markers", "差异标记"),
        ("Show changes using colors or +/- markers", "使用颜色或 +/- 标记显示更改"),
        ("Color", "颜色"),
        ("Configure approval policy and sandbox settings", "配置审批策略和沙盒设置"),
        ("Custom config.toml settings", "自定义 config.toml 设置"),
        ("Open config.toml", "打开 config.toml"),
        ("Approval policy", "审批策略"),
        ("Choose when ChatGPT asks for approval", "选择 ChatGPT 何时请求审批"),
        ("On request", "按需请求"),
        ("Sandbox settings", "沙盒设置"),
        ("Choose how much ChatGPT can do when running commands", "选择 ChatGPT 运行命令时可执行的操作范围"),
        ("Read only", "只读"),
        ("Custom instructions", "自定义指令"),
        ("Give ChatGPT extra instructions and context for all tasks on this host.", "为本机上的所有任务提供额外指令和上下文。"),
        ("Save", "保存"),
        ("Memory (experimental)", "记忆（实验性）"),
        ("Configure how ChatGPT collects, retains, and consolidates memories.", "配置 ChatGPT 如何收集、保留和整合记忆。"),
        ("Enable memories", "启用记忆"),
        ("Generate new memories from tasks and bring them into new tasks", "从任务中生成新记忆，并带入后续任务"),
        ("Allow memory generation from tool-assisted tasks", "允许从工具辅助任务中生成记忆"),
        ("Generate memories from tasks that used MCP tools or web search", "从使用 MCP 工具或网页搜索的任务中生成记忆"),
        ("Reset memories", "重置记忆"),
        ("Delete all ChatGPT memories", "删除所有 ChatGPT 记忆"),
        ("Reset", "重置"),
        ("Browsing data", "浏览数据"),
        ("Clear browsing history, site data, cache, and download history from the in-app browser", "清除应用内浏览器的历史记录、网站数据、缓存和下载记录"),
        ("Clear all browsing data", "清除所有浏览数据"),
        ("Show individual browsing data options", "显示单项浏览数据选项"),
        ("Annotation screenshots", "标注截图"),
        ("Screenshots help ChatGPT better understand and address comments, but increase plan usage", "截图可帮助 ChatGPT 更好理解和处理评论，但会增加套餐用量"),
        ("Always include", "始终包含"),
        ("Autofill and passwords", "自动填充和密码"),
        ("Password manager", "密码管理器"),
        ("Add, delete, and edit saved passwords", "添加、删除和编辑已保存的密码"),
        ("Manage", "管理"),
        ("Contact info", "联系信息"),
        ("Add, delete, and edit saved addresses, phone numbers, and email addresses", "添加、删除和编辑已保存的地址、电话和邮箱"),
        ("Site settings", "网站设置"),
        ("Control camera and microphone permissions in the built-in browser", "控制内置浏览器中的摄像头和麦克风权限"),
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
    fn translations_cover_the_reviewed_menus_and_settings_pages() {
        let menu = menu_label_translations();
        let renderer = renderer_text_translations();

        for source in ["New Task", "Toggle Bottom Panel", "Documentation"] {
            assert!(menu.iter().any(|(key, _)| *key == source), "missing menu: {source}");
        }
        for source in [
            "Unpin task",
            "Language",
            "Auto detect",
            "Browsing data",
            "Light theme",
            "Approval policy",
            "Custom instructions",
        ] {
            assert!(
                renderer.iter().any(|(key, _)| *key == source),
                "missing renderer text: {source}"
            );
        }
    }

    #[test]
    fn renderer_localizer_reports_untranslated_system_ui() {
        let script = renderer_locale_localizer_script();

        assert!(script.contains("untranslatedSystemUi"));
        assert!(script.contains("collectUntranslatedSystemUi"));
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
