import {createContext, useContext, useMemo, useState, type FC, type ReactNode, useCallback} from 'react';
import type {AllParams} from '../types/params';
import type {Telemetry} from '../types/telemetry';

const defaultParams: AllParams = {
    tracking: {
        templateSizePx: 20,
        searchMarginPx: 30,
        scoreThreshold: 0.6,
        adaptiveTemplate: true,
        templateUpdateAlpha: 0.2,
        markerShape: 'circle',
    },
    pointer: {
        sensitivity: 50,
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

export type StoreActions = {
    updateParams: (updater: (current: AllParams) => AllParams) => void;
    setTelemetry: (next: Telemetry) => void;
    setParams: (next: AllParams) => void;
};

export type AppState = {
    params: AllParams;
    telemetry: Telemetry;
} & StoreActions;

const AppContext = createContext<AppState | undefined>(undefined);

export const AppProvider: FC<{children: ReactNode}> = ({children}) => {
    const [params, setParams] = useState<AllParams>(defaultParams);
    const [telemetry, setTelemetry] = useState<Telemetry>(defaultTelemetry);

    const updateParams = useCallback((updater: (current: AllParams) => AllParams) => {
        setParams(prev => updater(prev));
    }, []);

    const setAllParams = useCallback((next: AllParams) => {
        setParams(next);
    }, []);

    const value = useMemo<AppState>(
        () => ({
            params,
            telemetry,
            updateParams,
            setTelemetry,
            setParams: setAllParams,
        }),
        [params, telemetry, updateParams, setAllParams]
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
