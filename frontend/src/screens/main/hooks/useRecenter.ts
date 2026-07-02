import { useCallback, useEffect } from "react";
import { EventsOn } from "../../../../wailsjs/runtime/runtime";
import { Recenter } from "../../../../wailsjs/go/main/App";
import { useRecenterCountdown } from "./useRecenterCountdown";

export const useRecenter = () => {
  const { value: countdown, start: startCountdown } = useRecenterCountdown();

  const handleRecenter = useCallback(async () => {
    if (countdown > 0) return;
    try {
      await Recenter();
    } catch (err) {
      console.error("recenter failed", err);
    }
    startCountdown(5);
  }, [countdown, startCountdown]);

  useEffect(() => {
    const off = EventsOn("recenter:hotkey", () => {
      void handleRecenter();
    });
    return off;
  }, [handleRecenter]);

  return { countdown, handleRecenter };
};
