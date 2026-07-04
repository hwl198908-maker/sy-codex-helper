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

  it("uses SY API as the beginner provider default", () => {
    const app = readProjectFile("src/App.tsx");
    const defaults = readProjectFile("src/lib/defaults.ts");

    expect(defaults).toContain("https://www.syapi.vip/v1");
    expect(defaults).toContain("gpt-5.5");
    expect(app).toContain("SY Codex（聚合安装）");
    expect(app).toContain("下一步：安装 Codex");
    expect(app).toContain("下一步：配置 API");
    expect(app).toContain("下一步：打开 Codex");
    expect(app).toContain("ProviderFormState");
    expect(app).toContain("setProviderForm");
  });

  it("uses a fixed installer-style desktop window", () => {
    const tauriConfig = readProjectFile("src-tauri/tauri.conf.json");

    expect(tauriConfig).toContain('"title": "SY Codex（聚合安装）"');
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

  it("describes the current completion behavior without live-state claims", () => {
    const completeStep = readProjectFile("src/components/CompleteStep.tsx");

    expect(completeStep).toContain("%USERPROFILE%\\.codex");
    expect(completeStep).toContain("打开 Codex 桌面 App");
    expect(completeStep).toContain("第 4 步 / 最后一步");
    expect(completeStep).toContain("保存 API 设置后");
    expect(completeStep).toContain("自动备份");
    expect(completeStep).toContain("检查更新");
    expect(completeStep).toContain("Codex 中文页面/菜单增强");
    expect(completeStep).toContain("enhancedMenu");
    expect(completeStep).not.toContain("%APPDATA%\\codex-manager");
  });

  it("opens update downloads through the Tauri opener plugin", () => {
    const completeStep = readProjectFile("src/components/CompleteStep.tsx");

    expect(completeStep).toContain("@tauri-apps/plugin-opener");
    expect(completeStep).toContain("openUrl");
    expect(completeStep).not.toContain("window.open");
  });

  it("does not expose provider loading through the Tauri invoke list", () => {
    const tauriLib = readProjectFile("src-tauri/src/lib.rs");

    expect(tauriLib).toContain("save_provider_record");
    expect(tauriLib).not.toContain("load_provider_record");
  });
});
