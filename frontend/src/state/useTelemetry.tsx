import { createContext, useCallback, useContext, useState, type FC, type ReactNode } from "react";
import type { Telemetry } from "../types/telemetry";

const defaultTelemetry: Telemetry = {
  fps: 0,
  score: 0,
  state: "idle",
  trackingOn: false,
  lost: false,
  posX: null,
  posY: null,
};

type TelemetryContextValue = {
  telemetry: Telemetry;
  setTelemetry: (next: Telemetry) => void;
};

const TelemetryContext = createContext<TelemetryContextValue | undefined>(undefined);

export const TelemetryProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const [telemetry, setTelemetryState] = useState<Telemetry>(defaultTelemetry);

  const setTelemetry = useCallback((next: Telemetry) => {
    setTelemetryState(next);
  }, []);

  return (
    <TelemetryContext.Provider value={{ telemetry, setTelemetry }}>
      {children}
    </TelemetryContext.Provider>
  );
};

export const useTelemetry = (): TelemetryContextValue => {
  const ctx = useContext(TelemetryContext);
  if (!ctx) throw new Error("useTelemetry must be used within TelemetryProvider");
  return ctx;
};
