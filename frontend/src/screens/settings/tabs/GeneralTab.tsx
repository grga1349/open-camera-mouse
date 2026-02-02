import type {FC} from 'react';
import {HotkeyField} from '../../../components/inputs/HotkeyField';
import type {TabProps} from './types';

export const GeneralTab: FC<TabProps> = ({draft, updateDraft}) => {
    const hotkeys = draft.hotkeys;
    const general = draft.general ?? {autoStart: false, dwellOnStartup: false};

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
                ...(current.general ?? {autoStart: false, dwellOnStartup: false}),
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

            <label className="block text-sm text-zinc-300">
                <div className="flex items-center gap-3 uppercase tracking-wide">
                    <input
                        type="checkbox"
                        className="h-5 w-5 rounded border border-zinc-700 bg-zinc-900 text-emerald-400 accent-emerald-400 focus:ring-emerald-400"
                        checked={general.dwellOnStartup}
                        onChange={event => updateGeneral({dwellOnStartup: event.target.checked})}
                    />
                    Enable dwell click on startup
                </div>
                <p className="mt-2 text-xs text-zinc-500">Restore dwell click state from saved settings when the app launches.</p>
            </label>
        </div>
    );
};

