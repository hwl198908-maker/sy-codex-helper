import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ProviderConfig, ProviderProtocol } from "../types";
import { buildModelsUrl, parseModelList, validateProviderInput } from "../lib/provider";

const userAgent = "CodexManager/1.0";

type CommandProvider = {
  name: string;
  base_url: string;
  api_key: string;
  protocol: ProviderProtocol;
  default_model?: string;
  user_agent: string;
};

function buildProviderConfig(
  baseUrl: string,
  apiKey: string,
  protocol: ProviderProtocol,
  defaultModel?: string
): ProviderConfig {
  return {
    name: "Custom Provider",
    baseUrl,
    apiKey,
    protocol,
    defaultModel,
    userAgent
  };
}

function toCommandProvider(config: ProviderConfig): CommandProvider {
  return {
    name: config.name,
    base_url: config.baseUrl,
    api_key: config.apiKey,
    // The Rust writer currently persists Codex wire_api as responses.
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

export function ProviderStep() {
  const [baseUrl, setBaseUrl] = useState("https://api.openai.com/v1");
  const [apiKey, setApiKey] = useState("");
  const [protocol, setProtocol] = useState<ProviderProtocol>("responses");
  const [models, setModels] = useState<string[]>([]);
  const [selectedModel, setSelectedModel] = useState("");
  const [isLoadingModels, setIsLoadingModels] = useState(false);
  const [modelStatus, setModelStatus] = useState("模型状态：尚未获取模型列表。");
  const [status, setStatus] = useState("本地状态：配置尚未保存。");

  function currentConfig(): ProviderConfig {
    return buildProviderConfig(baseUrl, apiKey, protocol, selectedModel || undefined);
  }

  async function loadModels() {
    const config = currentConfig();
    const validation = validateProviderInput(config);
    if (!validation.ok) {
      setModelStatus(`模型状态：${validation.message}`);
      return;
    }

    setIsLoadingModels(true);
    setModelStatus("模型状态：正在获取模型列表...");
    try {
      const nextModels = await fetchModels(config.baseUrl, config.apiKey);
      setModels(nextModels);
      setSelectedModel(nextModels[0] ?? "");
      setModelStatus(
        nextModels.length > 0
          ? `模型状态：已获取 ${nextModels.length} 个模型。`
          : "模型状态：接口返回了空模型列表，仍可保存配置。"
      );
    } catch {
      setModels([]);
      setSelectedModel("");
      setModelStatus("模型状态：模型列表获取失败，仍可保存配置；请确认 Base URL 和 API Key 后稍后重试。");
    } finally {
      setIsLoadingModels(false);
    }
  }

  async function saveProviderConfig() {
    const config = currentConfig();
    const validation = validateProviderInput(config);
    if (!validation.ok) {
      setStatus(`本地状态：${validation.message}`);
      return;
    }

    try {
      await invoke("write_provider_config", {
        provider: toCommandProvider(config)
      });
      setStatus(
        modelStatus.includes("获取失败")
          ? "本地状态：配置已保存。注意：模型列表获取失败，可稍后重试。"
          : "本地状态：配置已保存。"
      );
    } catch (error) {
      setStatus(`本地状态：保存失败：${error instanceof Error ? error.message : String(error)}`);
    }
  }

  return (
    <section className="panel" aria-labelledby="provider-title">
      <div className="section-heading">
        <p className="eyebrow">第 3 步</p>
        <h2 id="provider-title">配置 API</h2>
        <p>填写服务地址和密钥，获取可用模型后保存到本机 Codex 配置。</p>
      </div>

      <div className="form-grid">
        <label className="field">
          <span>Base URL（接口地址）</span>
          <input
            value={baseUrl}
            onChange={(event) => setBaseUrl(event.currentTarget.value)}
            placeholder="例如：https://api.openai.com/v1"
          />
        </label>

        <label className="field">
          <span>API Key（密钥）</span>
          <input
            type="password"
            value={apiKey}
            onChange={(event) => setApiKey(event.currentTarget.value)}
            placeholder="粘贴你的 API Key"
          />
        </label>
      </div>

      <fieldset className="choice-group">
        <legend>协议类型</legend>
        <label>
          <input
            type="radio"
            name="protocol"
            checked={protocol === "responses"}
            onChange={() => setProtocol("responses")}
          />
          Responses API（默认）
        </label>
        <label>
          <input
            type="radio"
            name="protocol"
            checked={protocol === "chat_completions"}
            onChange={() => setProtocol("chat_completions")}
          />
          Chat Completions
        </label>
      </fieldset>

      <label className="field">
        <span>默认模型</span>
        {models.length > 0 ? (
          <select value={selectedModel} onChange={(event) => setSelectedModel(event.currentTarget.value)}>
            {models.map((model) => (
              <option key={model} value={model}>
                {model}
              </option>
            ))}
          </select>
        ) : (
          <select value="" disabled>
            <option value="">获取模型列表后可选择默认模型</option>
          </select>
        )}
      </label>

      <div className="status-box" role="status">
        {modelStatus}
      </div>

      <div className="button-row">
        <button type="button" onClick={loadModels} disabled={isLoadingModels}>
          {isLoadingModels ? "正在获取模型..." : "获取模型列表"}
        </button>
        <button className="primary inline-action" type="button" onClick={saveProviderConfig}>
          保存配置
        </button>
      </div>
      <p className="local-status">{status}</p>
    </section>
  );
}
