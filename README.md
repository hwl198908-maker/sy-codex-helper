# SY Codex 助手

面向中国用户的 Codex 入门安装与配置工具。

SY Codex 助手把 Codex 桌面 App 的安装、代理 API 配置、模型选择、中文增强和版本更新整理成一个简单向导，帮助没有技术基础的用户更快完成 Codex 初始化。

> 本项目不是 OpenAI 官方项目，不包含 OpenAI 官方安装包，也不修改官方 Codex 安装包。它是一个独立的安装、配置和中文增强辅助工具。

## 解决什么问题

很多中国新手用户第一次使用 Codex 会卡在这些地方：

- Microsoft Store 打不开或下载失败。
- 不知道 Codex 桌面 App 应该怎么安装。
- 不会填写 Base URL、API Key、模型名称和协议。
- 不理解 `config.toml`、`auth.json` 等本地配置文件。
- 英文界面和设置项看不懂。
- 配好 API 后不知道下一步怎么打开 Codex。
- 软件更新时不知道去哪下载新版。

SY Codex 助手把这些步骤整理成：

1. 选择工具。
2. 安装 Codex。
3. 配置 API。
4. 打开 Codex 桌面 App。
5. 设置风格。
6. 意见反馈。

## 适合谁

- 第一次使用 Codex 的 AI 编程新手。
- Microsoft Store 无法正常下载的 Windows 用户。
- 需要配置 OpenAI 兼容 API 或代理 API 的用户。
- 不熟悉配置文件的普通用户。
- AI 编程课程、社群、培训用户。
- 想要 Codex 中文操作引导的中文用户。

## 当前功能

- Codex 一键安装入口。
- 默认安装线路配置。
- API Base URL 和 API Key 保存。
- Responses API / Chat Completions 协议选择。
- 模型列表获取和模型选择。
- 写入 Codex 本地 `config.toml` 和 `auth.json`。
- Codex 原生菜单和页面运行时中文增强。
- 软件内检查更新、下载进度、SHA256 校验和安装包启动。
- 意见反馈页面。
- OpenClaw 安装入口预留。

## 默认服务

当前默认 API 地址：

```text
https://www.syapi.vip/v1
```

当前默认更新清单：

```text
https://www.syapi.vip/codex-manager/latest.json
```

这些默认值可以在 `src/lib/defaults.ts` 中修改，适配你自己的服务或企业内网环境。

## Codex 安装清单格式

如果你要把 Codex 安装包放到自己的服务器或企业内网镜像，可以提供一个安装清单接口。清单字段使用 camelCase，工具包条目至少包含 `toolId`、`platform`、`version`、`packageUrl` 和 `checksumSha256`。

示例：

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

安装页会读取清单里的 `packageUrl`，下载 Windows 安装包并校验 `checksumSha256`。它不会绕过 Windows 的安装策略；如果电脑被管理员限制安装，仍需要管理员放行。

## 下载

当前推荐从官网固定地址下载：

```text
https://www.syapi.vip/codex-manager/SY-Codex_0.1.9_x64-setup.exe
```

当前版本：

```text
0.1.9
```

SHA256：

```text
075a4a7b79b5e0a3ba5f94c536b069e95cb329b311926a3f3343e3c42ae1c1a3
```

## 安全说明

SY Codex 助手不绕过 Windows 安全机制，不关闭系统防护，也不做免杀处理。

如果下载后提示“不安全程序”，通常是因为安装包未完成代码签名或安全软件信誉不足。正式解决方式是：

- 使用 HTTPS 官网固定下载地址。
- 补齐发布者、公司名、版权和安装包描述。
- 向安全软件厂商提交误报申诉。
- 接入 OV 或 EV 代码签名证书。
- 持续积累下载信誉。

## 隐私说明

- API Key 由用户输入后写入本机 Codex 配置。
- 项目不会把 API Key 上传到反馈接口。
- 意见反馈会附带应用版本、系统类型和诊断日志路径，方便排查问题。
- 真实反馈数据不应提交到 GitHub 仓库。

## 技术栈

- Tauri 2
- React
- TypeScript
- Mantine
- Rust
- NSIS Windows 安装包

## 开发环境

需要先安装：

- Node.js
- npm
- Rust
- Windows 上 Tauri 打包所需依赖

安装依赖：

```powershell
npm install
```

启动开发模式：

```powershell
npm run tauri dev
```

只启动前端：

```powershell
npm run dev
```

## 测试

前端测试：

```powershell
npm test
```

Rust 测试：

```powershell
cargo test --manifest-path src-tauri\Cargo.toml -- --nocapture
```

## 构建

构建前端：

```powershell
npm run build
```

构建 Windows 安装包：

```powershell
npm run tauri build
```

安装包输出位置通常为：

```text
src-tauri\target\release\bundle\nsis\
```

## 发布前检查

公开发布前请确认：

- 仓库中没有服务器密码、API Key、反馈后台 token。
- 仓库中没有用户本地 `.codex` 配置。
- 仓库中没有真实反馈数据。
- 不提交 `node_modules`、`dist`、`src-tauri/target`、安装包和临时缓存。
- `latest.json` 中的中文更新说明建议用 Unicode 转义写入，避免编码变成问号。

## 项目边界

本项目是安装和配置辅助工具，不提供以下能力：

- 不破解 Codex。
- 不伪装 OpenAI 官方应用。
- 不绕过 Windows、Microsoft Store 或安全软件策略。
- 不提供免杀或规避安全检测能力。

## License

MIT
