import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Alert, Button, Group, Paper, Progress, Stack, Text, TextInput, Title } from "@mantine/core";
import { DEFAULT_MIRROR_BASE_URL } from "../lib/defaults";
import type { InstallStatus, MirrorManifest, MirrorToolPackage } from "../types";

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
    <Paper className="panel" radius="md" p="xl">
      <Stack gap="md">
        <div>
          <Text className="eyebrow">第 2 步</Text>
          <Title order={2}>安装 Codex</Title>
          <Text c="dimmed" mt={6}>
            使用默认线路下载安装包。下载时会显示进度，完成后会打开安装程序。
          </Text>
        </div>

        <TextInput
          label="默认下载线路"
          description="一般保持默认即可。"
          value={mirrorBaseUrl}
          onChange={(event) => setMirrorBaseUrl(event.currentTarget.value)}
          placeholder={DEFAULT_MIRROR_BASE_URL}
        />

        <Group>
          <Button variant="default" onClick={readManifest} loading={isReading}>
            检查安装包
          </Button>
          <Button onClick={downloadAndInstall} loading={isInstalling}>
            安装 Codex
          </Button>
        </Group>

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
          {!error && codexPackage && <Text>可用版本：Codex Windows {codexPackage.version}</Text>}
          {!error && manifest && !codexPackage && <Text>清单中没有找到 Codex Windows 安装包。</Text>}
          {!error && !manifest && !installMessage && !isInstalling && (
            <Text>本地状态：尚未执行安装。可以直接点击“安装 Codex”。</Text>
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

function findCodexWindowsPackage(manifest: MirrorManifest): MirrorToolPackage | undefined {
  return manifest.tools.find(
    (tool) => tool.toolId === "codex" && tool.platform === "windows-x64",
  );
}
