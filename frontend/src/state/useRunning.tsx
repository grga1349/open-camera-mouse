import { createContext, useCallback, useContext, useState, type FC, type ReactNode } from "react";

type RunningContextValue = {
  isRunning: boolean;
  setRunning: (running: boolean) => void;
};

const RunningContext = createContext<RunningContextValue | undefined>(undefined);

export const RunningProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const [isRunning, setIsRunning] = useState(false);

  const setRunning = useCallback((running: boolean) => {
    setIsRunning(running);
  }, []);

  return <RunningContext.Provider value={{ isRunning, setRunning }}>{children}</RunningContext.Provider>;
};

export const useRunning = (): RunningContextValue => {
  const ctx = useContext(RunningContext);
  if (!ctx) throw new Error("useRunning must be used within RunningProvider");
  return ctx;
};
