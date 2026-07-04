import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MirrorManifest, MirrorToolPackage } from "../types";
import { AdvancedSettings } from "./AdvancedSettings";

export function InstallStep() {
  const [mirrorBaseUrl, setMirrorBaseUrl] = useState("");
  const [manifest, setManifest] = useState<MirrorManifest | null>(null);
  const [error, setError] = useState("");
  const [isReading, setIsReading] = useState(false);

  const codexPackage = manifest ? findCodexWindowsPackage(manifest) : undefined;

  async function readManifest() {
    setError("");
    setManifest(null);
    setIsReading(true);

    try {
      const result = await invoke<MirrorManifest>("read_mirror_manifest", {
        baseUrl: mirrorBaseUrl,
      });
      setManifest(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsReading(false);
    }
  }

  return (
    <section className="panel" aria-labelledby="install-title">
      <div className="section-heading">
        <p className="eyebrow">第 2 步</p>
        <h2 id="install-title">安装 Codex</h2>
        <p>先读取镜像清单，确认可用的 Windows 安装包。安装动作会在后续版本接入。</p>
      </div>

      <label className="field">
        <span>镜像 Base URL</span>
        <input
          value={mirrorBaseUrl}
          onChange={(event) => setMirrorBaseUrl(event.currentTarget.value)}
          placeholder="例如：https://mirror.example.com/codex"
        />
      </label>

      <div className="button-row">
        <button type="button" onClick={readManifest} disabled={isReading}>
          {isReading ? "正在读取..." : "读取镜像清单"}
        </button>
        <button type="button" disabled>
          安装功能待接入
        </button>
      </div>

      <div className="status-box" role="status">
        {error && <p>{error}</p>}
        {!error && codexPackage && <p>可用版本：Codex Windows {codexPackage.version}</p>}
        {!error && manifest && !codexPackage && <p>镜像清单中没有 Codex Windows 安装包。</p>}
        {!error && !manifest && <p>本地状态：尚未执行安装。请先读取镜像清单。</p>}
      </div>

      <AdvancedSettings />
    </section>
  );
}

function findCodexWindowsPackage(manifest: MirrorManifest): MirrorToolPackage | undefined {
  return manifest.tools.find(
    (tool) => tool.tool_id === "codex" && tool.platform === "windows-x64",
  );
}
