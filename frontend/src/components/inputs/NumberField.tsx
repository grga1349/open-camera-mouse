import type {FC} from 'react';

type NumberFieldProps = {
    label: string;
    value: number;
    min: number;
    max: number;
    step: number;
    onChange: (value: number) => void;
};

export const NumberField: FC<NumberFieldProps> = ({label, value, min, max, step, onChange}) => (
    <label className="block text-sm">
        <span className="mb-1 block text-xs font-semibold uppercase tracking-wide text-zinc-400">{label}</span>
        <input
            type="number"
            value={value}
            min={min}
            max={max}
            step={step}
            onChange={event => onChange(parseFloat(event.target.value))}
            className="w-full rounded-lg border border-zinc-800 bg-zinc-950 px-3 py-2"
        />
    </label>
);

