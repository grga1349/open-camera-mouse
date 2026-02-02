import type { AllParams } from "../../../types/params";

export type TabProps = {
  draft: AllParams;
  updateDraft: (updater: (current: AllParams) => AllParams) => void;
};
