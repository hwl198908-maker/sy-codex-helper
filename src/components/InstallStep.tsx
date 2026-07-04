import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { DEFAULT_MIRROR_BASE_URL } from "../lib/defaults";
import type { InstallStatus, MirrorManifest, MirrorToolPackage } from "../types";
import { AdvancedSettings } from "./AdvancedSettings";

export function InstallStep() {
  const [mirrorBaseUrl, setMirrorBaseUrl] = useState(DEFAULT_MIRROR_BASE_URL);
  const [manifest, setManifest] = useState<MirrorManifest | null>(null);
  const [error, setError] = useState("");
  const [installMessage, setInstallMessage] = useState("");
  const [isReading, setIsReading] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);

  const codexPackage = manifest ? findCodexWindowsPackage(manifest) : undefined;
  const downloadPercent = useMemo(() => {
    if (!downloadProgress?.totalBytes) {
      return undefined;
    }
    return Math.min(100, Math.round((downloadProgress.downloadedBytes / downloadProgress.totalBytes) * 100));
  }, [downloadProgress]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<DownloadProgress>("codex-download-progress", (event) => {
      setDownloadProgress(event.payload);
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  async function readManifest() {
    setError("");
    setInstallMessage("");
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

  async function downloadAndInstall() {
    setError("");
    setInstallMessage("");
    setDownloadProgress({ phase: "准备下载", downloadedBytes: 0 });
    setIsInstalling(true);

    try {
      const result = await invoke<InstallStatus>("download_and_open_codex", {
        baseUrl: mirrorBaseUrl,
      });
      setInstallMessage(result.message);
      if (result.available_version) {
        setManifest({
          tools: [
            {
              toolId: "codex",
              version: result.available_version,
              platform: "windows-x64",
              packageUrl: "",
              checksumSha256: "",
            },
          ],
        });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsInstalling(false);
    }
  }

  return (
    <section className="panel" aria-labelledby="install-title">
      <div className="section-heading">
        <p className="eyebrow">第 2 步</p>
        <h2 id="install-title">安装 Codex</h2>
        <p>默认使用镜像线路下载 Codex Windows 安装包，也可以手动更换镜像地址。</p>
      </div>

      <label className="field">
        <span>镜像 Base URL</span>
        <input
          value={mirrorBaseUrl}
          onChange={(event) => setMirrorBaseUrl(event.currentTarget.value)}
          placeholder={DEFAULT_MIRROR_BASE_URL}
        />
      </label>

      <div className="button-row">
        <button type="button" onClick={readManifest} disabled={isReading}>
          {isReading ? "正在读取..." : "读取镜像清单"}
        </button>
        <button type="button" className="primary" onClick={downloadAndInstall} disabled={isInstalling}>
          {isInstalling ? "正在下载..." : "下载并安装 Codex"}
        </button>
      </div>

      <div className="status-box" role="status">
        {error && <p>{error}</p>}
        {!error && isInstalling && downloadProgress && (
          <div className="download-progress">
            <div className="download-progress-header">
              <strong>{downloadProgress.phase}</strong>
              <span>{downloadPercent === undefined ? "请稍候" : `${downloadPercent}%`}</span>
            </div>
            <progress
              aria-label="codex-download-progress"
              max={downloadProgress.totalBytes ?? 100}
              value={downloadProgress.totalBytes ? downloadProgress.downloadedBytes : 35}
            />
            <p>
              已下载 {formatBytes(downloadProgress.downloadedBytes)}
              {downloadProgress.totalBytes ? ` / ${formatBytes(downloadProgress.totalBytes)}` : ""}
            </p>
          </div>
        )}
        {!error && installMessage && <p>{installMessage}</p>}
        {!error && codexPackage && <p>可用版本：Codex Windows {codexPackage.version}</p>}
        {!error && manifest && !codexPackage && <p>镜像清单中没有 Codex Windows 安装包。</p>}
        {!error && !manifest && !installMessage && <p>本地状态：尚未执行安装。可以直接下载并安装，或先读取镜像清单。</p>}
      </div>

      <AdvancedSettings />
    </section>
  );
}

type DownloadProgress = {
  phase: string;
  downloadedBytes: number;
  totalBytes?: number | null;
};

function formatBytes(bytes: number): string {
  if (bytes < 1024 * 1024) {
    return `${Math.max(0, Math.round(bytes / 1024))} KB`;
  }
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function findCodexWindowsPackage(manifest: MirrorManifest): MirrorToolPackage | undefined {
  return manifest.tools.find(
    (tool) => tool.toolId === "codex" && tool.platform === "windows-x64",
  );
}
