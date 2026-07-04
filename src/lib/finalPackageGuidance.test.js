import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const readProjectFile = (path) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");

describe("final package guidance", () => {
  it("provides a default mirror address for the install step", () => {
    const installStep = readProjectFile("src/components/InstallStep.tsx");
    const defaults = readProjectFile("src/lib/defaults.ts");

    expect(defaults).toContain("DEFAULT_MIRROR_BASE_URL");
    expect(defaults).toContain("https://codexapp.agentsmirror.com/latest/win");
    expect(installStep).toContain("DEFAULT_MIRROR_BASE_URL");
    expect(installStep).not.toContain('const [mirrorBaseUrl, setMirrorBaseUrl] = useState("");');
  });

  it("documents the supported Windows mirror manifest shape", () => {
    const readme = readProjectFile("README.md");

    expect(readme).toContain('"platform": "windows-x64"');
    expect(readme).toContain('"checksumSha256"');
    expect(readme).not.toContain('"platform": "windows"');
    expect(readme).not.toContain('"arch": "x64"');
  });

  it("describes the current completion behavior without live-state claims", () => {
    const completeStep = readProjectFile("src/components/CompleteStep.tsx");

    expect(completeStep).toContain("%USERPROFILE%\\.codex");
    expect(completeStep).toContain("打开 Codex");
    expect(completeStep).toContain("保存 API 设置后");
    expect(completeStep).toContain("自动备份");
    expect(completeStep).not.toContain("等待接入保存命令");
    expect(completeStep).not.toContain("%APPDATA%\\codex-manager");
    expect(completeStep).not.toContain("真实安装、配置和模型状态");
  });

  it("does not expose provider loading through the Tauri invoke list", () => {
    const tauriLib = readProjectFile("src-tauri/src/lib.rs");

    expect(tauriLib).toContain("save_provider_record");
    expect(tauriLib).not.toContain("load_provider_record");
  });
});
