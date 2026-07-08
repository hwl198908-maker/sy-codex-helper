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
    current: "选择工具：第一版主打 Codex 一键安装和 API 配置。",
    next: "下一步：下载并安装 Codex 桌面 App。",
    action: "下一步",
  },
  install: {
    label: "02",
    current: "安装 Codex：使用默认线路下载安装包。",
    next: "下一步：配置 API 供应商和令牌。",
    action: "下一步",
  },
  provider: {
    label: "03",
    current: "配置 API：选择供应商、粘贴 Key，并一键获取上游模型。",
    next: "下一步：保存配置后打开 Codex 桌面 App。",
    action: "下一步",
  },
  complete: {
    label: "04",
    current: "完成配置：打开 Codex 桌面 App。",
    next: "下一步：可继续设置中文增强和界面风格。",
    action: "下一步",
  },
  style: {
    label: "05",
    current: "设置风格：管理 Codex 中文增强和诊断信息。",
    next: "下一步：提交使用反馈，方便后续版本更新。",
    action: "下一步",
  },
  feedback: {
    label: "06",
    current: "意见反馈：提交安装、配置、中文显示或更新问题。",
    next: "反馈会进入后台，后续用于版本更新排期。",
    action: "完成",
  },
};

const stepLabels: Record<WizardStep, string> = {
  tool: "选择工具",
  install: "安装 Codex",
  provider: "配置 API",
  complete: "打开 Codex",
  style: "中文增强",
  feedback: "反馈更新",
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
        <Container className="app-frame" size={1120} py="xl">
          <Stack gap="md">
            <Paper className="hero-card" radius="md" p="lg">
              <Group justify="space-between" align="center" gap="md">
                <Group gap="md" align="center" wrap="nowrap">
                  <div className="brand-mark" aria-hidden="true">SY</div>
                  <Box>
                    <Title id="app-title" order={1}>SY Codex（聚合安装）</Title>
                    <Text c="dimmed" mt={6}>
                      一键安装 Codex，并完成代理 API 配置。按步骤操作即可。
                    </Text>
                  </Box>
                </Group>
                <Group gap="xs" className="hero-badges">
                  <Text size="sm" c="dimmed">当前版本 {APP_VERSION}</Text>
                  <Badge size="lg" color="green" variant="light">中文增强已开启</Badge>
                </Group>
              </Group>
              <Progress value={stepProgress[step]} mt="lg" radius="xl" size="sm" />
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
                  <Stack gap="md">
                    <Group justify="space-between">
                      <div>
                        <Text className="eyebrow">新手教程</Text>
                        <Title order={2}>SY API 配置步骤</Title>
                      </div>
                      <Badge color="green" variant="light">SY API</Badge>
                    </Group>
                    <div className="tutorial-hero">
                      <Text fw={900}>SY API 是什么</Text>
                      <Text size="sm" c="dimmed" mt={6}>
                        2 折 GPT 模型聚合入口，适合国内用户快速配置 Codex 代理 API。
                      </Text>
                    </div>
                    <Stack gap="sm">
                      <div className="tutorial-item"><b>1 打开网站</b><span>进入 www.syapi.com，登录后跳转充值页面。</span></div>
                      <div className="tutorial-item"><b>2 充值并创建令牌</b><span>充值后在后台创建 API 令牌，复制生成的 Key。</span></div>
                      <div className="tutorial-item"><b>3 回到软件填写</b><span>粘贴 Key，点击“一键获取上游模型”，选择模型后保存。</span></div>
                      <div className="tutorial-item"><b>4 自动识别系统</b><span>Windows 写入 %USERPROFILE%\.codex；Mac 写入 ~/.codex。</span></div>
                    </Stack>
                    <Paper className="support-card" radius="md" p="md">
                      <Text fw={800}>客服联系方式</Text>
                      <Text c="dimmed" mt={4}>weixxxnb</Text>
                    </Paper>
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
