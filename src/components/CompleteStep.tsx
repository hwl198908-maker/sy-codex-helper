import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const summary = [
  ["安装状态", "完成第 2 步后即可打开 Codex"],
  ["配置状态", "保存 API 设置后写入 Codex 配置"],
  ["默认模型", "尚未选择"],
  ["配置目录", "%USERPROFILE%\\.codex"],
];

export function CompleteStep() {
  const [status, setStatus] = useState("安装和 API 配置完成后，可以在这里打开 Codex。");

  async function openCodex() {
    setStatus("正在打开 Codex...");
    try {
      await invoke("open_codex");
      setStatus("已尝试打开 Codex。如果没有看到窗口，请确认 Codex 已安装完成。");
    } catch (err) {
      setStatus(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <section className="panel" aria-labelledby="complete-title">
      <div className="section-heading">
        <p className="eyebrow">完成</p>
        <h2 id="complete-title">检查配置结果</h2>
        <p>这里展示本机配置会写到哪里，不代表实时检测到的运行状态。</p>
      </div>

      <dl className="summary-list">
        {summary.map(([label, value]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>

      <div className="button-row">
        <button type="button" className="primary" onClick={openCodex}>
          打开 Codex
        </button>
        <button type="button">打开配置目录</button>
        <button type="button">检查更新</button>
      </div>

      <div className="status-box" role="status">
        <p>{status}</p>
        <p>保存 API 设置后，应用会把 Codex 配置和登录凭据写入 %USERPROFILE%\.codex，并在覆盖前自动备份旧配置。</p>
      </div>
    </section>
  );
}
