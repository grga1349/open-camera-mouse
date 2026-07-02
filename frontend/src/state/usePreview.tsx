import { createContext, useCallback, useContext, useState, type FC, type ReactNode } from "react";
import type { PreviewFrame } from "../types/preview";

type PreviewContextValue = {
  preview: PreviewFrame | null;
  setPreview: (frame: PreviewFrame | null) => void;
};

const PreviewContext = createContext<PreviewContextValue | undefined>(undefined);

export const PreviewProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const [preview, setPreviewState] = useState<PreviewFrame | null>(null);

  const setPreview = useCallback((frame: PreviewFrame | null) => {
    setPreviewState(frame);
  }, []);

  return (
    <PreviewContext.Provider value={{ preview, setPreview }}>
      {children}
    </PreviewContext.Provider>
  );
};

export const usePreview = (): PreviewContextValue => {
  const ctx = useContext(PreviewContext);
  if (!ctx) throw new Error("usePreview must be used within PreviewProvider");
  return ctx;
};
