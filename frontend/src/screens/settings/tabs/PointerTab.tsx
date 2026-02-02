import type { FC } from "react";
import { Button } from "../../../components/Button";
import { NumberField } from "../../../components/inputs/NumberField";
import { SliderField } from "../../../components/inputs/SliderField";
import type { TabProps } from "./types";

const clampSensitivity = (value: number) => Math.min(100, Math.max(1, value));
const gainFromSensitivity = (value: number) => {
  const clamped = clampSensitivity(value);
  const normalized = (clamped - 1) / 99;
  const baseGain = 1.2 + normalized * (5 - 1.2);
  return baseGain * 3;
};
const smoothingFromSensitivity = (value: number) => {
  const clamped = clampSensitivity(value);
  const normalized = (clamped - 1) / 99;
  return 0.35 + normalized * (0.15 - 0.35);
};

export const PointerTab: FC<TabProps> = ({ draft, updateDraft }) => {
  const pointer = draft.pointer;
  const updatePointer = (changes: Partial<typeof pointer>) => {
    updateDraft((current) => ({
      ...current,
      pointer: {
        ...current.pointer,
        ...changes,
      },
    }));
  };

  const autoGain = gainFromSensitivity(pointer.sensitivity);
  const autoSmoothing = smoothingFromSensitivity(pointer.sensitivity);
  const autoAdvancedDefaults = { gainX: autoGain, gainY: autoGain, smoothing: autoSmoothing };

  const updateAdvanced = (changes: { gainX?: number; gainY?: number; smoothing?: number } | null) => {
    updatePointer({ advanced: changes ? { ...(pointer.advanced ?? autoAdvancedDefaults), ...changes } : null });
  };

  const advanced = pointer.advanced;

  return (
    <div className="space-y-4">
      <SliderField
        label={`Sensitivity (${pointer.sensitivity})`}
        min={1}
        max={100}
        step={1}
        value={pointer.sensitivity}
        onChange={(value) => updatePointer({ sensitivity: value })}
      />

      <SliderField
        label={`Deadzone (${pointer.deadzonePx}px)`}
        min={0}
        max={20}
        step={1}
        value={pointer.deadzonePx}
        onChange={(value) => updatePointer({ deadzonePx: value })}
      />

      <SliderField
        label={`Max speed (${pointer.maxSpeedPx}px)`}
        min={10}
        max={60}
        step={1}
        value={pointer.maxSpeedPx}
        onChange={(value) => updatePointer({ maxSpeedPx: value })}
      />

      <div className="rounded-2xl border border-zinc-800 p-3">
        <div className="mb-2 flex items-center justify-between text-sm">
          <p className="font-semibold text-zinc-200">Advanced gain</p>
          <Button variant="ghost" onClick={() => updateAdvanced(advanced ? null : autoAdvancedDefaults)}>
            {advanced ? "Disable" : "Enable"}
          </Button>
        </div>
        {advanced ? (
          <div className="space-y-3 text-sm">
            <NumberField
              label="Gain X"
              value={advanced.gainX}
              min={0.5}
              max={18}
              step={0.1}
              onChange={(value) => updateAdvanced({ gainX: value })}
            />
            <NumberField
              label="Gain Y"
              value={advanced.gainY}
              min={0.5}
              max={18}
              step={0.1}
              onChange={(value) => updateAdvanced({ gainY: value })}
            />
            <NumberField
              label="Smoothing"
              value={advanced.smoothing}
              min={0.05}
              max={0.9}
              step={0.05}
              onChange={(value) => updateAdvanced({ smoothing: value })}
            />
          </div>
        ) : (
          <p className="text-xs text-zinc-400">Enable to override automatically mapped gain + smoothing.</p>
        )}
      </div>
    </div>
  );
};
