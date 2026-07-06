import { useCallback, useEffect, useRef, useState } from "react";
import { EventsOn } from "../../../../wailsjs/runtime/runtime";
import { BeginRecenter, ConfirmRecenter } from "../../../../wailsjs/go/main/App";
import { useAppError } from "../../../state/useAppError";
import { useRecenterCountdown } from "./useRecenterCountdown";

const RECENTER_COUNTDOWN_SECONDS = 3;

/**
 * Single guided recenter flow shared by the UI button and the F12 hotkey:
 * pause tracking (BeginRecenter), let the user reposition during a visible
 * countdown, then pick the frame center and resume (ConfirmRecenter). F12
 * reaches this same function via the "recenter:hotkey" event instead of
 * duplicating the flow on the Go side.
 */
export const useRecenter = () => {
  const [isRecentering, setIsRecentering] = useState(false);
  const isRecenteringRef = useRef(false);
  const { reportError, clearError } = useAppError();
  const { value: countdown, start: startCountdown } = useRecenterCountdown();

  const runRecenterFlow = useCallback(async () => {
    if (isRecenteringRef.current) return;

    try {
      await BeginRecenter();
    } catch (err) {
      console.error("begin recenter failed", err);
      reportError("Could not start recenter — is tracking running?");
      return;
    }

    isRecenteringRef.current = true;
    setIsRecentering(true);
    clearError();

    startCountdown(RECENTER_COUNTDOWN_SECONDS, async () => {
      try {
        await ConfirmRecenter();
        clearError();
      } catch (err) {
        console.error("confirm recenter failed", err);
        reportError("Recenter failed to complete.");
      } finally {
        isRecenteringRef.current = false;
        setIsRecentering(false);
      }
    });
  }, [reportError, clearError, startCountdown]);

  useEffect(() => {
    const off = EventsOn("recenter:hotkey", () => {
      void runRecenterFlow();
    });
    return off;
  }, [runRecenterFlow]);

  return { countdown, isRecentering, handleRecenter: runRecenterFlow };
};
