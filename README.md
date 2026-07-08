# SY Codex（聚合安装）

SY Codex 是面向新手的一键安装和配置工具，帮助用户完成 Codex 桌面 App 安装、代理 API 配置、上游模型获取和 Codex 启动。

## 解决什么问题

- 不熟悉 Codex 配置文件的用户，可以通过图形界面完成配置。
- 国内入门用户可以按步骤配置代理 API，不需要手动编辑 `config.toml` 和 `auth.json`。
- 保存 API 配置前会自动备份旧配置，降低误操作风险。
- 支持一键获取上游模型列表，选择后写入 Codex 默认模型。
- 预留 OpenClaw 入口，后续版本继续完善。

## 当前功能

- Codex 安装入口，默认使用稳定下载线路。
- API 供应商选择：SY API、DeepSeek 官方、智谱官方、自定义。
- SY API 新手教程：充值、创建令牌、粘贴 Key、获取模型、保存配置。
- 自动写入 Codex 配置：
  - Windows：`%USERPROFILE%\.codex`
  - Mac：`~/.codex`
- Codex 中文增强开关，默认开启，不修改官方安装包。
- 意见反馈和版本检测。

## 默认下载线路

Codex 安装页会自动识别系统，并填入对应默认线路：

```text
Windows x64:
https://codexapp.agentsmirror.com/manager/latest/CodexAppManager_x64-setup.exe

macOS Apple Silicon:
https://codexapp.agentsmirror.com/manager/latest/CodexAppManager_aarch64.dmg
```

如果使用清单格式，字段保持如下结构：

```json
{
  "tools": [
    {
      "toolId": "codex",
      "version": "0.142.5",
      "platform": "windows-x64",
      "packageUrl": "https://example.com/CodexAppManager_x64-setup.exe",
      "checksumSha256": "..."
    },
    {
      "toolId": "codex",
      "version": "0.142.5",
      "platform": "macos-arm64",
      "packageUrl": "https://example.com/CodexAppManager_aarch64.dmg",
      "checksumSha256": "..."
    }
  ]
}
```

## SY API 配置

默认供应商：

```text
https://www.syapi.vip/v1
```

使用步骤：

1. 打开 `www.syapi.vip`。
2. 登录后充值。
3. 创建 API 令牌。
4. 回到 SY Codex 粘贴 Key。
5. 点击“一键获取上游模型”。
6. 选择模型并保存配置。
7. 打开 Codex 桌面 App。

客服联系方式：`weixxxnb`

## 下载

当前版本：

```text
0.2.4
```

安装包：

```text
https://www.syapi.vip/codex-manager/SY-Codex_0.2.4_x64-setup.exe
```

SHA256：

```text
64302360e30a64d160af2c7487e01bb634b4b7e194094158e83e07138e1e9778
```

在线更新清单：

```text
https://www.syapi.vip/codex-manager/latest.json
```

## 安全说明

SY Codex 不绕过 Windows 安全机制，不关闭系统防护，也不做免杀处理。

如果下载后提示“不安全程序”，通常是因为安装包没有认证，允许安装即可。正式降低提示的方式是使用 HTTPS 固定下载地址、完善发布者信息、提交误报申诉，并逐步接入代码签名证书。

## 隐私说明

- API Key 由用户输入后写入本机 Codex 配置。
- 项目不会把 API Key 上传到反馈接口。
- 意见反馈会附带应用版本、系统类型和诊断日志路径，方便排查问题。

## 开发

安装依赖：

```powershell
npm install
```

启动前端：

```powershell
npm run dev
```

打包：

```powershell
npm run tauri build
```

测试：

```powershell
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```
