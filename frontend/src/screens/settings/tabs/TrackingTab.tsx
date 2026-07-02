import type { FC } from "react";
import { ChoiceButton } from "../../../components/inputs/ChoiceButton";
import type { TabProps } from "./types";
import { makeUpdater } from "./utils";

const TEMPLATE_SIZES = [30, 45, 60];

export const TrackingTab: FC<TabProps> = ({ draft, updateDraft }) => {
  const update = makeUpdater(updateDraft);

  return (
    <div className="space-y-4">
      <div>
        <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-zinc-400">Template size</p>
        <div className="flex gap-2">
          {TEMPLATE_SIZES.map((size) => (
            <ChoiceButton
              key={size}
              selected={draft.templateSizePx === size}
              onClick={() => update({ templateSizePx: size })}
            >
              {size}px
            </ChoiceButton>
          ))}
        </div>
      </div>
    </div>
  );
};
