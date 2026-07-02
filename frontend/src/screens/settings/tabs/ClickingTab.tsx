import type { FC } from "react";
import { SliderField } from "../../../components/inputs/SliderField";
import type { TabProps } from "./types";
import { makeUpdater } from "./utils";

export const ClickingTab: FC<TabProps> = ({ draft, updateDraft }) => {
  const update = makeUpdater(updateDraft);

  return (
    <div className="space-y-4">
      <SliderField
        label={`Dwell time (${draft.dwellTimeMs} ms)`}
        min={200}
        max={1500}
        step={50}
        value={draft.dwellTimeMs}
        onChange={(value) => update({ dwellTimeMs: value })}
      />
    </div>
  );
};
