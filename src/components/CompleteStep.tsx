import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Alert,
  Anchor,
  Badge,
  Button,
  Group,
  List,
  Paper,
  Progress,
  SimpleGrid,
  Stack,
  Switch,
  Text,
  ThemeIcon,
  Title
} from "@mantine/core";
import type { ProviderFormState } from "../types";
import { APP_VERSION, SY_API_SITE_URL, UPDATE_MANIFEST_URL } from "../lib/defaults";
import { isNewerVersion, parseUpdateManifest, type UpdateManifest } from "../lib/updates";

type CompleteStepProps = {
  providerForm: ProviderFormState;
};

type UpdateDownloadProgress = {
  downloaded: number;
  total?: number;
  percent?: number;
};

type UpdateInstallResult = {
  version: string;
  path: string;
  reusedCachedFile: boolean;
};

export function CompleteStep({ providerForm }: CompleteStepProps) {
  const [status, setStatus] = useState("最后一步：点击按钮打开 Codex 桌面 App。");
  const [updateStatus, setUpdateStatus] = useState("尚未检查更新。");
  const [updateManifest, setUpdateManifest] = useState<UpdateManifest | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [isInstallingUpdate, setIsInstallingUpdate] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<UpdateDownloadProgress | null>(null);
  const [enhancedMenu, setEnhancedMenu] = useState(true);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<UpdateDownloadProgress>("update-download-progress", (event) => {
      setDownloadProgress(event.payload);
    }).then((nextUnlisten) => {
      unlisten = nextUnlisten;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  async function openCodex() {
    setStatus("正在打开 Codex 桌面 App...");
    try {
      await invoke("open_codex", { enhancedMenu });
      setStatus("已尝试打开 Codex。如果没有看到窗口，请确认 Codex 已安装完成。");
    } catch (err) {
      setStatus(err instanceof Error ? err.message : String(err));
    }
  }

  async function checkUpdate() {
    setIsCheckingUpdate(true);
    setUpdateStatus("正在检查新版本...");
    setUpdateManifest(null);

    try {
      const manifest = parseUpdateManifest(await invoke("check_update", { manifestUrl: UPDATE_MANIFEST_URL }));
      setUpdateManifest(manifest);
      setUpdateStatus(
        isNewerVersion(APP_VERSION, manifest.version)
          ? `发现新版本 ${manifest.version}。`
          : `当前已经是最新版本 ${APP_VERSION}。`
      );
    } catch (error) {
      setUpdateStatus(`检查更新失败：${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsCheckingUpdate(false);
    }
  }

  async function installUpdate(manifest: UpdateManifest) {
    setIsInstallingUpdate(true);
    setDownloadProgress(null);
    setUpdateStatus(`正在下载新版本 ${manifest.version}...`);

    try {
      const result = await invoke<UpdateInstallResult>("download_and_install_update", { manifest });
      setUpdateStatus(
        result.reusedCachedFile
          ? `已使用缓存安装包，正在启动 ${result.version} 安装程序。`
          : `新版本 ${result.version} 下载完成，正在启动安装程序。`
      );
    } catch (error) {
      setUpdateStatus(`在线更新失败：${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsInstallingUpdate(false);
    }
  }

  return (
    <Paper className="panel" radius="md" p="xl">
      <Stack gap="md">
        <Paper className="finish-hero" radius="md" p="lg">
          <Group justify="space-between" align="center" gap="md">
            <div>
              <Badge color="green" variant="filled">第 4 步 / 最后一步</Badge>
              <Title order={2} mt={8}>完成配置，打开 Codex 桌面 App</Title>
              <Text c="dimmed" mt={6}>
                API 保存后，点击右侧按钮启动 Codex，并开始使用你选择的模型。
              </Text>
            </div>
            <Button className="open-codex-button success-action" size="xl" onClick={openCodex}>
              打开 Codex 桌面 App
            </Button>
          </Group>
        </Paper>

        <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
          <Paper withBorder radius="md" p="md">
            <Text fw={700}>当前 API 设置</Text>
            <Text size="sm" c="dimmed" mt={8}>Base URL：{providerForm.baseUrl}</Text>
            <Text size="sm" c="dimmed">默认模型：{providerForm.selectedModel || "gpt-5.5"}</Text>
            <Text size="sm" c="dimmed">
              协议：{providerForm.protocol === "responses" ? "Responses API" : "Chat Completions"}
            </Text>
          </Paper>

          <Paper withBorder radius="md" p="md">
            <Text fw={700}>配置写入位置</Text>
            <Text size="sm" c="dimmed" mt={8}>Windows：%USERPROFILE%\.codex</Text>
            <Text size="sm" c="dimmed">Mac：~/.codex</Text>
            <Text size="sm" c="dimmed">保存前会自动备份旧配置。</Text>
          </Paper>
        </SimpleGrid>

        <Paper withBorder radius="md" p="md">
          <Group justify="space-between" align="center" gap="md">
            <div>
              <Text fw={700}>Codex 中文增强</Text>
              <Text size="sm" c="dimmed" mt={4}>
                默认开启，不修改官方安装包；打开 Codex 时运行时尝试汉化页面和原生菜单。
              </Text>
            </div>
            <Switch
              checked={enhancedMenu}
              onChange={(event) => setEnhancedMenu(event.currentTarget.checked)}
            />
          </Group>
        </Paper>

        <Paper withBorder radius="md" p="md">
          <Text fw={700}>SY API 配置教程</Text>
          <List mt="sm" spacing="xs" icon={<ThemeIcon color="blue" size={20} radius="xl">✓</ThemeIcon>}>
            <List.Item>
              打开 <Anchor href={SY_API_SITE_URL} target="_blank" rel="noreferrer">www.syapi.vip</Anchor>。
            </List.Item>
            <List.Item>充值后创建令牌，把令牌复制到“配置 API”的 API Key。</List.Item>
            <List.Item>点击“一键获取上游模型”，选择模型后保存配置。</List.Item>
            <List.Item>客服联系方式：weixxxnb。</List.Item>
          </List>
        </Paper>

        <Alert color="yellow" variant="light">
          下载后提示“不安全程序”通常是因为安装包没有认证，允许安装即可。
        </Alert>

        <Group>
          <Button variant="default" onClick={checkUpdate} loading={isCheckingUpdate}>检查更新</Button>
          {updateManifest && isNewerVersion(APP_VERSION, updateManifest.version) && (
            <Button variant="light" onClick={() => void installUpdate(updateManifest)} loading={isInstallingUpdate}>
              立即更新
            </Button>
          )}
        </Group>

        {isInstallingUpdate && (
          <Paper withBorder radius="md" p="md">
            <Text fw={700}>正在下载新版本安装包</Text>
            <Progress value={downloadProgress?.percent ?? 0} mt="sm" animated />
            <Text size="sm" c="dimmed" mt={6}>
              {formatDownloadProgress(downloadProgress)}
            </Text>
          </Paper>
        )}

        <Alert color="gray" variant="light">
          <Text>{status}</Text>
          <Text mt={6}>当前版本：{APP_VERSION}。{updateStatus}</Text>
          {updateManifest?.notes && <Text mt={6}>更新说明：{updateManifest.notes}</Text>}
        </Alert>
      </Stack>
    </Paper>
  );
}

function formatDownloadProgress(progress: UpdateDownloadProgress | null): string {
  if (!progress) {
    return "正在连接更新服务器...";
  }

  if (progress.total && progress.percent !== undefined) {
    return `${formatBytes(progress.downloaded)} / ${formatBytes(progress.total)}（${progress.percent}%）`;
  }

  return `已下载 ${formatBytes(progress.downloaded)}`;
}

function formatBytes(value: number): string {
  if (value >= 1024 * 1024) {
    return `${(value / 1024 / 1024).toFixed(1)} MB`;
  }
  if (value >= 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${value} B`;
}
