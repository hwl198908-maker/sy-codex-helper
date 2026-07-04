import type { WizardStep } from "../types";

const order: WizardStep[] = ["tool", "install", "provider", "complete"];

export function getNextStep(step: WizardStep): WizardStep {
  const index = order.indexOf(step);
  return order[Math.min(index + 1, order.length - 1)];
}

export function getPreviousStep(step: WizardStep): WizardStep {
  const index = order.indexOf(step);
  return order[Math.max(index - 1, 0)];
}
