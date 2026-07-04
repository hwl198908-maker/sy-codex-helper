import { useState } from "react";
import type { ProviderProtocol } from "../types";

export function ProviderStep() {
  const [baseUrl, setBaseUrl] = useState("https://api.openai.com/v1");
  const [apiKey, setApiKey] = useState("");
  const [protocol, setProtocol] = useState<ProviderProtocol>("responses");
  const [status, setStatus] = useState("本地状态：配置尚未保存。");

  function saveLocalDraft() {
    setStatus("本地状态：已记录本页填写内容，联网保存会在后续版本接入。");
  }

  return (
    <section className="panel" aria-labelledby="provider-title">
      <div className="section-heading">
        <p className="eyebrow">第 3 步</p>
        <h2 id="provider-title">配置 API</h2>
        <p>填写服务地址和密钥。当前页面只保存本地提示，不会发起网络请求。</p>
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
        <select defaultValue="">
          <option value="" disabled>
            保存后将在这里加载可用模型
          </option>
        </select>
      </label>

      <div className="status-box" role="status">
        模型状态：尚未联网获取模型列表。
      </div>

      <button className="primary inline-action" type="button" onClick={saveLocalDraft}>
        保存配置
      </button>
      <p className="local-status">{status}</p>
    </section>
  );
}
