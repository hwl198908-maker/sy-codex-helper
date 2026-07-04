export type MirrorToolPackage = {
  toolId: "codex" | "openclaw";
  version: string;
  platform: "windows-x64" | "macos-arm64" | "macos-x64";
  packageUrl: string;
  checksumSha256: string;
  releaseNotes?: string;
};

export type MirrorManifest = {
  tools: MirrorToolPackage[];
};

function requireString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`镜像清单缺少 ${field}`);
  }
  return value;
}

function isSupportedToolId(value: string): value is MirrorToolPackage["toolId"] {
  return value === "codex" || value === "openclaw";
}

function isSupportedPlatform(value: string): value is MirrorToolPackage["platform"] {
  return value === "windows-x64" || value === "macos-arm64" || value === "macos-x64";
}

export function parseMirrorManifest(input: unknown): MirrorManifest {
  if (!input || typeof input !== "object" || !Array.isArray((input as { tools?: unknown }).tools)) {
    throw new Error("镜像清单格式不正确");
  }

  const tools = (input as { tools: unknown[] }).tools.map((item) => {
    if (!item || typeof item !== "object") {
      throw new Error("镜像清单工具项格式不正确");
    }

    const record = item as Record<string, unknown>;
    const toolId = requireString(record.toolId, "toolId");
    const platform = requireString(record.platform, "platform");

    if (!isSupportedToolId(toolId)) {
      throw new Error("镜像清单 toolId 不支持");
    }

    if (!isSupportedPlatform(platform)) {
      throw new Error("镜像清单 platform 不支持");
    }

    return {
      toolId,
      version: requireString(record.version, "version"),
      platform,
      packageUrl: requireString(record.packageUrl, "packageUrl"),
      checksumSha256: requireString(record.checksumSha256, "checksumSha256"),
      releaseNotes: typeof record.releaseNotes === "string" ? record.releaseNotes : undefined
    };
  });

  return { tools };
}
