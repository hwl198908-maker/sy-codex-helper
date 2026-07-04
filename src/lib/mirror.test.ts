import { describe, expect, it } from "vitest";
import { parseMirrorManifest } from "./mirror";

describe("parseMirrorManifest", () => {
  it("accepts a Windows Codex package manifest", () => {
    const manifest = parseMirrorManifest({
      tools: [
        {
          toolId: "codex",
          version: "1.2.3",
          platform: "windows-x64",
          packageUrl: "https://mirror.example/codex-1.2.3.exe",
          checksumSha256: "a".repeat(64),
          releaseNotes: "Stable release"
        }
      ]
    });

    expect(manifest.tools[0].toolId).toBe("codex");
    expect(manifest.tools[0].platform).toBe("windows-x64");
  });

  it("rejects a missing checksum", () => {
    expect(() =>
      parseMirrorManifest({
        tools: [
          {
            toolId: "codex",
            version: "1.2.3",
            platform: "windows-x64",
            packageUrl: "https://mirror.example/codex.exe"
          }
        ]
      })
    ).toThrow("镜像清单缺少 checksumSha256");
  });
});
