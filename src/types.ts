export type ToolId = "codex" | "openclaw";

export type WizardStep = "tool" | "install" | "provider" | "complete";

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
  tool_id: string;
  version: string;
  platform: string;
  package_url: string;
  checksum_sha256: string;
  release_notes: string;
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
