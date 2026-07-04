import { useState } from "react";
import "./styles.css";
import type { WizardStep } from "./types";
import { getNextStep, getPreviousStep } from "./lib/wizard";
import { ToolStep } from "./components/ToolStep";
import { InstallStep } from "./components/InstallStep";
import { ProviderStep } from "./components/ProviderStep";
import { CompleteStep } from "./components/CompleteStep";

const stepLabels: Record<WizardStep, string> = {
  tool: "选择工具",
  install: "安装 Codex",
  provider: "配置 API",
  complete: "完成",
};

const steps: WizardStep[] = ["tool", "install", "provider", "complete"];

export default function App() {
  const [step, setStep] = useState<WizardStep>("tool");

  return (
    <main className="app-shell">
      <section className="hero" aria-labelledby="app-title">
        <p className="eyebrow">Codex 安装与配置管理器</p>
        <h1 id="app-title">四步完成 Codex 安装、配置和启动</h1>
        <p>适合第一次使用的用户：选择工具、安装 Codex、填写密钥、打开 Codex。</p>
      </section>

      <nav className="steps" aria-label="安装步骤">
        {steps.map((item, index) => (
          <span
            className={item === step ? "step active" : "step"}
            key={item}
            aria-current={item === step ? "step" : undefined}
          >
            <span className="step-number">{index + 1}</span>
            <span>{stepLabels[item]}</span>
          </span>
        ))}
      </nav>

      {step === "tool" && <ToolStep />}
      {step === "install" && <InstallStep />}
      {step === "provider" && <ProviderStep />}
      {step === "complete" && <CompleteStep />}

      <footer className="actions">
        <button onClick={() => setStep(getPreviousStep(step))} disabled={step === "tool"}>
          上一步
        </button>
        <button className="primary" onClick={() => setStep(getNextStep(step))}>
          {step === "complete" ? "完成" : "下一步"}
        </button>
      </footer>
    </main>
  );
}
