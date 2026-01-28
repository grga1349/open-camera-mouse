import {useState, type FC, type ReactNode} from 'react';
import {Button} from '../components/Button';
import {useSettingsDraft} from '../state/useSettingsDraft';
import type {AllParams} from '../types/params';

const tabs = ['Tracking', 'Pointer', 'Clicking', 'Hotkeys'] as const;

const tabButtonBase = 'w-full rounded-xl px-3 py-2 text-xs font-semibold uppercase tracking-wide text-center transition';

type SettingsScreenProps = {
    onSave: (params: AllParams) => void | Promise<void>;
    onCancel: () => void;
};

export const SettingsScreen: FC<SettingsScreenProps> = ({onSave, onCancel}) => {
    const [activeTab, setActiveTab] = useState<typeof tabs[number]>('Tracking');
    const {draft, dirty, resetDraft} = useSettingsDraft();

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
                        const tabClass = isActive
                            ? `${tabButtonBase} bg-emerald-500 text-white`
                            : `${tabButtonBase} border border-zinc-900 text-zinc-400 hover:bg-zinc-900`;
                        return (
                            <button
                                key={tab}
                                className={tabClass}
                                onClick={() => setActiveTab(tab)}
                                aria-current={isActive ? 'page' : undefined}
                            >
                                {tab}
                            </button>
                        );
                    })}
                </nav>
                <section className="flex-1 overflow-auto px-4 py-4 text-sm text-zinc-400">
                    {renderPlaceholder(activeTab, draft)}
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

function renderPlaceholder(tab: typeof tabs[number], draft: AllParams): ReactNode {
    return (
        <div className="space-y-3 rounded-2xl border border-zinc-900 bg-zinc-950 p-4 text-zinc-100">
            <p className="text-sm font-semibold">{tab} tab</p>
            <pre className="overflow-auto rounded bg-zinc-900 p-3 text-xs text-zinc-400">
                {tab === 'Hotkeys' ? 'Hotkey settings coming soon' : JSON.stringify((draft as any)[tab.toLowerCase()], null, 2)}
            </pre>
        </div>
    );
}
