import { useState } from "react";
import { AdvancedSettings } from "./AdvancedSettings";

export function InstallStep() {
  const [mirrorBaseUrl, setMirrorBaseUrl] = useState("https://registry.npmjs.org");

  return (
    <section className="panel" aria-labelledby="install-title">
      <div className="section-heading">
        <p className="eyebrow">第 2 步</p>
        <h2 id="install-title">安装 Codex</h2>
        <p>这里先确认下载来源。安装命令会在后续版本接入。</p>
      </div>

      <label className="field">
        <span>镜像 Base URL</span>
        <input
          value={mirrorBaseUrl}
          onChange={(event) => setMirrorBaseUrl(event.currentTarget.value)}
          placeholder="例如：https://registry.npmjs.org"
        />
      </label>

      <div className="status-box" role="status">
        本地状态：尚未执行安装。下一版会在这里显示安装进度和版本信息。
      </div>

      <AdvancedSettings />
    </section>
  );
}
