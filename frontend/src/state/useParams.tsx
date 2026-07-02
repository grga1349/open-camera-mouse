import { createContext, useCallback, useContext, useState, type FC, type ReactNode } from "react";
import type { AllParams } from "../types/params";

export const defaultParams: AllParams = {
  tracking: {
    templateSizePx: 30,
    searchMarginPx: 30,
    scoreThreshold: 0.6,
    adaptiveTemplate: true,
    templateUpdateAlpha: 0.2,
    markerShape: "circle",
  },
  pointer: {
    sensitivity: 30,
    amplification: 4,
    deadzonePx: 1,
    maxSpeedPx: 25,
    advanced: null,
  },
  clicking: {
    dwellEnabled: false,
    dwellTimeMs: 500,
    dwellRadiusPx: 30,
    clickType: "left",
    rightClickToggle: false,
  },
  hotkeys: {
    startPause: "F11",
    recenter: "F12",
  },
  general: {
    autoStart: false,
    dwellOnStartup: false,
  },
};

type ParamsContextValue = {
  params: AllParams;
  setParams: (next: AllParams) => void;
  updateParams: (updater: (current: AllParams) => AllParams) => void;
};

const ParamsContext = createContext<ParamsContextValue | undefined>(undefined);

export const ParamsProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const [params, setParamsState] = useState<AllParams>(defaultParams);

  const setParams = useCallback((next: AllParams) => {
    setParamsState(next);
  }, []);

  const updateParams = useCallback((updater: (current: AllParams) => AllParams) => {
    setParamsState((prev) => updater(prev));
  }, []);

  return (
    <ParamsContext.Provider value={{ params, setParams, updateParams }}>
      {children}
    </ParamsContext.Provider>
  );
};

export const useParams = (): ParamsContextValue => {
  const ctx = useContext(ParamsContext);
  if (!ctx) throw new Error("useParams must be used within ParamsProvider");
  return ctx;
};
