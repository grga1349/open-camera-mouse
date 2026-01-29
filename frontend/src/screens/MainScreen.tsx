import {useEffect, useRef, useState, type FC, type MouseEvent} from 'react';
import {Button} from '../components/Button';
import {useAppStore} from '../state/useAppStore';
import {config as backendConfig} from '../../wailsjs/go/models';
import {Recenter, SetPickPoint, ToggleTracking, UpdateParams} from '../../wailsjs/go/main/App';

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
    const dwellHoverRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        return () => {
            if (countdownRef.current) {
                clearInterval(countdownRef.current);
            }
            if (dwellHoverRef.current) {
                clearTimeout(dwellHoverRef.current);
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

        const trackingWasEnabled = telemetry.trackingOn;
        try {
            if (trackingWasEnabled) {
                await ToggleTracking(false);
            }
        } catch (err) {
            console.error('failed to pause tracking before recenter', err);
        }

        try {
            await Recenter();
        } catch (err) {
            console.error('recenter failed', err);
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
                if (trackingWasEnabled) {
                    await ToggleTracking(true);
                }
            } catch (err) {
                console.error('failed to resume tracking after recenter', err);
            }
        }, 1000);
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

    const toggleDwell = () => {
        if (dwellHoverRef.current) {
            clearTimeout(dwellHoverRef.current);
            dwellHoverRef.current = null;
        }
        void updateClicking({dwellEnabled: !params.clicking.dwellEnabled});
    };

    const handleDwellHoverStart = () => {
        if (params.clicking.dwellEnabled || dwellHoverRef.current) {
            return;
        }
        dwellHoverRef.current = window.setTimeout(() => {
            dwellHoverRef.current = null;
            if (!params.clicking.dwellEnabled) {
                void updateClicking({dwellEnabled: true});
            }
        }, 500);
    };

    const handleDwellHoverEnd = () => {
        if (dwellHoverRef.current) {
            clearTimeout(dwellHoverRef.current);
            dwellHoverRef.current = null;
        }
    };
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
                        {recenterCountdown > 0 ? `Recenter in ${recenterCountdown}` : 'Recenter'}
                    </Button>
                    <div className="grid grid-cols-2 gap-3">
                        <Button
                            fullWidth
                            onClick={toggleDwell}
                            onMouseEnter={handleDwellHoverStart}
                            onMouseLeave={handleDwellHoverEnd}
                            title="Enable dwell clicking"
                        >
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
