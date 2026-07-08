import { Badge, Group, Paper, SimpleGrid, Stack, Text, Title } from "@mantine/core";

export function ToolStep() {
  return (
    <Paper className="panel" radius="md" p="xl">
      <Stack gap="md">
        <Group justify="space-between" align="flex-start">
          <div>
            <Text className="eyebrow">第 1 步</Text>
            <Title order={2}>选择工具</Title>
            <Text c="dimmed" mt={6}>
              第一版主打 Codex 一键安装和 API 配置。OpenClaw 入口先预留，后续版本上线。
            </Text>
          </div>
          <Badge color="green" variant="light">Codex 当前可用</Badge>
        </Group>

        <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
          <Paper className="tool-card selected" p="lg" radius="md">
            <Group justify="space-between" align="flex-start" wrap="nowrap">
              <div className="tool-icon">C</div>
              <div className="tool-copy">
                <Title order={3}>Codex</Title>
                <Text c="dimmed" mt={8}>下载安装 + API 配置 + 打开桌面 App。</Text>
              </div>
              <Badge color="blue">已选择</Badge>
            </Group>
          </Paper>

          <Paper className="tool-card disabled" p="lg" radius="md">
            <Group justify="space-between" align="flex-start" wrap="nowrap">
              <div className="tool-icon muted">O</div>
              <div className="tool-copy">
                <Title order={3}>OpenClaw</Title>
                <Text c="dimmed" mt={8}>预留入口，后续加入一键安装和代理 API 配置。</Text>
              </div>
              <Badge color="yellow" variant="light">预留</Badge>
            </Group>
          </Paper>
        </SimpleGrid>
      </Stack>
    </Paper>
  );
}
