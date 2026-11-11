import { createContext, useCallback, useContext, useState, type FC, type ReactNode } from "react";

type AppErrorContextValue = {
  error: string | null;
  reportError: (message: string) => void;
  clearError: () => void;
};

const AppErrorContext = createContext<AppErrorContextValue | undefined>(undefined);

export const AppErrorProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const [error, setError] = useState<string | null>(null);

  const reportError = useCallback((message: string) => {
    setError(message);
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return <AppErrorContext.Provider value={{ error, reportError, clearError }}>{children}</AppErrorContext.Provider>;
};

export const useAppError = (): AppErrorContextValue => {
  const ctx = useContext(AppErrorContext);
  if (!ctx) throw new Error("useAppError must be used within AppErrorProvider");
  return ctx;
};
