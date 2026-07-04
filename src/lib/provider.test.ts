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
});

describe("parseModelList", () => {
  it("reads OpenAI-compatible model ids", () => {
    expect(
      parseModelList({
        data: [{ id: "gpt-4.1" }, { id: "gpt-4.1-mini" }]
      })
    ).toEqual(["gpt-4.1", "gpt-4.1-mini"]);
  });
});
