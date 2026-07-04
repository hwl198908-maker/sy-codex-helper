import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Alert,
  Anchor,
  Button,
  Group,
  Paper,
  PasswordInput,
  Radio,
  Select,
  Stack,
  Text,
  TextInput,
  Title
} from "@mantine/core";
import type { ProviderConfig, ProviderFormState, ProviderProtocol } from "../types";
import { buildModelsUrl, parseModelList, validateProviderInput } from "../lib/provider";
import { DEFAULT_PROVIDER_MODEL, SY_API_SITE_URL } from "../lib/defaults";

const userAgent = "CodexManager/1.0";

type CommandProvider = {
  name: string;
  base_url: string;
  api_key: string;
  protocol: ProviderProtocol;
  default_model?: string;
  user_agent: string;
};

type ProviderStepProps = {
  form: ProviderFormState;
  onFormChange: (form: ProviderFormState) => void;
};

function buildProviderConfig(form: ProviderFormState): ProviderConfig {
  return {
    name: "SY API",
    baseUrl: form.baseUrl,
    apiKey: form.apiKey,
    protocol: form.protocol,
    defaultModel: form.selectedModel || DEFAULT_PROVIDER_MODEL,
    userAgent
  };
}

function toCommandProvider(config: ProviderConfig): CommandProvider {
  return {
    name: config.name,
    base_url: config.baseUrl,
    api_key: config.apiKey,
    protocol: config.protocol,
    default_model: config.defaultModel,
    user_agent: config.userAgent
  };
}

function buildFetchHeaders(apiKey: string, includeUserAgent: boolean): Headers {
  const headers = new Headers({
    Authorization: `Bearer ${apiKey}`
  });

  if (includeUserAgent) {
    try {
      headers.set("User-Agent", userAgent);
    } catch {
      // Browser fetch may block User-Agent. The request should still proceed.
    }
  }

  return headers;
}

async function fetchModels(baseUrl: string, apiKey: string): Promise<string[]> {
  const url = buildModelsUrl(baseUrl);

  try {
    const response = await fetch(url, {
      headers: buildFetchHeaders(apiKey, true)
    });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return parseModelList(await response.json());
  } catch (error) {
    if (error instanceof TypeError) {
      const response = await fetch(url, {
        headers: buildFetchHeaders(apiKey, false)
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      return parseModelList(await response.json());
    }
    throw error;
  }
}

export function ProviderStep({ form, onFormChange }: ProviderStepProps) {
  const [models, setModels] = useState<string[]>([]);
  const [isLoadingModels, setIsLoadingModels] = useState(false);
  const [modelStatus, setModelStatus] = useState("还没有获取模型列表。可以先保存配置，稍后再回来获取。");
  const [status, setStatus] = useState("API 配置尚未保存。");

  function updateForm(patch: Partial<ProviderFormState>) {
    onFormChange({ ...form, ...patch });
  }

  async function loadModels() {
    const config = buildProviderConfig(form);
    const validation = validateProviderInput(config);
    if (!validation.ok) {
      setModelStatus(validation.message);
      return;
    }

    setIsLoadingModels(true);
    setModelStatus("正在连接上游并读取模型列表...");
    try {
      const nextModels = await fetchModels(config.baseUrl, config.apiKey);
      setModels(nextModels);
      updateForm({ selectedModel: nextModels[0] ?? DEFAULT_PROVIDER_MODEL });
      setModelStatus(nextModels.length > 0 ? `已获取 ${nextModels.length} 个模型。` : "接口返回空模型列表，仍可保存配置。");
    } catch {
      setModels([]);
      setModelStatus("模型列表获取失败。请确认 Base URL 和 API Key，或直接保存后在 Codex 中测试。");
    } finally {
      setIsLoadingModels(false);
    }
  }

  async function saveProviderConfig() {
    const config = buildProviderConfig(form);
    const validation = validateProviderInput(config);
    if (!validation.ok) {
      setStatus(validation.message);
      return;
    }

    try {
      await invoke("save_provider_record", {
        provider: toCommandProvider(config)
      });
      await invoke("write_provider_config", {
        provider: toCommandProvider(config)
      });
      setStatus("配置已保存到本机 Codex。返回上一步再回来也不会清空。");
    } catch (error) {
      setStatus(`保存失败：${error instanceof Error ? error.message : String(error)}`);
    }
  }

  return (
    <Paper className="panel" radius="md" p="xl">
      <Stack gap="md">
        <div>
          <Text className="eyebrow">第 3 步</Text>
          <Title order={2}>配置 API</Title>
          <Text c="dimmed" mt={6}>
            默认使用 SY API 中转。充值后创建令牌，把 Key 填到这里，然后保存即可。
          </Text>
        </div>

        <Alert color="blue" variant="light">
          GPT-5.5 中转 API 入口：
          <Anchor href={SY_API_SITE_URL} target="_blank" rel="noreferrer" ml={4}>
            www.syapi.vip
          </Anchor>
          。客服联系方式：weixxxnb
        </Alert>

        <TextInput
          label="Base URL"
          description="默认地址已经填好，一般不需要修改。"
          value={form.baseUrl}
          onChange={(event) => updateForm({ baseUrl: event.currentTarget.value })}
          placeholder="https://www.syapi.vip/v1"
        />

        <PasswordInput
          label="API Key"
          description="在 SY API 充值后创建令牌，再复制到这里。"
          value={form.apiKey}
          onChange={(event) => updateForm({ apiKey: event.currentTarget.value })}
          placeholder="粘贴你的 API Key"
        />

        <Radio.Group
          label="协议类型"
          value={form.protocol}
          onChange={(value) => updateForm({ protocol: value as ProviderProtocol })}
        >
          <Group mt="xs">
            <Radio value="responses" label="Responses API（推荐）" />
            <Radio value="chat_completions" label="Chat Completions" />
          </Group>
        </Radio.Group>

        <Select
          label="默认模型"
          description="如果没有获取模型列表，会默认保存 gpt-5.5。"
          value={form.selectedModel || DEFAULT_PROVIDER_MODEL}
          onChange={(value) => updateForm({ selectedModel: value || DEFAULT_PROVIDER_MODEL })}
          data={models.length > 0 ? models : [DEFAULT_PROVIDER_MODEL]}
          searchable
        />

        <Alert color="gray" variant="light">{modelStatus}</Alert>

        <Group>
          <Button variant="default" onClick={loadModels} loading={isLoadingModels}>
            获取模型列表
          </Button>
          <Button onClick={saveProviderConfig}>
            保存 API 配置
          </Button>
        </Group>

        <Text c="dimmed">{status}</Text>
      </Stack>
    </Paper>
  );
}
