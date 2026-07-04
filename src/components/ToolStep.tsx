export function ToolStep() {
  return (
    <section className="panel" aria-labelledby="tool-title">
      <div className="section-heading">
        <p className="eyebrow">第 1 步</p>
        <h2 id="tool-title">选择要安装的工具</h2>
        <p>先选择本次要管理的工具。当前版本只开放 Codex。</p>
      </div>

      <div className="tool-grid">
        <article className="tool-card selected">
          <div>
            <h3>Codex</h3>
            <p>一键安装，并自动完成 API 配置。</p>
          </div>
          <strong>当前选择</strong>
        </article>

        <article className="tool-card disabled" aria-disabled="true">
          <div>
            <h3>OpenClaw</h3>
            <p>已预留安装入口，后续版本开放。</p>
          </div>
          <strong>即将支持</strong>
        </article>
      </div>
    </section>
  );
}
