import type {FC} from 'react';

type HotkeyFieldProps = {
    label: string;
    description: string;
    value: string;
    onChange: (value: string) => void;
};

const formatHotkeyInput = (value: string): string => value.replace(/\s+/g, '').toUpperCase();

export const HotkeyField: FC<HotkeyFieldProps> = ({label, description, value, onChange}) => (
    <label className="block text-sm">
        <span className="mb-1 block text-xs font-semibold uppercase tracking-wide text-zinc-400">{label}</span>
        <input
            type="text"
            value={(value ?? '').toUpperCase()}
            onChange={event => onChange(formatHotkeyInput(event.target.value))}
            className="w-full rounded-lg border border-zinc-800 bg-zinc-950 px-3 py-2 uppercase"
        />
        <p className="mt-1 text-xs text-zinc-500">{description}</p>
    </label>
);

