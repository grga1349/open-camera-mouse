import type { FC } from "react";
import { SliderField } from "../../../components/inputs/SliderField";
import type { TabProps } from "./types";
import { makeUpdater } from "./utils";

export const ClickingTab: FC<TabProps> = ({ draft, updateDraft }) => {
  const clicking = draft.clicking;
  const updateClicking = makeUpdater(updateDraft, "clicking");

  return (
    <div className="space-y-4">
      <SliderField
        label={`Dwell time (${clicking.dwellTimeMs} ms)`}
        min={200}
        max={1500}
        step={50}
        value={clicking.dwellTimeMs}
        onChange={(value) => updateClicking({ dwellTimeMs: value })}
      />

      <SliderField
        label={`Dwell radius (${clicking.dwellRadiusPx}px)`}
        min={5}
        max={80}
        step={5}
        value={clicking.dwellRadiusPx}
        onChange={(value) => updateClicking({ dwellRadiusPx: value })}
      />
    </div>
  );
};
