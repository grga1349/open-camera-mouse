import {useState, type FC, type ReactNode} from 'react';
import {Button} from '../components/Button';

const tabs = ['Tracking', 'Pointer', 'Clicking', 'Hotkeys'] as const;

const tabButtonBase = 'w-full rounded-xl px-3 py-2 text-xs font-semibold uppercase tracking-wide text-center transition';

type SettingsScreenProps = {
    onSave: () => void;
    onCancel: () => void;
};

export const SettingsScreen: FC<SettingsScreenProps> = ({onSave, onCancel}) => {
	const [activeTab, setActiveTab] = useState<typeof tabs[number]>('Tracking');

		return (
			<div className="mx-auto flex h-screen max-w-sm flex-col gap-4 bg-zinc-950 px-5 py-4 text-zinc-100">
			<header className="flex items-center gap-3 rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
				<Button onClick={onCancel}>
					Back
				</Button>
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
					{renderPlaceholder(activeTab)}
				</section>
			</main>

			<footer className="flex items-center justify-end gap-3 rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
				<Button onClick={onCancel}>
					Cancel
				</Button>
				<Button variant="action" onClick={onSave}>
					Save
				</Button>
			</footer>
		</div>
	);
};

function renderPlaceholder(tab: typeof tabs[number]): ReactNode {
	return (
		<div className="space-y-3 rounded-2xl border border-zinc-900 bg-zinc-950 p-4 text-zinc-100">
			<p className="text-sm font-semibold">{tab} tab</p>
			<p className="text-zinc-400">
				Replace this placeholder with the real settings controls as you implement the later phases.
			</p>
		</div>
	);
}
