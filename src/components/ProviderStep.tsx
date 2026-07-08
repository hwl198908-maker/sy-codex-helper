import { useMemo, useState } from "react";
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
  SimpleGrid,
  Stack,
  Text,
  TextInput,
  Title
} from "@mantine/core";
import type { ProviderConfig, ProviderFormState, ProviderPresetId, ProviderProtocol } from "../types";
import { buildModelsUrl, parseModelList, validateProviderInput } from "../lib/provider";
import { DEFAULT_PROVIDER_MODEL, SY_API_SITE_URL } from "../lib/defaults";
import { getProviderPreset, PROVIDER_PRESETS } from "../lib/providerPresets";

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
  const preset = getProviderPreset(form.providerPresetId);

  return {
    name: preset.id === "custom" ? "custom" : preset.name,
    baseUrl: form.baseUrl,
    apiKey: form.apiKey,
    protocol: form.protocol,
    defaultModel: form.selectedModel || preset.defaultModel || DEFAULT_PROVIDER_MODEL,
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
  const [modelStatus, setModelStatus] = useState("粘贴 API Key 后，可以一键获取上游模型列表。");
  const [status, setStatus] = useState("API 配置尚未保存。");
  const selectedPreset = getProviderPreset(form.providerPresetId);
  const modelOptions = useMemo(() => {
    const fallback = form.selectedModel || selectedPreset.defaultModel || DEFAULT_PROVIDER_MODEL;
    return models.length > 0 ? models : [fallback];
  }, [form.selectedModel, models, selectedPreset.defaultModel]);

  function updateForm(patch: Partial<ProviderFormState>) {
    onFormChange({ ...form, ...patch });
  }

  function selectPreset(providerPresetId: ProviderPresetId) {
    const preset = getProviderPreset(providerPresetId);
    setModels([]);
    setModelStatus("已切换供应商，粘贴 Key 后可重新获取上游模型。");
    onFormChange({
      ...form,
      providerPresetId,
      baseUrl: preset.baseUrl || form.baseUrl,
      protocol: preset.protocol,
      selectedModel: preset.defaultModel || form.selectedModel
    });
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
      updateForm({ selectedModel: nextModels[0] ?? config.defaultModel ?? DEFAULT_PROVIDER_MODEL });
      setModelStatus(
        nextModels.length > 0
          ? `已获取 ${nextModels.length} 个模型，请选择要写入 Codex 的默认模型。`
          : "上游返回空模型列表，可以手动填写模型后保存。"
      );
    } catch {
      setModels([]);
      setModelStatus("模型列表获取失败，可以手动填写模型后保存。");
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
      setStatus("配置已保存到 Codex。本次保存已自动备份旧配置。");
    } catch (error) {
      setStatus(`保存失败：${error instanceof Error ? error.message : String(error)}`);
    }
  }

  return (
    <Paper className="panel provider-panel" radius="md" p="xl">
      <Stack gap="md">
        <div>
          <Text className="eyebrow">第 3 步</Text>
          <Title order={2}>配置 API 供应商</Title>
          <Text c="dimmed" mt={6}>
            新手默认选择 SY API。选择供应商后会自动填写 Base URL、协议和推荐模型，你只需要粘贴 Key。
          </Text>
        </div>

        <Alert color="blue" variant="light">
          SY API 是 2 折 GPT 模型聚合入口。
          <Anchor href={SY_API_SITE_URL} target="_blank" rel="noreferrer" ml={4}>
            www.syapi.com
          </Anchor>
          ：充值后创建令牌，把 Key 粘贴到这里。
        </Alert>

        <SimpleGrid cols={{ base: 1, sm: 2, md: 4 }} spacing="sm">
          {PROVIDER_PRESETS.map((preset) => (
            <button
              type="button"
              key={preset.id}
              className={preset.id === form.providerPresetId ? "provider-card selected" : "provider-card"}
              onClick={() => selectPreset(preset.id)}
            >
              <span className="provider-icon">{preset.name.slice(0, 1)}</span>
              <Text fw={900}>{preset.name}</Text>
              <Text size="sm" c="dimmed" mt={4}>
                {preset.description}
              </Text>
            </button>
          ))}
        </SimpleGrid>

        <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="sm">
          <TextInput
            label="Base URL"
            description={selectedPreset.editable ? "自定义供应商需要填写接口地址。" : "已根据供应商自动填写。"}
            value={form.baseUrl}
            onChange={(event) => updateForm({ baseUrl: event.currentTarget.value })}
            placeholder="https://www.syapi.com/v1"
            disabled={!selectedPreset.editable}
          />

          <Radio.Group
            label="协议类型"
            value={form.protocol}
            onChange={(value) => updateForm({ protocol: value as ProviderProtocol })}
          >
            <Group mt="xs">
              <Radio value="responses" label="Responses API" disabled={!selectedPreset.editable} />
              <Radio value="chat_completions" label="Chat Completions" disabled={!selectedPreset.editable} />
            </Group>
          </Radio.Group>
        </SimpleGrid>

        <PasswordInput
          label="API Key"
          description="这里只保存到本机 Codex 配置中，不会显示完整 Key。"
          value={form.apiKey}
          onChange={(event) => updateForm({ apiKey: event.currentTarget.value })}
          placeholder="粘贴你创建的 API Key"
        />

        <Select
          label="默认模型"
          description="点击一键获取上游模型后，可以从供应商返回的模型列表中选择。"
          value={form.selectedModel || selectedPreset.defaultModel || DEFAULT_PROVIDER_MODEL}
          onChange={(value) => updateForm({ selectedModel: value || selectedPreset.defaultModel })}
          data={modelOptions}
          searchable
          allowDeselect={false}
        />

        <Alert color="gray" variant="light">{modelStatus}</Alert>

        <Group>
          <Button className="primary-action" onClick={loadModels} loading={isLoadingModels}>
            一键获取上游模型
          </Button>
          <Button className="success-action" onClick={saveProviderConfig}>
            保存 API 配置
          </Button>
        </Group>

        <Alert color="green" variant="light">
          自动识别配置写入位置：Windows 写入 %USERPROFILE%\.codex；Mac 写入 ~/.codex。
        </Alert>

        <Text c="dimmed">{status}</Text>
      </Stack>
    </Paper>
  );
}
