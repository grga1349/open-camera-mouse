import { createContext, useCallback, useContext, useState, type FC, type ReactNode } from "react";

export type Status = {
  lost: boolean;
};

type StatusContextValue = {
  status: Status;
  setStatus: (next: Status) => void;
};

const StatusContext = createContext<StatusContextValue | undefined>(undefined);

export const StatusProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const [status, setStatusState] = useState<Status>({ lost: false });

  const setStatus = useCallback((next: Status) => {
    setStatusState(next);
  }, []);

  return <StatusContext.Provider value={{ status, setStatus }}>{children}</StatusContext.Provider>;
};

export const useStatus = (): StatusContextValue => {
  const ctx = useContext(StatusContext);
  if (!ctx) throw new Error("useStatus must be used within StatusProvider");
  return ctx;
};
