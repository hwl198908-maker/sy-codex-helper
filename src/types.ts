export type ToolId = "codex" | "openclaw";

export type WizardStep = "tool" | "install" | "provider" | "complete";

export type InstallStatus = {
  installed: boolean;
  installedVersion?: string;
  availableVersion?: string;
  message: string;
};

export type ProviderProtocol = "responses" | "chat_completions";

export type ProviderConfig = {
  name: string;
  baseUrl: string;
  apiKey: string;
  protocol: ProviderProtocol;
  userAgent: string;
  defaultModel?: string;
};
