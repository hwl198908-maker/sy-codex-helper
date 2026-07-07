import type { ProviderPresetId, ProviderProtocol } from "../types";
import { DEFAULT_PROVIDER_BASE_URL, DEFAULT_PROVIDER_MODEL, DEFAULT_PROVIDER_PROTOCOL } from "./defaults";

export type ProviderPreset = {
  id: ProviderPresetId;
  name: string;
  description: string;
  baseUrl: string;
  protocol: ProviderProtocol;
  defaultModel: string;
  editable: boolean;
};

export const PROVIDER_PRESETS: ProviderPreset[] = [
  {
    id: "sy_api",
    name: "SY API",
    description: "2 折 GPT 模型聚合，推荐新手使用",
    baseUrl: DEFAULT_PROVIDER_BASE_URL,
    protocol: DEFAULT_PROVIDER_PROTOCOL,
    defaultModel: DEFAULT_PROVIDER_MODEL,
    editable: false
  },
  {
    id: "deepseek",
    name: "DeepSeek 官方",
    description: "DeepSeek 官方 OpenAI 兼容接口",
    baseUrl: "https://api.deepseek.com",
    protocol: "chat_completions",
    defaultModel: "deepseek-chat",
    editable: false
  },
  {
    id: "zhipu",
    name: "智谱官方",
    description: "智谱官方 OpenAI 兼容接口",
    baseUrl: "https://open.bigmodel.cn/api/coding/paas/v4",
    protocol: "chat_completions",
    defaultModel: "glm-4.5",
    editable: false
  },
  {
    id: "custom",
    name: "自定义",
    description: "手动填写 Base URL、协议和模型",
    baseUrl: "",
    protocol: "responses",
    defaultModel: DEFAULT_PROVIDER_MODEL,
    editable: true
  }
];

export function getProviderPreset(id: ProviderPresetId): ProviderPreset {
  return PROVIDER_PRESETS.find((preset) => preset.id === id) ?? PROVIDER_PRESETS[0];
}
