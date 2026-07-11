import { describe, expect, it } from "vitest";
import {
  DEFAULT_MACOS_ARM64_MIRROR_BASE_URL,
  DEFAULT_MIRROR_BASE_URL,
  DEFAULT_WINDOWS_MIRROR_BASE_URL
} from "./defaults";
import { getProviderPreset, PROVIDER_PRESETS } from "./providerPresets";

describe("provider presets", () => {
  it("keeps the Codex download URL unchanged", () => {
    expect(DEFAULT_WINDOWS_MIRROR_BASE_URL).toBe(
      "https://codexapp.agentsmirror.com/manager/latest/CodexAppManager_x64-setup.exe"
    );
    expect(DEFAULT_MACOS_ARM64_MIRROR_BASE_URL).toBe(
      "https://codexapp.agentsmirror.com/manager/latest/CodexAppManager_aarch64.dmg"
    );
    expect(DEFAULT_MIRROR_BASE_URL).toBe(DEFAULT_WINDOWS_MIRROR_BASE_URL);
  });

  it("uses SY API as the beginner default", () => {
    const preset = getProviderPreset("sy_api");

    expect(preset.baseUrl).toBe("https://www.syapi.vip/v1");
    expect(preset.protocol).toBe("responses");
    expect(preset.defaultModel).toBe("gpt-5.5");
  });

  it("includes official provider options and custom", () => {
    expect(PROVIDER_PRESETS.map((preset) => preset.id)).toEqual([
      "sy_api",
      "openai",
      "deepseek",
      "zhipu",
      "custom"
    ]);
    expect(getProviderPreset("openai").baseUrl).toBe("https://api.openai.com/v1");
    expect(getProviderPreset("openai").protocol).toBe("responses");
    expect(getProviderPreset("deepseek").baseUrl).toBe("https://api.deepseek.com");
    expect(getProviderPreset("zhipu").baseUrl).toBe("https://open.bigmodel.cn/api/coding/paas/v4");
    expect(getProviderPreset("custom").editable).toBe(true);
  });
});
