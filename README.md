# Codex Manager

Codex Manager 是一个面向 Windows 独立分发的 Tauri 桌面应用，用来帮助用户管理 Codex 安装源和 OpenAI 兼容 API 配置。

当前范围：

- Codex：已启用默认镜像线路、安装包下载、安装程序打开和 API 配置流程。
- OpenClaw：入口已预留，当前版本显示为即将支持。
- Provider：支持 OpenAI 兼容服务地址、API Key、模型读取和本机配置写入流程。

## 安装依赖

先安装 Node.js、npm、Rust 和 Tauri Windows 打包所需的系统依赖。首次进入项目后执行：

```powershell
npm install
```

如果当前终端找不到 Rust 或 Cargo，重新打开终端，或刷新 Rust 环境变量后再执行构建命令。

## 开发模式

启动前端和 Tauri 开发窗口：

```powershell
npm run tauri dev
```

只运行前端开发服务器：

```powershell
npm run dev
```

## 构建 Windows 包

先执行前端构建：

```powershell
npm run build
```

再构建 Tauri Windows 安装包：

```powershell
npm run tauri build
```

当前配置生成 NSIS 安装包。构建机仍需要满足 Tauri/Windows 打包工具链要求；如果缺少外部打包前置条件，应先安装对应工具，不要绕过安全或签名流程。

## 镜像清单格式

安装页默认填入 `src/lib/defaults.ts` 中的 `DEFAULT_MIRROR_BASE_URL`。当前默认值指向本机已有安装包目录：

```text
D:\codex++codex安装包(2)\CodexWindows
```

如果要改成网络镜像，替换这个常量即可。

网络镜像接口应在 `<Base URL>/manifest.json` 返回 JSON，字段使用 camelCase。工具包条目至少包含 `toolId`、`packageUrl`、`checksumSha256`：

```json
{
  "version": 1,
  "tools": [
    {
      "toolId": "codex",
      "platform": "windows-x64",
      "version": "1.2.3",
      "packageUrl": "https://mirror.example.invalid/codex/windows/x64/codex-1.2.3.exe",
      "checksumSha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }
  ]
}
```

README 中的网络地址只是示例，不代表真实镜像服务。安装页会读取清单中的 `packageUrl`，下载 Windows 安装包，并打开安装程序。若 Base URL 是本地目录，应用会直接查找目录中的 Windows 安装包并打开。

## 使用流程

1. 选择工具：当前启用 Codex，OpenClaw 显示为即将支持。
2. 安装 Codex：使用默认镜像线路，或手动替换镜像 Base URL 后下载安装包。
3. 配置 API：填写 Base URL 和 API Key，获取模型列表并保存到本机 Codex 配置。
4. 完成：打开 Codex，确认可以正常运行。

## Windows 安全说明

Codex Manager 不绕过 Windows 安全机制，不关闭系统防护，也不伪造受信任发布者状态。项目按独立分发设计，后续可以接入正式代码签名、可信发布和时间戳服务；当前仓库不包含真实签名证书，也不声称安装包已经完成签名。
