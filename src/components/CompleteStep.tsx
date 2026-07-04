const summary = [
  ["安装状态", "等待接入安装命令"],
  ["配置状态", "保存 API 设置后写入 Codex 配置"],
  ["默认模型", "尚未选择"],
  ["配置目录", "%USERPROFILE%\\.codex"],
];

export function CompleteStep() {
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
        <button type="button">测试连接</button>
        <button type="button">打开配置目录</button>
        <button type="button">检查更新</button>
      </div>

      <div className="status-box" role="status">
        保存 API 设置后，应用会把 Codex 配置和登录凭据写入 %USERPROFILE%\.codex，并在覆盖前自动备份旧配置。
      </div>
    </section>
  );
}
