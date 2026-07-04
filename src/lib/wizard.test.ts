import { describe, expect, it } from "vitest";
import { getNextStep, getPreviousStep } from "./wizard";

describe("wizard navigation", () => {
  it("moves forward through the beginner flow", () => {
    expect(getNextStep("tool")).toBe("install");
    expect(getNextStep("install")).toBe("provider");
    expect(getNextStep("provider")).toBe("style");
    expect(getNextStep("style")).toBe("complete");
    expect(getNextStep("complete")).toBe("feedback");
    expect(getNextStep("feedback")).toBe("feedback");
  });

  it("moves backward without leaving the first step", () => {
    expect(getPreviousStep("tool")).toBe("tool");
    expect(getPreviousStep("install")).toBe("tool");
    expect(getPreviousStep("provider")).toBe("install");
    expect(getPreviousStep("style")).toBe("provider");
    expect(getPreviousStep("complete")).toBe("style");
    expect(getPreviousStep("feedback")).toBe("complete");
  });
});
