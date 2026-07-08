import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const readProjectFile = (path) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");

describe("final package guidance", () => {
  it("provides a default mirror address for the install step", () => {
    const installStep = readProjectFile("src/components/InstallStep.tsx");
    const defaults = readProjectFile("src/lib/defaults.ts");

    expect(defaults).toContain("DEFAULT_MIRROR_BASE_URL");
    expect(defaults).toContain("https://codexapp.agentsmirror.com/manager/latest/CodexAppManager_x64-setup.exe");
    expect(installStep).toContain("DEFAULT_MIRROR_BASE_URL");
    expect(installStep).not.toContain('const [mirrorBaseUrl, setMirrorBaseUrl] = useState("");');
  });

  it("uses SY API and the simplified step guide", () => {
    const app = readProjectFile("src/App.tsx");
    const defaults = readProjectFile("src/lib/defaults.ts");

    expect(defaults).toContain("https://www.syapi.vip/v1");
    expect(defaults).toContain("gpt-5.5");
    expect(app).toContain("SY Codex（聚合安装）");
    expect(app).toContain("中文增强");
    expect(app).toContain("意见反馈");
    expect(app).toContain("guide-next-button");
    expect(app).not.toContain("footer-actions");
    expect(app).toContain("ProviderFormState");
    expect(app).toContain("setProviderForm");
  });

  it("keeps style settings immediately before feedback", () => {
    const wizard = readProjectFile("src/lib/wizard.ts");

    expect(wizard).toContain('["tool", "install", "provider", "complete", "style", "feedback"]');
  });

  it("uses a fixed installer-style desktop window", () => {
    const tauriConfig = readProjectFile("src-tauri/tauri.conf.json");

    expect(tauriConfig).toContain('"width": 980');
    expect(tauriConfig).toContain('"height": 720');
    expect(tauriConfig).toContain('"resizable": false');
  });

  it("documents the supported Windows mirror manifest shape", () => {
    const readme = readProjectFile("README.md");

    expect(readme).toContain('"platform": "windows-x64"');
    expect(readme).toContain('"checksumSha256"');
    expect(readme).not.toContain('"platform": "windows"');
    expect(readme).not.toContain('"arch": "x64"');
  });

  it("shows install download progress instead of looking frozen", () => {
    const installStep = readProjectFile("src/components/InstallStep.tsx");

    expect(installStep).toContain("codex-download-progress");
    expect(installStep).toContain("<Progress");
  });

  it("keeps completion and feedback behavior discoverable", () => {
    const completeStep = readProjectFile("src/components/CompleteStep.tsx");
    const feedbackStep = readProjectFile("src/components/FeedbackStep.tsx");

    expect(completeStep).toContain("%USERPROFILE%\\.codex");
    expect(completeStep).toContain("~/.codex");
    expect(completeStep).toContain("enhancedMenu");
    expect(feedbackStep).toContain("submit_feedback");
    expect(feedbackStep).toContain("FEEDBACK_ENDPOINT_URL");
  });

  it("downloads update installers inside the app", () => {
    const completeStep = readProjectFile("src/components/CompleteStep.tsx");
    const tauriLib = readProjectFile("src-tauri/src/lib.rs");

    expect(completeStep).toContain("update-download-progress");
    expect(completeStep).toContain("download_and_install_update");
    expect(completeStep).toContain("<Progress");
    expect(tauriLib).toContain("updater::download_and_install_update");
    expect(completeStep).not.toContain("window.open");
  });

  it("does not expose provider loading through the Tauri invoke list", () => {
    const tauriLib = readProjectFile("src-tauri/src/lib.rs");

    expect(tauriLib).toContain("save_provider_record");
    expect(tauriLib).not.toContain("load_provider_record");
  });
});
