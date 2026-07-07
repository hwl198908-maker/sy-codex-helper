import { useState } from "react";
import "@mantine/core/styles.css";
import {
  AppShell,
  Badge,
  Box,
  Button,
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
import { getNextStep } from "./lib/wizard";
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

type StepGuide = {
  label: string;
  current: string;
  next: string;
  action: string;
};

const stepGuides: Record<WizardStep, StepGuide> = {
  tool: {
    label: "01",
    current: "选择工具：本版本主打 Codex 一键安装和 API 配置",
    next: "下一步下载安装 Codex 桌面 App",
    action: "下一步",
  },
  install: {
    label: "02",
    current: "下载并安装 Codex，使用默认线路即可",
    next: "安装完成后配置 API 供应商和令牌",
    action: "下一步",
  },
  provider: {
    label: "03",
    current: "选择 API 供应商，粘贴 Key，并一键获取上游模型",
    next: "保存后打开 Codex 桌面 App",
    action: "下一步",
  },
  complete: {
    label: "04",
    current: "完成配置，打开 Codex 桌面 App",
    next: "可继续设置中文增强和界面风格",
    action: "下一步",
  },
  style: {
    label: "05",
    current: "设置界面风格和 Codex 中文增强",
    next: "最后可以提交意见反馈",
    action: "下一步",
  },
  feedback: {
    label: "06",
    current: "提交意见反馈",
    next: "反馈会进入后台，后续用于版本更新排期",
    action: "完成",
  },
};

const stepLabels: Record<WizardStep, string> = {
  tool: "选择工具",
  install: "安装 Codex",
  provider: "配置 API",
  complete: "打开 Codex",
  style: "中文增强",
  feedback: "意见反馈",
};

const steps: WizardStep[] = ["tool", "install", "provider", "complete", "style", "feedback"];

const stepProgress: Record<WizardStep, number> = {
  tool: 16,
  install: 32,
  provider: 50,
  complete: 66,
  style: 84,
  feedback: 100,
};

export default function App() {
  const [step, setStep] = useState<WizardStep>("tool");
  const [providerForm, setProviderForm] = useState<ProviderFormState>({
    providerPresetId: "sy_api",
    baseUrl: DEFAULT_PROVIDER_BASE_URL,
    apiKey: "",
    protocol: DEFAULT_PROVIDER_PROTOCOL,
    selectedModel: DEFAULT_PROVIDER_MODEL,
  });
  const guide = stepGuides[step];

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
                    一键安装 Codex，并完成代理 API 配置。新手按步骤操作即可。
                  </Text>
                </Box>
                <Group gap="xs">
                  <Badge size="lg" color="green" variant="filled">GPT-5.5</Badge>
                  <Badge size="lg" color="gray" variant="light">v{APP_VERSION}</Badge>
                </Group>
              </Group>
              <Progress value={stepProgress[step]} mt="md" radius="xl" />
            </Paper>

            <nav className="steps" aria-label="安装步骤">
              {steps.map((item, index) => (
                <button
                  type="button"
                  className={item === step ? "step active" : "step"}
                  key={item}
                  onClick={() => setStep(item)}
                >
                  <span className="step-number">{index + 1}</span>
                  <span>{stepLabels[item]}</span>
                </button>
              ))}
            </nav>

            <Paper className="guide-card" radius="md" p="md">
              <Group gap="md" align="center" wrap="nowrap">
                <div className="guide-index">{guide.label}</div>
                <Box className="guide-copy">
                  <Text fw={800}>{guide.current}</Text>
                  <Text c="dimmed" size="sm" mt={3}>{guide.next}</Text>
                </Box>
                <Button
                  className="guide-next-button"
                  radius="xl"
                  variant="light"
                  color="green"
                  onClick={() => setStep(getNextStep(step))}
                >
                  {guide.action}
                </Button>
              </Group>
            </Paper>

            {step === "provider" ? (
              <SimpleGrid cols={{ base: 1, md: 2 }} spacing="md">
                <ProviderStep form={providerForm} onFormChange={setProviderForm} />
                <Paper className="panel tutorial-panel" radius="md" p="xl">
                  <Stack gap="sm">
                    <div>
                      <Text className="eyebrow">新手教程</Text>
                      <Title order={2}>SY API 配置步骤</Title>
                    </div>
                    <Text><b>1. 打开网站：</b>进入 www.syapi.com。</Text>
                    <Text><b>2. 充值：</b>登录后跳转充值页面完成充值。</Text>
                    <Text><b>3. 创建令牌：</b>在后台创建 API 令牌，复制生成的 Key。</Text>
                    <Text><b>4. 回到软件：</b>粘贴 Key，点击“一键获取上游模型”。</Text>
                    <Text><b>5. 保存配置：</b>选择模型后保存，再打开 Codex。</Text>
                    <Text c="dimmed">Windows 写入 %USERPROFILE%\.codex；Mac 写入 ~/.codex。</Text>
                    <Text fw={700}>客服联系方式：weixxxnb</Text>
                  </Stack>
                </Paper>
              </SimpleGrid>
            ) : null}

            {step === "tool" && <ToolStep />}
            {step === "install" && <InstallStep />}
            {step === "complete" && <CompleteStep providerForm={providerForm} />}
            {step === "style" && <StyleSettingsStep />}
            {step === "feedback" && <FeedbackStep />}
          </Stack>
        </Container>
      </AppShell>
    </MantineProvider>
  );
}
