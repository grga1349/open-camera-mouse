import type { Params } from "../../../types/params";

export type TabProps = {
  draft: Params;
  updateDraft: (updater: (current: Params) => Params) => void;
};
