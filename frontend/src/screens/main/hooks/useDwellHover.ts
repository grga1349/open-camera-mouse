import { useCallback, useEffect, useRef } from "react";
import { UpdateParams } from "../../../../wailsjs/go/main/App";
import { config as backendConfig } from "../../../../wailsjs/go/models";
import { useParams } from "../../../state/useParams";

export const useDwellHover = () => {
  const { params, setParams } = useParams();
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    },
    [],
  );

  const updateDwellEnabled = useCallback(
    async (enabled: boolean) => {
      const next = { ...params, dwellEnabled: enabled };
      setParams(next);
      try {
        await UpdateParams(next as unknown as backendConfig.Params);
      } catch (err) {
        console.error("update params failed", err);
      }
    },
    [params, setParams],
  );

  const onHoverStart = useCallback(() => {
    if (params.dwellEnabled || timerRef.current) return;
    timerRef.current = window.setTimeout(() => {
      timerRef.current = null;
      void updateDwellEnabled(true);
    }, 500);
  }, [params.dwellEnabled, updateDwellEnabled]);

  const onHoverEnd = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  return { onHoverStart, onHoverEnd };
};
