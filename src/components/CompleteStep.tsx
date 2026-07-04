import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Alert,
  Anchor,
  Badge,
  Button,
  Group,
  List,
  Paper,
  SimpleGrid,
  Stack,
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

export function CompleteStep({ providerForm }: CompleteStepProps) {
  const [status, setStatus] = useState("最后一步：点击上方按钮打开 Codex 桌面 App。");
  const [updateStatus, setUpdateStatus] = useState("尚未检查更新。");
  const [updateManifest, setUpdateManifest] = useState<UpdateManifest | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);

  async function openCodex() {
    setStatus("正在打开 Codex 桌面 App...");
    try {
      await invoke("open_codex");
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

  function openDownload(url: string) {
    window.open(url, "_blank", "noopener,noreferrer");
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
                API 已保存后，点击右侧按钮启动 Codex，并在 Codex 中开始使用 GPT-5.5。
              </Text>
            </div>
            <Button className="open-codex-button" size="xl" onClick={openCodex}>
              打开 Codex 桌面 App
            </Button>
          </Group>
        </Paper>

        <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
          <Paper withBorder radius="md" p="md">
            <Text fw={700}>当前 API 设置</Text>
            <Text size="sm" c="dimmed" mt={8}>Base URL：{providerForm.baseUrl}</Text>
            <Text size="sm" c="dimmed">默认模型：{providerForm.selectedModel || "gpt-5.5"}</Text>
            <Text size="sm" c="dimmed">协议：{providerForm.protocol === "responses" ? "Responses API" : "Chat Completions"}</Text>
          </Paper>

          <Paper withBorder radius="md" p="md">
            <Text fw={700}>软件定位</Text>
            <Text size="sm" c="dimmed" mt={8}>
              一键安装 Codex、预留 OpenClaw，并帮助用户完成代理 API 中转设置。
            </Text>
          </Paper>
        </SimpleGrid>

        <Paper withBorder radius="md" p="md">
          <Text fw={700}>SY API 配置教程</Text>
          <List mt="sm" spacing="xs" icon={<ThemeIcon color="blue" size={20} radius="xl">✓</ThemeIcon>}>
            <List.Item>
              打开 <Anchor href={SY_API_SITE_URL} target="_blank" rel="noreferrer">www.syapi.vip</Anchor>。
            </List.Item>
            <List.Item>充值后创建令牌，把令牌复制到“配置 API”的 API Key。</List.Item>
            <List.Item>Base URL 默认使用 https://www.syapi.vip/v1。</List.Item>
            <List.Item>获取模型列表后选择 GPT-5.5，或直接保存默认 GPT-5.5。</List.Item>
            <List.Item>客服联系方式：weixxxnb。</List.Item>
          </List>
        </Paper>

        <Alert color="yellow" variant="light">
          下载后提示“不安全程序”通常是因为安装包没有认证，允许安装即可。
        </Alert>

        <Group>
          <Button variant="default" onClick={checkUpdate} loading={isCheckingUpdate}>检查更新</Button>
          {updateManifest && isNewerVersion(APP_VERSION, updateManifest.version) && (
            <Button variant="light" onClick={() => openDownload(updateManifest.downloadUrl)}>下载新版</Button>
          )}
        </Group>

        <Alert color="gray" variant="light">
          <Text>{status}</Text>
          <Text mt={6}>保存 API 设置后，会写入 %USERPROFILE%\.codex，并自动备份旧配置。</Text>
          <Text mt={6}>当前版本：{APP_VERSION}。{updateStatus}</Text>
          {updateManifest?.notes && <Text mt={6}>更新说明：{updateManifest.notes}</Text>}
        </Alert>
      </Stack>
    </Paper>
  );
}
