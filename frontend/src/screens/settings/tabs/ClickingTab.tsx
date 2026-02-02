import type { FC } from "react";
import { SliderField } from "../../../components/inputs/SliderField";
import type { TabProps } from "./types";

export const ClickingTab: FC<TabProps> = ({ draft, updateDraft }) => {
  const clicking = draft.clicking;
  const updateClicking = (changes: Partial<typeof clicking>) => {
    updateDraft((current) => ({
      ...current,
      clicking: {
        ...current.clicking,
        ...changes,
      },
    }));
  };

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
