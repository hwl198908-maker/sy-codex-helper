import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Alert, Button, Code, Group, Paper, Progress, Stack, Text, TextInput, Title } from "@mantine/core";
import {
  DEFAULT_MACOS_ARM64_MIRROR_BASE_URL,
  DEFAULT_WINDOWS_MIRROR_BASE_URL
} from "../lib/defaults";
import type { InstallStatus, MirrorManifest, MirrorToolPackage } from "../types";

type InstallTarget = {
  label: string;
  platform: "windows-x64" | "macos-arm64";
  defaultUrl: string;
};

function detectInstallTarget(): InstallTarget {
  const platform = `${navigator.platform} ${navigator.userAgent}`.toLowerCase();
  if (platform.includes("mac")) {
    return {
      label: "macOS · Apple Silicon",
      platform: "macos-arm64",
      defaultUrl: DEFAULT_MACOS_ARM64_MIRROR_BASE_URL,
    };
  }

  return {
    label: "Windows · x64",
    platform: "windows-x64",
    defaultUrl: DEFAULT_WINDOWS_MIRROR_BASE_URL,
  };
}

export function InstallStep() {
  const installTarget = useMemo(() => detectInstallTarget(), []);
  const [mirrorBaseUrl, setMirrorBaseUrl] = useState(installTarget.defaultUrl);
  const [manifest, setManifest] = useState<MirrorManifest | null>(null);
  const [error, setError] = useState("");
  const [installMessage, setInstallMessage] = useState("");
  const [isReading, setIsReading] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);

  const codexPackage = manifest ? findCodexPackage(manifest, installTarget.platform) : undefined;
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
              platform: installTarget.platform,
              packageUrl: mirrorBaseUrl,
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
    <Paper className="panel" radius="md" p="xl">
      <Stack gap="md">
        <div>
          <Text className="eyebrow">第 2 步</Text>
          <Title order={2}>安装 Codex</Title>
          <Text c="dimmed" mt={6}>
            软件会自动识别当前系统并填入对应下载线路。Windows 使用 x64 安装包，Mac 使用 Apple Silicon DMG。
          </Text>
        </div>

        <Alert color="blue" variant="light">
          当前识别：{installTarget.label}。如果识别不准，可以手动替换下面的下载地址。
        </Alert>

        <TextInput
          label="默认下载线路"
          description="Windows 和 Mac 分别使用不同安装包；这里默认使用当前系统对应线路。"
          value={mirrorBaseUrl}
          onChange={(event) => setMirrorBaseUrl(event.currentTarget.value)}
          placeholder={installTarget.defaultUrl}
        />

        <Group>
          <Button variant="default" onClick={readManifest} loading={isReading}>
            检查安装包
          </Button>
          <Button className="primary-action" onClick={downloadAndInstall} loading={isInstalling}>
            下载并安装 Codex
          </Button>
        </Group>

        {installTarget.platform === "macos-arm64" && (
          <Alert color="yellow" variant="light">
            <Stack gap={6}>
              <Text fw={700}>Mac 提示“App 已损坏”时</Text>
              <Text size="sm">1. 先将 SY Codex.app 拖入“应用程序”文件夹。</Text>
              <Text size="sm">2. 打开“终端”，粘贴下面命令并按回车。</Text>
              <Code block>xattr -dr com.apple.quarantine "/Applications/SY Codex.app"</Code>
              <Text size="sm">3. 回到“应用程序”，重新打开 SY Codex.app。</Text>
            </Stack>
          </Alert>
        )}

        <Alert color={error ? "red" : "blue"} variant="light">
          {error && <Text>{error}</Text>}
          {!error && isInstalling && downloadProgress && (
            <Stack gap={6}>
              <Group justify="space-between">
                <Text fw={700}>{downloadProgress.phase}</Text>
                <Text>{downloadPercent === undefined ? "请稍等" : `${downloadPercent}%`}</Text>
              </Group>
              <Progress value={downloadPercent ?? 35} animated={downloadPercent === undefined} />
              <Text size="sm" c="dimmed">
                已下载 {formatBytes(downloadProgress.downloadedBytes)}
                {downloadProgress.totalBytes ? ` / ${formatBytes(downloadProgress.totalBytes)}` : ""}
              </Text>
            </Stack>
          )}
          {!error && installMessage && <Text>{installMessage}</Text>}
          {!error && codexPackage && <Text>可用版本：Codex {codexPackage.version}（{codexPackage.platform}）</Text>}
          {!error && manifest && !codexPackage && <Text>清单中没有找到当前系统可用的 Codex 安装包。</Text>}
          {!error && !manifest && !installMessage && !isInstalling && (
            <Text>本地状态：尚未执行安装。可以直接点击“下载并安装 Codex”。</Text>
          )}
        </Alert>
      </Stack>
    </Paper>
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

function findCodexPackage(
  manifest: MirrorManifest,
  platform: InstallTarget["platform"],
): MirrorToolPackage | undefined {
  return manifest.tools.find((tool) => tool.toolId === "codex" && tool.platform === platform);
}
