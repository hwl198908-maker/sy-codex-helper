export type UpdateManifest = {
  version: string;
  downloadUrl: string;
  notes?: string;
};

export function parseUpdateManifest(value: unknown): UpdateManifest {
  if (!value || typeof value !== "object") {
    throw new Error("更新清单格式不正确");
  }

  const manifest = value as Record<string, unknown>;
  if (typeof manifest.version !== "string" || typeof manifest.downloadUrl !== "string") {
    throw new Error("更新清单缺少版本号或下载地址");
  }

  return {
    version: manifest.version,
    downloadUrl: manifest.downloadUrl,
    notes: typeof manifest.notes === "string" ? manifest.notes : undefined
  };
}

export function isNewerVersion(current: string, latest: string): boolean {
  const currentParts = current.split(".").map((part) => Number.parseInt(part, 10) || 0);
  const latestParts = latest.split(".").map((part) => Number.parseInt(part, 10) || 0);
  const length = Math.max(currentParts.length, latestParts.length);

  for (let index = 0; index < length; index += 1) {
    const currentPart = currentParts[index] ?? 0;
    const latestPart = latestParts[index] ?? 0;
    if (latestPart > currentPart) {
      return true;
    }
    if (latestPart < currentPart) {
      return false;
    }
  }

  return false;
}
