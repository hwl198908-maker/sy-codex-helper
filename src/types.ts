export type ToolId = "codex" | "openclaw";

export type WizardStep = "tool" | "install" | "provider" | "style" | "complete" | "feedback";

export type InstallStatus = {
  installed: boolean;
  installed_version?: string;
  available_version?: string;
  message: string;
};

export type MirrorManifest = {
  tools: MirrorToolPackage[];
};

export type MirrorToolPackage = {
  toolId: string;
  version: string;
  platform: string;
  packageUrl: string;
  checksumSha256: string;
  releaseNotes?: string;
};

export type ProviderProtocol = "responses" | "chat_completions";

export type ProviderPresetId = "sy_api" | "deepseek" | "zhipu" | "custom";

export type ProviderConfig = {
  name: string;
  baseUrl: string;
  apiKey: string;
  protocol: ProviderProtocol;
  userAgent: string;
  defaultModel?: string;
};

export type ProviderFormState = {
  providerPresetId: ProviderPresetId;
  baseUrl: string;
  apiKey: string;
  protocol: ProviderProtocol;
  selectedModel: string;
};
