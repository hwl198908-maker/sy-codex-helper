const summary = [
  ["安装状态", "等待接入安装命令"],
  ["配置状态", "等待接入保存命令"],
  ["默认模型", "尚未选择"],
  ["配置目录", "%APPDATA%\\codex-manager"],
];

export function CompleteStep() {
  return (
    <section className="panel" aria-labelledby="complete-title">
      <div className="section-heading">
        <p className="eyebrow">完成</p>
        <h2 id="complete-title">检查配置结果</h2>
        <p>后续接入后，这里会显示真实安装、配置和模型状态。</p>
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
        本地状态：按钮暂为占位操作，暂不执行后端命令。
      </div>
    </section>
  );
}
