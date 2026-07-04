import { Badge, Group, Paper, SimpleGrid, Stack, Text, Title } from "@mantine/core";

export function ToolStep() {
  return (
    <Paper className="panel" radius="md" p="xl">
      <Stack gap="md">
        <div>
          <Text className="eyebrow">第 1 步</Text>
          <Title order={2}>选择要安装的工具</Title>
          <Text c="dimmed" mt={6}>
            当前版本先完成 Codex 一键安装和 API 配置，OpenClaw 已预留入口，后续版本开放。
          </Text>
        </div>

        <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
          <Paper className="tool-card selected" p="lg" radius="md">
            <Group justify="space-between" align="flex-start">
              <div>
                <Title order={3}>Codex</Title>
                <Text c="dimmed" mt={8}>下载安装桌面 App，并写入 SY API 配置。</Text>
              </div>
              <Badge color="blue">当前选择</Badge>
            </Group>
          </Paper>

          <Paper className="tool-card disabled" p="lg" radius="md">
            <Group justify="space-between" align="flex-start">
              <div>
                <Title order={3}>OpenClaw</Title>
                <Text c="dimmed" mt={8}>后续加入一键安装和代理 API 配置。</Text>
              </div>
              <Badge color="yellow" variant="light">即将支持</Badge>
            </Group>
          </Paper>
        </SimpleGrid>
      </Stack>
    </Paper>
  );
}
