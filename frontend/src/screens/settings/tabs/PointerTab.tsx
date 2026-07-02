import type { FC } from "react";
import { SliderField } from "../../../components/inputs/SliderField";
import type { TabProps } from "./types";
import { makeUpdater } from "./utils";

export const PointerTab: FC<TabProps> = ({ draft, updateDraft }) => {
  const update = makeUpdater(updateDraft);

  return (
    <div className="space-y-4">
      <SliderField
        label={`Gain (${draft.gainMultiplier.toFixed(1)}x)`}
        min={1}
        max={30}
        step={0.5}
        value={draft.gainMultiplier}
        onChange={(value) => update({ gainMultiplier: value })}
      />

      <SliderField
        label={`Smoothing (${Math.round(draft.smoothing * 100)}%)`}
        min={0}
        max={85}
        step={5}
        value={Math.round(draft.smoothing * 100)}
        onChange={(value) => update({ smoothing: value / 100 })}
      />
    </div>
  );
};
