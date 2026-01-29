import {useState, type FC, type ReactNode} from 'react';
import {Button} from '../components/Button';
import {useSettingsDraft} from '../state/useSettingsDraft';
import type {AllParams} from '../types/params';

const tabs = ['Tracking', 'Pointer', 'Clicking', 'General'] as const;

type SettingsScreenProps = {
    onSave: (params: AllParams) => void | Promise<void>;
    onCancel: () => void;
};

export const SettingsScreen: FC<SettingsScreenProps> = ({onSave, onCancel}) => {
    const [activeTab, setActiveTab] = useState<typeof tabs[number]>('Tracking');
    const {draft, dirty, resetDraft, updateDraft} = useSettingsDraft();

    const handleCancel = () => {
        resetDraft();
        onCancel();
    };

    const handleSave = async () => {
        await onSave(draft);
        resetDraft();
    };

    return (
        <div className="mx-auto flex h-screen max-w-sm flex-col gap-4 bg-zinc-950 px-5 py-4 text-zinc-100">
            <header className="flex items-center gap-3 rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
                <Button onClick={handleCancel}>Back</Button>
                <div className="text-left">
                    <p className="text-[11px] uppercase tracking-[0.2em] text-zinc-400">Settings</p>
                    <p className="text-base font-semibold">Configure tracking + input</p>
                </div>
            </header>

            <main className="flex flex-1 flex-col overflow-hidden rounded-2xl border border-zinc-900 bg-zinc-950">
                <nav className="grid grid-cols-2 gap-2 border-b border-zinc-900 px-4 py-3">
                    {tabs.map(tab => {
                        const isActive = activeTab === tab;
                        return (
                            <Button
                                key={tab}
                                fullWidth
                                variant={isActive ? 'action' : 'highlight'}
                                className={`text-xs font-semibold uppercase tracking-wide ${
                                    isActive ? '' : 'border-zinc-900 bg-zinc-950 text-zinc-400 hover:bg-zinc-900'
                                }`}
                                onClick={() => setActiveTab(tab)}
                                aria-current={isActive ? 'page' : undefined}
                            >
                                {tab}
                            </Button>
                        );
                    })}
                </nav>
                <section className="flex-1 overflow-auto px-4 py-4 text-sm text-zinc-300">
                    {renderTab(activeTab, draft, updateDraft)}
                </section>
            </main>

            <footer className="flex items-center justify-end gap-3 rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
                <Button onClick={handleCancel}>Cancel</Button>
                <Button variant="action" disabled={!dirty} onClick={handleSave}>
                    Save
                </Button>
            </footer>
        </div>
    );
};

type TabProps = {
    draft: AllParams;
    updateDraft: (updater: (current: AllParams) => AllParams) => void;
};

const TrackingTab: FC<TabProps> = ({draft, updateDraft}) => {
    const tracking = draft.tracking;
    const updateTracking = (changes: Partial<typeof tracking>) => {
        updateDraft(current => ({
            ...current,
            tracking: {
                ...current.tracking,
                ...changes,
            },
        }));
    };

    const templateSizes = [30, 40, 50];

    return (
        <div className="space-y-4">
            <div>
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-zinc-400">Template size</p>
                <div className="flex gap-2">
                    {templateSizes.map(size => (
                        <ChoiceButton
                            key={size}
                            selected={tracking.templateSizePx === size}
                            onClick={() => updateTracking({templateSizePx: size})}
                        >
                            {size}px
                        </ChoiceButton>
                    ))}
                </div>
            </div>

            <SliderField
                label="Search margin"
                min={10}
                max={120}
                step={5}
                value={tracking.searchMarginPx}
                onChange={value => updateTracking({searchMarginPx: value})}
            />

            <SliderField
                label={`Score threshold (${tracking.scoreThreshold.toFixed(2)})`}
                min={30}
                max={95}
                step={1}
                value={Math.round(tracking.scoreThreshold * 100)}
                onChange={value => updateTracking({scoreThreshold: value / 100})}
            />

            <label className="flex items-center gap-3 text-sm uppercase tracking-wide">
                <input
                    type="checkbox"
                    className="h-4 w-4 rounded border border-zinc-700 bg-zinc-900 text-emerald-400 accent-emerald-400 focus:ring-emerald-400"
                    checked={tracking.adaptiveTemplate}
                    onChange={event => updateTracking({adaptiveTemplate: event.target.checked})}
                />
                Adaptive template
            </label>

            <SliderField
                label={`Template alpha (${tracking.templateUpdateAlpha.toFixed(2)})`}
                min={0}
                max={100}
                step={5}
                value={Math.round(tracking.templateUpdateAlpha * 100)}
                disabled={!tracking.adaptiveTemplate}
                onChange={value => updateTracking({templateUpdateAlpha: value / 100})}
            />

            <div>
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-zinc-400">Marker shape</p>
                <div className="flex gap-2">
                    {['circle', 'square'].map(shape => (
                        <ChoiceButton
                            key={shape}
                            selected={tracking.markerShape === shape}
                            onClick={() => updateTracking({markerShape: shape as any})}
                        >
                            {shape.toUpperCase()}
                        </ChoiceButton>
                    ))}
                </div>
            </div>
        </div>
    );
};

const PointerTab: FC<TabProps> = ({draft, updateDraft}) => {
    const pointer = draft.pointer;
    const updatePointer = (changes: Partial<typeof pointer>) => {
        updateDraft(current => ({
            ...current,
            pointer: {
                ...current.pointer,
                ...changes,
            },
        }));
    };

    const updateAdvanced = (changes: {gainX?: number; gainY?: number; smoothing?: number} | null) => {
        updatePointer({advanced: changes ? {...(pointer.advanced ?? {gainX: pointer.sensitivity / 20, gainY: pointer.sensitivity / 20, smoothing: 0.2}), ...changes} : null});
    };

    const advanced = pointer.advanced;

    return (
        <div className="space-y-4">
            <SliderField
                label={`Sensitivity (${pointer.sensitivity})`}
                min={10}
                max={120}
                step={1}
                value={pointer.sensitivity}
                onChange={value => updatePointer({sensitivity: value})}
            />

            <SliderField
                label={`Deadzone (${pointer.deadzonePx}px)`}
                min={0}
                max={20}
                step={1}
                value={pointer.deadzonePx}
                onChange={value => updatePointer({deadzonePx: value})}
            />

            <SliderField
                label={`Max speed (${pointer.maxSpeedPx}px)`}
                min={10}
                max={60}
                step={1}
                value={pointer.maxSpeedPx}
                onChange={value => updatePointer({maxSpeedPx: value})}
            />

            <div className="rounded-2xl border border-zinc-800 p-3">
                <div className="mb-2 flex items-center justify-between text-sm">
                    <p className="font-semibold text-zinc-200">Advanced gain</p>
                    <Button variant="ghost" onClick={() => updateAdvanced(advanced ? null : {gainX: pointer.sensitivity / 20, gainY: pointer.sensitivity / 20, smoothing: 0.2})}>
                        {advanced ? 'Disable' : 'Enable'}
                    </Button>
                </div>
                {advanced ? (
                    <div className="space-y-3 text-sm">
                        <NumberField
                            label="Gain X"
                            value={advanced.gainX}
                            min={0.5}
                            max={6}
                            step={0.1}
                            onChange={value => updateAdvanced({gainX: value})}
                        />
                        <NumberField
                            label="Gain Y"
                            value={advanced.gainY}
                            min={0.5}
                            max={6}
                            step={0.1}
                            onChange={value => updateAdvanced({gainY: value})}
                        />
                        <NumberField
                            label="Smoothing"
                            value={advanced.smoothing}
                            min={0.05}
                            max={0.9}
                            step={0.05}
                            onChange={value => updateAdvanced({smoothing: value})}
                        />
                    </div>
                ) : (
                    <p className="text-xs text-zinc-400">Enable to override automatically mapped gain + smoothing.</p>
                )}
            </div>
        </div>
    );
};

const ClickingTab: FC<TabProps> = ({draft, updateDraft}) => {
    const clicking = draft.clicking;
    const updateClicking = (changes: Partial<typeof clicking>) => {
        updateDraft(current => ({
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
                onChange={value => updateClicking({dwellTimeMs: value})}
            />

            <SliderField
                label={`Dwell radius (${clicking.dwellRadiusPx}px)`}
                min={5}
                max={80}
                step={5}
                value={clicking.dwellRadiusPx}
                onChange={value => updateClicking({dwellRadiusPx: value})}
            />

        </div>
    );
};

const GeneralTab: FC<TabProps> = ({draft, updateDraft}) => {
    const hotkeys = draft.hotkeys;
    const general = draft.general ?? {autoStart: false};
    const updateHotkeys = (changes: Partial<typeof hotkeys>) => {
        updateDraft(current => ({
            ...current,
            hotkeys: {
                ...current.hotkeys,
                ...changes,
            },
        }));
    };

    const updateGeneral = (changes: Partial<typeof general>) => {
        updateDraft(current => ({
            ...current,
            general: {
                ...(current.general ?? {autoStart: false}),
                ...changes,
            },
        }));
    };

    return (
        <div className="space-y-4 text-sm text-zinc-300">
            <HotkeyField
                label="Start / Pause hotkey"
                description="Runs even when the app is in the background"
                value={hotkeys.startPause}
                onChange={value => updateHotkeys({startPause: value})}
            />
            <HotkeyField
                label="Recenter hotkey"
                description="Recenters the tracker without pausing the camera"
                value={hotkeys.recenter}
                onChange={value => updateHotkeys({recenter: value})}
            />

            <label className="block text-sm text-zinc-300">
                <div className="flex items-center gap-3 uppercase tracking-wide">
                    <input
                        type="checkbox"
                        className="h-5 w-5 rounded border border-zinc-700 bg-zinc-900 text-emerald-400 accent-emerald-400 focus:ring-emerald-400"
                        checked={general.autoStart}
                        onChange={event => updateGeneral({autoStart: event.target.checked})}
                    />
                    Autostart camera
                </div>
                <p className="mt-2 text-xs text-zinc-500">Begin capturing automatically when the app launches.</p>
            </label>
        </div>
    );
};

const renderTab = (
    tab: typeof tabs[number],
    draft: AllParams,
    updateDraft: (updater: (current: AllParams) => AllParams) => void
) => {
    switch (tab) {
        case 'Tracking':
            return <TrackingTab draft={draft} updateDraft={updateDraft}/>;
        case 'Pointer':
            return <PointerTab draft={draft} updateDraft={updateDraft}/>;
        case 'Clicking':
            return <ClickingTab draft={draft} updateDraft={updateDraft}/>;
        default:
            return <GeneralTab draft={draft} updateDraft={updateDraft}/>;
    }
};

const ChoiceButton: FC<{selected: boolean; onClick: () => void; children: ReactNode}> = ({selected, onClick, children}) => (
    <Button
        type="button"
        variant={selected ? 'action' : 'highlight'}
        className={`flex-1 text-sm ${selected ? '' : 'border-zinc-800 bg-zinc-950 text-zinc-400 hover:bg-zinc-900'}`}
        onClick={onClick}
    >
        {children}
    </Button>
);

const SliderField: FC<{
    label: string;
    value: number;
    min: number;
    max: number;
    step: number;
    disabled?: boolean;
    onChange: (value: number) => void;
}> = ({label, value, min, max, step, disabled, onChange}) => (
    <label className="block text-sm">
        <span className="mb-1 block text-xs font-semibold uppercase tracking-wide text-zinc-400">{label}</span>
        <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={value}
            disabled={disabled}
            onChange={event => onChange(parseFloat(event.target.value))}
            className={`slider-input ${disabled ? 'cursor-not-allowed opacity-50' : ''}`}
        />
    </label>
);

const NumberField: FC<{
    label: string;
    value: number;
    min: number;
    max: number;
    step: number;
    onChange: (value: number) => void;
}> = ({label, value, min, max, step, onChange}) => (
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

const HotkeyField: FC<{label: string; description: string; value: string; onChange: (value: string) => void}> = ({
    label,
    description,
    value,
    onChange,
}) => (
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

const formatHotkeyInput = (value: string): string => value.replace(/\s+/g, '').toUpperCase();
