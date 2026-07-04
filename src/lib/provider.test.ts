import { describe, expect, it } from "vitest";
import { parseModelList, validateProviderInput } from "./provider";

describe("validateProviderInput", () => {
  it("accepts a valid OpenAI-compatible provider", () => {
    const result = validateProviderInput({
      name: "默认供应商",
      baseUrl: "https://api.example.com/v1",
      apiKey: "sk-test",
      protocol: "responses",
      userAgent: "CodexManager/1.0"
    });

    expect(result.ok).toBe(true);
  });

  it("rejects an invalid Base URL", () => {
    const result = validateProviderInput({
      name: "默认供应商",
      baseUrl: "not a url",
      apiKey: "sk-test",
      protocol: "responses",
      userAgent: "CodexManager/1.0"
    });

    expect(result).toEqual({ ok: false, message: "Base URL（接口地址）格式不正确" });
  });

  it("rejects a blank API Key", () => {
    const result = validateProviderInput({
      name: "默认供应商",
      baseUrl: "https://api.example.com/v1",
      apiKey: "   ",
      protocol: "responses",
      userAgent: "CodexManager/1.0"
    });

    expect(result).toEqual({ ok: false, message: "API Key（密钥）不能为空" });
  });

  it("accepts a short non-blank API Key", () => {
    const result = validateProviderInput({
      name: "默认供应商",
      baseUrl: "https://api.example.com/v1",
      apiKey: "ab",
      protocol: "responses",
      userAgent: "CodexManager/1.0"
    });

    expect(result.ok).toBe(true);
  });
});

describe("parseModelList", () => {
  it("reads OpenAI-compatible model ids", () => {
    expect(
      parseModelList({
        data: [{ id: "gpt-4.1" }, { id: "gpt-4.1-mini" }]
      })
    ).toEqual(["gpt-4.1", "gpt-4.1-mini"]);
  });

  it("returns valid ids from malformed model list inputs", () => {
    expect(parseModelList({})).toEqual([]);
    expect(parseModelList({ data: [{ id: 123 }, null, { id: "gpt-ok" }] })).toEqual(["gpt-ok"]);
  });
});
