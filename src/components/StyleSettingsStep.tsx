import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Alert,
  Badge,
  Button,
  Group,
  Paper,
  SegmentedControl,
  SimpleGrid,
  Stack,
  Switch,
  Text,
  Title
} from "@mantine/core";

export function StyleSettingsStep() {
  const [tone, setTone] = useState("beginner");
  const [enhancedChinese, setEnhancedChinese] = useState(true);
  const [showProgress, setShowProgress] = useState(true);
  const [diagnosticPath, setDiagnosticPath] = useState("尚未读取日志位置。");

  async function readDiagnosticPath() {
    try {
      const path = await invoke<string>("get_diagnostic_log_path");
      setDiagnosticPath(path);
    } catch (error) {
      setDiagnosticPath(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <Paper className="panel" radius="md" p="xl">
      <Stack gap="md">
        <div>
          <Text className="eyebrow">第 5 步</Text>
          <Title order={2}>设置风格</Title>
          <Text c="dimmed" mt={6}>
            这里先放新手最容易理解的显示与诊断设置，默认推荐项已经选好。
          </Text>
        </div>

        <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
          <Paper withBorder radius="md" p="md">
            <Group justify="space-between" align="center">
              <div>
                <Text fw={800}>Codex 中文增强</Text>
                <Text size="sm" c="dimmed" mt={4}>
                  打开 Codex 时自动尝试汉化菜单、侧边栏、首页和插件页。
                </Text>
              </div>
              <Switch checked={enhancedChinese} onChange={(event) => setEnhancedChinese(event.currentTarget.checked)} />
            </Group>
          </Paper>

          <Paper withBorder radius="md" p="md">
            <Group justify="space-between" align="center">
              <div>
                <Text fw={800}>显示进度指引</Text>
                <Text size="sm" c="dimmed" mt={4}>
                  保留顶部步骤和下一步提示，适合第一次使用的用户。
                </Text>
              </div>
              <Switch checked={showProgress} onChange={(event) => setShowProgress(event.currentTarget.checked)} />
            </Group>
          </Paper>
        </SimpleGrid>

        <Paper withBorder radius="md" p="md">
          <Text fw={800}>界面风格</Text>
          <SegmentedControl
            mt="sm"
            value={tone}
            onChange={setTone}
            data={[
              { label: "新手清晰", value: "beginner" },
              { label: "简洁专业", value: "compact" },
              { label: "诊断优先", value: "diagnostic" },
            ]}
          />
          <Text size="sm" c="dimmed" mt="sm">
            当前版本先保存为页面状态；后续可扩展为全局主题和持久化偏好。
          </Text>
        </Paper>

        <Alert color="blue" variant="light">
          <Group justify="space-between" align="center" gap="sm">
            <div>
              <Text fw={700}>诊断日志</Text>
              <Text size="sm" mt={4}>{diagnosticPath}</Text>
            </div>
            <Button variant="light" onClick={readDiagnosticPath}>查看日志位置</Button>
          </Group>
        </Alert>

        <Paper withBorder radius="md" p="md">
          <Group gap="xs">
            <Badge color={enhancedChinese ? "green" : "gray"}>中文增强 {enhancedChinese ? "开启" : "关闭"}</Badge>
            <Badge color={showProgress ? "blue" : "gray"}>新手指引 {showProgress ? "开启" : "关闭"}</Badge>
            <Badge color="teal">风格：{tone === "beginner" ? "新手清晰" : tone === "compact" ? "简洁专业" : "诊断优先"}</Badge>
          </Group>
        </Paper>
      </Stack>
    </Paper>
  );
}
