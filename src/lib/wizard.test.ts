import { describe, expect, it } from "vitest";
import { getNextStep, getPreviousStep } from "./wizard";

describe("wizard navigation", () => {
  it("moves forward through the beginner flow", () => {
    expect(getNextStep("tool")).toBe("install");
    expect(getNextStep("install")).toBe("provider");
    expect(getNextStep("provider")).toBe("complete");
    expect(getNextStep("complete")).toBe("complete");
  });

  it("moves backward without leaving the first step", () => {
    expect(getPreviousStep("tool")).toBe("tool");
    expect(getPreviousStep("install")).toBe("tool");
    expect(getPreviousStep("provider")).toBe("install");
    expect(getPreviousStep("complete")).toBe("provider");
  });
});
