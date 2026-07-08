import { describe, expect, it } from "vitest";
import {
  DEFAULT_PROVIDER_BASE_URL,
  DEFAULT_PROVIDER_MODEL,
  DEFAULT_PROVIDER_PROTOCOL,
  SY_API_SITE_URL
} from "./defaults";

describe("SY defaults", () => {
  it("uses SY API as the beginner default provider", () => {
    expect(DEFAULT_PROVIDER_BASE_URL).toBe("https://www.syapi.vip/v1");
    expect(DEFAULT_PROVIDER_PROTOCOL).toBe("responses");
    expect(DEFAULT_PROVIDER_MODEL).toBe("gpt-5.5");
    expect(SY_API_SITE_URL).toBe("https://www.syapi.vip/");
  });
});
