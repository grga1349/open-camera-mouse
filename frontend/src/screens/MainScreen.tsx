import type {FC} from 'react';
import {Button} from '../components/Button';

type MainScreenProps = {
    onOpenSettings: () => void;
};

export const MainScreen: FC<MainScreenProps> = ({onOpenSettings}) => {
	return (
		<div className="mx-auto flex h-screen max-w-sm flex-col gap-4 bg-zinc-950 px-5 py-4 text-zinc-100">
			<header className="flex items-center justify-between rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
				<div className="text-left">
					<p className="text-[11px] uppercase tracking-[0.2em] text-zinc-400">Open Camera Mouse</p>
					<p className="text-base font-semibold">Status: Idle</p>
				</div>
				<Button onClick={onOpenSettings}>
					Settings
				</Button>
			</header>

			<main className="flex flex-1 flex-col gap-4">
					<div className="flex justify-center">
						<div
							className="relative w-full max-w-[360px] overflow-hidden rounded-3xl border border-zinc-900 bg-zinc-950 p-2"
						style={{aspectRatio: '4 / 3'}}
					>
							<div className="flex h-full w-full items-center justify-center rounded-2xl border border-dashed border-zinc-800 text-sm text-zinc-500">
								Camera preview placeholder
							</div>
					</div>
				</div>

				<div className="grid gap-3 text-sm">
					<Button variant="action" fullWidth>
						Recenter
					</Button>
					<Button fullWidth>
						Start / Pause
					</Button>
					<div className="grid grid-cols-2 gap-3">
						<Button fullWidth>
							Dwell Toggle
						</Button>
						<Button fullWidth>
							Right Click Toggle
						</Button>
					</div>
				</div>
			</main>
		</div>
	);
};
