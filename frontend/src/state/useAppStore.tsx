import {createContext, useCallback, useContext, useMemo, useState, type FC, type ReactNode} from 'react';
import type {AllParams} from '../types/params';
import type {Telemetry} from '../types/telemetry';

export const defaultParams: AllParams = {
    tracking: {
        templateSizePx: 30,
        searchMarginPx: 30,
        scoreThreshold: 0.6,
        adaptiveTemplate: true,
        templateUpdateAlpha: 0.2,
        markerShape: 'circle',
    },
	pointer: {
		sensitivity: 30,
        deadzonePx: 1,
        maxSpeedPx: 25,
        advanced: null,
    },
    clicking: {
        dwellEnabled: false,
        dwellTimeMs: 500,
        dwellRadiusPx: 30,
        clickType: 'left',
        rightClickToggle: false,
    },
    hotkeys: {
        startPause: 'F11',
        recenter: 'F12',
    },
    general: {
        autoStart: false,
    },
};

const defaultTelemetry: Telemetry = {
    fps: 0,
    score: 0,
    state: 'idle',
    trackingOn: false,
    lost: false,
    posX: null,
    posY: null,
};

export type PreviewFrame = {
    data: string;
    width: number;
    height: number;
    timestamp: string;
};

export type StoreActions = {
    updateParams: (updater: (current: AllParams) => AllParams) => void;
    setTelemetry: (next: Telemetry) => void;
    setParams: (next: AllParams) => void;
    setPreview: (frame: PreviewFrame | null) => void;
    setRunning: (running: boolean) => void;
};

export type AppState = {
    params: AllParams;
    telemetry: Telemetry;
    preview: PreviewFrame | null;
    isRunning: boolean;
} & StoreActions;

const AppContext = createContext<AppState | undefined>(undefined);

export const AppProvider: FC<{children: ReactNode}> = ({children}) => {
    const [params, setParams] = useState<AllParams>(defaultParams);
    const [telemetry, setTelemetry] = useState<Telemetry>(defaultTelemetry);
    const [preview, setPreviewState] = useState<PreviewFrame | null>(null);
    const [isRunning, setIsRunning] = useState(false);

    const updateParams = useCallback((updater: (current: AllParams) => AllParams) => {
        setParams(prev => updater(prev));
    }, []);

    const setAllParams = useCallback((next: AllParams) => {
        setParams(next);
    }, []);

    const setPreview = useCallback((frame: PreviewFrame | null) => {
        setPreviewState(frame);
    }, []);

    const setRunning = useCallback((running: boolean) => {
        setIsRunning(running);
    }, []);

    const value = useMemo<AppState>(
        () => ({
            params,
            telemetry,
            preview,
            isRunning,
            updateParams,
            setTelemetry,
            setParams: setAllParams,
            setPreview,
            setRunning,
        }),
        [params, telemetry, preview, isRunning, updateParams, setAllParams, setPreview, setRunning]
    );

    return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
};

export const useAppStore = (): AppState => {
    const context = useContext(AppContext);
    if (!context) {
        throw new Error('useAppStore must be used within AppProvider');
    }
    return context;
};
