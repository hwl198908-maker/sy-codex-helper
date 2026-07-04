import { describe, expect, it } from "vitest";
import { isNewerVersion, parseUpdateManifest } from "./updates";

describe("parseUpdateManifest", () => {
  it("reads a valid update manifest", () => {
    expect(
      parseUpdateManifest({
        version: "0.1.1",
        downloadUrl: "https://example.com/SY-Codex.exe",
        sha256: "abc123",
        notes: "界面优化"
      })
    ).toEqual({
      version: "0.1.1",
      downloadUrl: "https://example.com/SY-Codex.exe",
      sha256: "abc123",
      notes: "界面优化"
    });
  });

  it("rejects invalid manifests", () => {
    expect(() => parseUpdateManifest({ version: "0.1.1" })).toThrow("更新清单缺少版本号或下载地址");
  });
});

describe("isNewerVersion", () => {
  it("compares semantic versions", () => {
    expect(isNewerVersion("0.1.0", "0.1.1")).toBe(true);
    expect(isNewerVersion("0.1.1", "0.1.1")).toBe(false);
    expect(isNewerVersion("0.2.0", "0.1.9")).toBe(false);
  });
});
