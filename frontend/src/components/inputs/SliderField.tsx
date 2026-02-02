import type { FC } from "react";
import { cn } from "../../lib/cn";

type SliderFieldProps = {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  disabled?: boolean;
  onChange: (value: number) => void;
};

export const SliderField: FC<SliderFieldProps> = ({ label, value, min, max, step, disabled, onChange }) => (
  <label className="block text-sm">
    <span className="mb-1 block text-xs font-semibold uppercase tracking-wide text-zinc-400">{label}</span>
    <input
      type="range"
      min={min}
      max={max}
      step={step}
      value={value}
      disabled={disabled}
      onChange={(event) => onChange(parseFloat(event.target.value))}
      className={cn("slider-input", disabled && "cursor-not-allowed opacity-50")}
    />
  </label>
);
