import type { AllParams } from "../../../types/params";
import type { TabProps } from "./types";

export function makeUpdater<K extends keyof AllParams>(
  updateDraft: TabProps["updateDraft"],
  key: K,
) {
  return (changes: Partial<AllParams[K]>) =>
    updateDraft((curr) => ({ ...curr, [key]: { ...curr[key], ...changes } }));
}
