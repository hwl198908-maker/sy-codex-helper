import { useState } from "react";
import "@mantine/core/styles.css";
import {
  AppShell,
  Badge,
  Box,
  Container,
  Group,
  MantineProvider,
  Paper,
  Progress,
  SimpleGrid,
  Stack,
  Text,
  Title
} from "@mantine/core";
import "./styles.css";
import type { ProviderFormState, WizardStep } from "./types";
import { ToolStep } from "./components/ToolStep";
import { InstallStep } from "./components/InstallStep";
import { ProviderStep } from "./components/ProviderStep";
import { CompleteStep } from "./components/CompleteStep";
import { StyleSettingsStep } from "./components/StyleSettingsStep";
import { FeedbackStep } from "./components/FeedbackStep";
import {
  APP_VERSION,
  DEFAULT_PROVIDER_BASE_URL,
  DEFAULT_PROVIDER_MODEL,
  DEFAULT_PROVIDER_PROTOCOL
} from "./lib/defaults";

const stepLabels: Record<WizardStep, string> = {
  tool: "选择工具",
  install: "安装 Codex",
  provider: "配置 API",
  complete: "打开 Codex",
  style: "中文增强",
  feedback: "意见反馈",
};

const steps: WizardStep[] = ["tool", "install", "provider", "complete", "style", "feedback"];

export default function App() {
  const [providerForm, setProviderForm] = useState<ProviderFormState>({
    providerPresetId: "sy_api",
    baseUrl: DEFAULT_PROVIDER_BASE_URL,
    apiKey: "",
    protocol: DEFAULT_PROVIDER_PROTOCOL,
    selectedModel: DEFAULT_PROVIDER_MODEL,
  });

  return (
    <MantineProvider defaultColorScheme="light">
      <AppShell className="app-bg">
        <Container size={1080} py="md">
          <Stack gap="md">
            <Paper className="hero-card" radius="md" p="lg">
              <Group justify="space-between" align="flex-start" gap="md">
                <Box>
                  <Badge color="blue" variant="light">聚合安装</Badge>
                  <Title id="app-title" order={1} mt={8}>SY Codex（聚合安装）</Title>
                  <Text c="dimmed" mt={6}>
                    一键安装 Codex，并完成代理 API 配置。新手按页面顺序操作即可。
                  </Text>
                </Box>
                <Group gap="xs">
                  <Badge size="lg" color="green" variant="filled">GPT-5.5</Badge>
                  <Badge size="lg" color="gray" variant="light">v{APP_VERSION}</Badge>
                </Group>
              </Group>
              <Progress value={100} mt="md" radius="xl" />
            </Paper>

            <nav className="steps" aria-label="操作步骤">
              {steps.map((item, index) => (
                <div className={index < 4 ? "step active" : "step"} key={item}>
                  <span className="step-number">{index + 1}</span>
                  <span>{stepLabels[item]}</span>
                </div>
              ))}
            </nav>

            <SimpleGrid className="main-workflow-grid" cols={{ base: 1, md: 2 }} spacing="md">
              <Stack gap="md">
                <ToolStep />
                <InstallStep />
                <ProviderStep form={providerForm} onFormChange={setProviderForm} />
                <CompleteStep providerForm={providerForm} />
              </Stack>

              <Stack gap="md">
                <Paper className="panel tutorial-panel" radius="md" p="xl">
                  <Stack gap="sm">
                    <div>
                      <Text className="eyebrow">新手教程</Text>
                      <Title order={2}>SY API 配置步骤</Title>
                    </div>
                    <Text><b>SY API 是什么：</b>2 折 GPT 模型聚合入口，适合国内用户快速配置 Codex 代理 API。</Text>
                    <Text><b>1. 打开网站：</b>进入 www.syapi.com。</Text>
                    <Text><b>2. 充值：</b>登录后跳转充值页面完成充值。</Text>
                    <Text><b>3. 创建令牌：</b>在后台创建 API 令牌，复制生成的 Key。</Text>
                    <Text><b>4. 回到软件：</b>粘贴 Key，点击“一键获取上游模型”。</Text>
                    <Text><b>5. 保存配置：</b>选择模型后保存，再打开 Codex。</Text>
                    <Text c="dimmed">Windows 写入 %USERPROFILE%\.codex；Mac 写入 ~/.codex。</Text>
                    <Text fw={700}>客服联系方式：weixxxnb</Text>
                  </Stack>
                </Paper>
                <StyleSettingsStep />
                <FeedbackStep />
              </Stack>
            </SimpleGrid>
          </Stack>
        </Container>
      </AppShell>
    </MantineProvider>
  );
}
