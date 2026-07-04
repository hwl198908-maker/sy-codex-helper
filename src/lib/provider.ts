import type { ProviderConfig } from "../types";

type ValidationResult = { ok: true } | { ok: false; message: string };

export function validateProviderInput(config: ProviderConfig): ValidationResult {
  try {
    const url = new URL(config.baseUrl);
    if (url.protocol !== "http:" && url.protocol !== "https:") {
      return { ok: false, message: "Base URL（接口地址）必须以 http 或 https 开头" };
    }
  } catch {
    return { ok: false, message: "Base URL（接口地址）格式不正确" };
  }

  if (config.apiKey.trim().length === 0) {
    return { ok: false, message: "API Key（密钥）不能为空" };
  }

  return { ok: true };
}

export function parseModelList(input: unknown): string[] {
  if (!input || typeof input !== "object" || !Array.isArray((input as { data?: unknown }).data)) {
    return [];
  }

  return (input as { data: unknown[] }).data
    .map((item) => {
      if (!item || typeof item !== "object") {
        return "";
      }
      const id = (item as { id?: unknown }).id;
      return typeof id === "string" ? id : "";
    })
    .filter(Boolean);
}
