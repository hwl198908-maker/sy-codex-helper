export function AdvancedSettings() {
  return (
    <details className="advanced-settings">
      <summary>高级设置</summary>

      <div className="advanced-grid">
        <label className="field">
          <span>配置目录</span>
          <input defaultValue="%APPDATA%\\codex-manager" />
        </label>

        <label className="field">
          <span>下载缓存目录</span>
          <input defaultValue="%LOCALAPPDATA%\\codex-manager\\cache" />
        </label>

        <label className="check-field">
          <input type="checkbox" defaultChecked />
          <span>启动时自动检查更新</span>
        </label>

        <label className="field">
          <span>User-Agent</span>
          <input defaultValue="CodexManager/0.1" />
        </label>
      </div>
    </details>
  );
}
