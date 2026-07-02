import type { Params } from "../../../types/params";
import type { TabProps } from "./types";

export function makeUpdater(updateDraft: TabProps["updateDraft"]) {
  return (changes: Partial<Params>) =>
    updateDraft((curr) => ({ ...curr, ...changes }));
}
