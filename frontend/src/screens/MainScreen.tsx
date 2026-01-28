import {useEffect, useRef, useState, type FC, type MouseEvent} from 'react';
import {Button} from '../components/Button';
import {useAppStore} from '../state/useAppStore';
import type {AllParams} from '../types/params';
import {config as backendConfig} from '../../wailsjs/go/models';
import {Recenter, SetPickPoint, UpdateParams} from '../../wailsjs/go/main/App';

const statusColor = (lost: boolean) => (lost ? 'text-red-400' : 'text-emerald-400');

type MainScreenProps = {
    onOpenSettings: () => void;
    onStart: () => Promise<void> | void;
    onStop: () => Promise<void> | void;
};

export const MainScreen: FC<MainScreenProps> = ({onOpenSettings, onStart, onStop}) => {
    const {params, telemetry, preview, isRunning, setParams} = useAppStore();
    const [recenterCountdown, setRecenterCountdown] = useState(0);
    const countdownRef = useRef<ReturnType<typeof setInterval> | null>(null);

    useEffect(() => {
        return () => {
            if (countdownRef.current) {
                clearInterval(countdownRef.current);
            }
        };
    }, []);

    const handleStartStop = async () => {
        try {
            if (isRunning) {
                await onStop();
            } else {
                await onStart();
            }
        } catch (err) {
            console.error('start/stop failed', err);
        }
    };

    const handleRecenter = async () => {
        if (recenterCountdown > 0) {
            return;
        }
        try {
            const wasRunning = isRunning;
            if (wasRunning) {
                await onStop();
            }
            let remaining = 5;
            setRecenterCountdown(remaining);
            countdownRef.current = window.setInterval(async () => {
                remaining -= 1;
                if (remaining > 0) {
                    setRecenterCountdown(remaining);
                    return;
                }
                if (countdownRef.current) {
                    clearInterval(countdownRef.current);
                    countdownRef.current = null;
                }
                setRecenterCountdown(0);
                try {
                    await Recenter();
                    if (wasRunning) {
                        await onStart();
                    }
                } catch (err) {
                    console.error('recenter failed', err);
                }
            }, 1000);
        } catch (err) {
            console.error('recenter failed', err);
        }
    };

    const handlePreviewClick = async (event: MouseEvent<HTMLDivElement>) => {
        if (!preview) {
            return;
        }
        const rect = event.currentTarget.getBoundingClientRect();
        const relX = event.clientX - rect.left;
        const relY = event.clientY - rect.top;
        const xRatio = preview.width > 0 ? preview.width / rect.width : 1;
        const yRatio = preview.height > 0 ? preview.height / rect.height : 1;
        const x = Math.max(0, Math.min(preview.width, Math.round(relX * xRatio)));
        const y = Math.max(0, Math.min(preview.height, Math.round(relY * yRatio)));
        try {
            await SetPickPoint(x, y);
        } catch (err) {
            console.error('set pick point failed', err);
        }
    };

    const updateClicking = async (updates: Partial<typeof params.clicking>) => {
        const next = {
            ...params,
            clicking: {
                ...params.clicking,
                ...updates,
            },
        };
        setParams(next);
        try {
            await UpdateParams(next as unknown as backendConfig.AllParams);
        } catch (err) {
            console.error('update params failed', err);
        }
    };

    const toggleDwell = () => updateClicking({dwellEnabled: !params.clicking.dwellEnabled});
    const toggleRightClick = () => updateClicking({rightClickToggle: !params.clicking.rightClickToggle});

    const previewSrc = preview ? `data:image/jpeg;base64,${preview.data}` : null;

    return (
        <div className="mx-auto flex h-screen max-w-sm flex-col gap-4 bg-zinc-950 px-5 py-4 text-zinc-100">
            <header className="flex items-center justify-between rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
                <div className="text-left">
                    <p className="text-[11px] uppercase tracking-[0.2em] text-zinc-400">Open Camera Mouse</p>
                    <p className={`text-base font-semibold ${statusColor(telemetry.lost)}`}>
                        {telemetry.lost ? 'LOST' : 'OK'} • {telemetry.fps.toFixed(1)} fps
                    </p>
                </div>
                <Button onClick={onOpenSettings}>Settings</Button>
            </header>

            <main className="flex flex-1 flex-col gap-4">
                <div className="flex justify-center">
                    <div
                        className="relative w-full max-w-[360px] overflow-hidden rounded-3xl border border-zinc-900 bg-zinc-950"
                        style={{aspectRatio: '4 / 3'}}
                        onClick={handlePreviewClick}
                    >
                        {previewSrc ? (
                            <img src={previewSrc} alt="camera preview" className="h-full w-full object-cover"/>
                        ) : (
                            <div className="flex h-full items-center justify-center text-sm text-zinc-500">
                                Preview unavailable
                            </div>
                        )}
                    </div>
                </div>

                <div className="grid gap-3 text-sm">
                    <Button variant="action" fullWidth onClick={handleStartStop}>
                        {isRunning ? 'Pause' : 'Start'}
                    </Button>
                    <Button fullWidth onClick={handleRecenter} disabled={recenterCountdown > 0}>
                        {recenterCountdown > 0 ? `Recenter (${recenterCountdown})` : 'Recenter'}
                    </Button>
                    <div className="grid grid-cols-2 gap-3">
                        <Button fullWidth onClick={toggleDwell} title="Enable dwell clicking">
                            Dwell {params.clicking.dwellEnabled ? 'On' : 'Off'}
                        </Button>
                        <Button fullWidth onClick={toggleRightClick}>
                            Right Click {params.clicking.rightClickToggle ? 'On' : 'Off'}
                        </Button>
                    </div>
                </div>
            </main>
        </div>
    );
};
