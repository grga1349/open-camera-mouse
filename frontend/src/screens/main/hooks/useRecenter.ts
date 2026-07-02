import { useCallback, useEffect } from "react";
import { EventsOn } from "../../../../wailsjs/runtime/runtime";
import { Recenter, ToggleTracking } from "../../../../wailsjs/go/main/App";
import { useTelemetry } from "../../../state/useTelemetry";
import { useRecenterCountdown } from "./useRecenterCountdown";

export const useRecenter = () => {
  const { telemetry } = useTelemetry();
  const { value: countdown, start: startCountdown } = useRecenterCountdown();

  const handleRecenter = useCallback(async () => {
    if (countdown > 0) return;

    const trackingWasEnabled = telemetry.trackingOn;
    if (trackingWasEnabled) {
      try {
        await ToggleTracking(false);
      } catch (err) {
        console.error("failed to pause tracking before recenter", err);
      }
    }

    try {
      await Recenter();
    } catch (err) {
      console.error("recenter failed", err);
    }

    startCountdown(5, async () => {
      if (trackingWasEnabled) {
        try {
          await ToggleTracking(true);
        } catch (err) {
          console.error("failed to resume tracking after recenter", err);
        }
      }
    });
  }, [countdown, telemetry.trackingOn, startCountdown]);

  useEffect(() => {
    const off = EventsOn("recenter:hotkey", () => {
      void handleRecenter();
    });
    return off;
  }, [handleRecenter]);

  return { countdown, handleRecenter };
};
