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
  Stack,
  Text,
  Title
} from "@mantine/core";
import "./styles.css";
import type { ProviderFormState, WizardStep } from "./types";
import { getNextStep, getPreviousStep } from "./lib/wizard";
import { ToolStep } from "./components/ToolStep";
import { InstallStep } from "./components/InstallStep";
import { ProviderStep } from "./components/ProviderStep";
import { CompleteStep } from "./components/CompleteStep";
import {
  DEFAULT_PROVIDER_BASE_URL,
  DEFAULT_PROVIDER_MODEL,
  DEFAULT_PROVIDER_PROTOCOL
} from "./lib/defaults";

const stepLabels: Record<WizardStep, string> = {
  tool: "选择工具",
  install: "安装 Codex",
  provider: "配置 API",
  complete: "教程与启动",
};

const steps: WizardStep[] = ["tool", "install", "provider", "complete"];

const stepProgress: Record<WizardStep, number> = {
  tool: 25,
  install: 50,
  provider: 75,
  complete: 100,
};

export default function App() {
  const [step, setStep] = useState<WizardStep>("tool");
  const [providerForm, setProviderForm] = useState<ProviderFormState>({
    baseUrl: DEFAULT_PROVIDER_BASE_URL,
    apiKey: "",
    protocol: DEFAULT_PROVIDER_PROTOCOL,
    selectedModel: DEFAULT_PROVIDER_MODEL,
  });

  return (
    <MantineProvider defaultColorScheme="light">
      <AppShell className="app-bg">
        <Container size="md" py="xl">
          <Stack gap="lg">
            <Paper className="hero-card" radius="md" p="xl">
              <Group justify="space-between" align="flex-start" gap="md">
                <Box>
                  <Badge color="blue" variant="light">聚合安装</Badge>
                  <Title id="app-title" order={1} mt="sm">SY Codex</Title>
                  <Text c="dimmed" mt="xs">
                    一键安装 Codex，预留 OpenClaw，并完成 SY API 中转配置。适合第一次使用的新手。
                  </Text>
                </Box>
                <Badge size="lg" color="green" variant="filled">GPT-5.5</Badge>
              </Group>
              <Progress value={stepProgress[step]} mt="lg" radius="xl" />
            </Paper>

            <nav className="steps" aria-label="安装步骤">
              {steps.map((item, index) => (
                <span className={item === step ? "step active" : "step"} key={item}>
                  <span className="step-number">{index + 1}</span>
                  <span>{stepLabels[item]}</span>
                </span>
              ))}
            </nav>

            {step === "tool" && <ToolStep />}
            {step === "install" && <InstallStep />}
            {step === "provider" && <ProviderStep form={providerForm} onFormChange={setProviderForm} />}
            {step === "complete" && <CompleteStep providerForm={providerForm} />}

            <Group justify="space-between">
              <Button variant="default" onClick={() => setStep(getPreviousStep(step))} disabled={step === "tool"}>
                上一步
              </Button>
              <Button onClick={() => setStep(getNextStep(step))}>
                {step === "complete" ? "完成" : "下一步"}
              </Button>
            </Group>
          </Stack>
        </Container>
      </AppShell>
    </MantineProvider>
  );
}
