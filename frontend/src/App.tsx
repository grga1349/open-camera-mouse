import {useEffect, useState} from 'react';
import {EventsOn} from '../wailsjs/runtime/runtime';
import {GetParams, SaveParams, Start, Stop} from '../wailsjs/go/main/App';
import {config as backendConfig} from '../wailsjs/go/models';
import {MainScreen} from './screens/MainScreen';
import {SettingsScreen} from './screens/SettingsScreen';
import {useAppStore} from './state/useAppStore';
import type {AllParams} from './types/params';

type Screen = 'main' | 'settings';

const normalizeParams = (input: backendConfig.AllParams): AllParams => JSON.parse(JSON.stringify(input));

function App() {
    const [screen, setScreen] = useState<Screen>('main');
    const {setParams, setTelemetry, setPreview, setRunning} = useAppStore();

    useEffect(() => {
        let offPreview: (() => void) | undefined;
        let offTelemetry: (() => void) | undefined;
        let offParams: (() => void) | undefined;

        GetParams()
            .then(res => setParams(normalizeParams(res)))
            .catch(err => console.error('failed to load params', err));

        offPreview = EventsOn('preview:frame', frame => {
            const data = frame?.Data ?? frame?.data;
            if (!data) {
                return;
            }
            setPreview({
                data,
                width: frame?.Width ?? frame?.width ?? 0,
                height: frame?.Height ?? frame?.height ?? 0,
                timestamp: frame?.Timestamp ?? frame?.timestamp ?? new Date().toISOString(),
            });
        });

        offTelemetry = EventsOn('telemetry:state', payload => {
            const fps = payload?.FPS ?? payload?.fps ?? 0;
            const lost = payload?.Lost ?? payload?.lost ?? false;
            const tracking = payload?.Tracking ?? payload?.tracking ?? false;
            setTelemetry({
                fps,
                score: payload?.Score ?? payload?.score ?? 0,
                state: lost ? 'lost' : tracking ? 'tracking' : 'idle',
                trackingOn: tracking,
                lost,
                posX: payload?.PosX ?? payload?.posX ?? null,
                posY: payload?.PosY ?? payload?.posY ?? null,
            });
        });

        offParams = EventsOn('params:update', payload => {
            if (!payload) {
                return;
            }
            setParams(normalizeParams(payload));
        });

        return () => {
            offPreview?.();
            offTelemetry?.();
            offParams?.();
        };
    }, [setParams, setPreview, setTelemetry]);

    const openSettings = () => setScreen('settings');
    const closeSettings = () => setScreen('main');

    const handleSettingsSave = async (next: AllParams) => {
        await SaveParams(next as unknown as backendConfig.AllParams);
        setParams(next);
        closeSettings();
    };

    const handleStart = async () => {
        await Start();
        setRunning(true);
    };

    const handleStop = async () => {
        await Stop();
        setRunning(false);
    };

    return screen === 'main' ? (
        <MainScreen onOpenSettings={openSettings} onStart={handleStart} onStop={handleStop}/>
    ) : (
        <SettingsScreen onCancel={closeSettings} onSave={handleSettingsSave}/>
    );
}

export default App;
