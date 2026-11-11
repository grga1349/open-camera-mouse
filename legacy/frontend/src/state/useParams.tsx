import { createContext, useCallback, useContext, useState, type FC, type ReactNode } from "react";
import type { Params } from "../types/params";

export const defaultParams: Params = {
  templateSizePx: 45,
  gainMultiplier: 8.0,
  smoothing: 0.3,
  dwellEnabled: false,
  dwellTimeMs: 500,
  autoStart: false,
  rightClickEnabled: false,
};

type ParamsContextValue = {
  params: Params;
  setParams: (next: Params) => void;
  updateParams: (updater: (current: Params) => Params) => void;
};

const ParamsContext = createContext<ParamsContextValue | undefined>(undefined);

export const ParamsProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const [params, setParamsState] = useState<Params>(defaultParams);

  const setParams = useCallback((next: Params) => {
    setParamsState(next);
  }, []);

  const updateParams = useCallback((updater: (current: Params) => Params) => {
    setParamsState((prev) => updater(prev));
  }, []);

  return <ParamsContext.Provider value={{ params, setParams, updateParams }}>{children}</ParamsContext.Provider>;
};

export const useParams = (): ParamsContextValue => {
  const ctx = useContext(ParamsContext);
  if (!ctx) throw new Error("useParams must be used within ParamsProvider");
  return ctx;
};
