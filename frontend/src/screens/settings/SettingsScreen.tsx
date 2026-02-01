import {useMemo, useState, type FC} from 'react';
import {Button} from '../../components/Button';
import {ScreenShell} from '../../components/layout/ScreenShell';
import {SettingsTabs} from '../../components/settings/SettingsTabs';
import {defaultParams} from '../../state/useAppStore';
import {useSettingsDraft} from '../../state/useSettingsDraft';
import type {AllParams} from '../../types/params';
import {ClickingTab} from './tabs/ClickingTab';
import {GeneralTab} from './tabs/GeneralTab';
import {PointerTab} from './tabs/PointerTab';
import {TrackingTab} from './tabs/TrackingTab';
import type {TabProps} from './tabs/types';

const SETTINGS_TABS = ['Tracking', 'Pointer', 'Clicking', 'General'] as const;
type SettingsTab = (typeof SETTINGS_TABS)[number];

const TAB_COMPONENTS: Record<SettingsTab, FC<TabProps>> = {
    Tracking: TrackingTab,
    Pointer: PointerTab,
    Clicking: ClickingTab,
    General: GeneralTab,
};

const cloneDefaults = (): AllParams => JSON.parse(JSON.stringify(defaultParams));

type SettingsScreenProps = {
    onSave: (params: AllParams) => void | Promise<void>;
    onCancel: () => void;
};

export const SettingsScreen: FC<SettingsScreenProps> = ({onSave, onCancel}) => {
    const [activeTab, setActiveTab] = useState<SettingsTab>('Tracking');
    const {draft, dirty, resetDraft, updateDraft} = useSettingsDraft();

    const handleCancel = () => {
        resetDraft();
        onCancel();
    };

    const handleSave = async () => {
        await onSave(draft);
        resetDraft();
    };

    const handleResetDefaults = () => {
        updateDraft(() => cloneDefaults());
    };

    const activeTabContent = useMemo(() => {
        const Component = TAB_COMPONENTS[activeTab];
        return <Component draft={draft} updateDraft={updateDraft}/>;
    }, [activeTab, draft, updateDraft]);

    return (
        <ScreenShell
            header={
                <header className="flex items-center gap-3 rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
                    <Button onClick={handleCancel}>Back</Button>
                    <div className="text-left">
                        <p className="text-[11px] uppercase tracking-[0.2em] text-zinc-400">Settings</p>
                        <p className="text-base font-semibold">Configure tracking + input</p>
                    </div>
                </header>
            }
            footer={
                <footer className="flex items-center justify-end gap-3 rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
                    <Button onClick={handleResetDefaults}>Reset</Button>
                    <Button onClick={handleCancel}>Cancel</Button>
                    <Button variant="action" disabled={!dirty} onClick={handleSave}>
                        Save
                    </Button>
                </footer>
            }
            mainClassName="gap-4"
        >
            <div className="flex flex-1 flex-col overflow-hidden rounded-2xl border border-zinc-900 bg-zinc-950">
                <SettingsTabs tabs={SETTINGS_TABS} activeTab={activeTab} onChange={setActiveTab}/>
                <section className="flex-1 overflow-auto px-4 py-4 text-sm text-zinc-300">{activeTabContent}</section>
            </div>
        </ScreenShell>
    );
};

