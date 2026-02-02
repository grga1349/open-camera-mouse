import { useCallback, useEffect, useRef, useState } from "react";

export const useRecenterCountdown = () => {
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const [value, setValue] = useState(0);

  const clear = useCallback(() => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    setValue(0);
  }, []);

  const start = useCallback(
    (duration: number, onComplete?: () => void | Promise<void>) => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
      setValue(duration);
      let remaining = duration;
      timerRef.current = window.setInterval(() => {
        remaining -= 1;
        if (remaining > 0) {
          setValue(remaining);
          return;
        }
        clear();
        if (onComplete) {
          Promise.resolve(onComplete()).catch((err) => console.error("countdown completion failed", err));
        }
      }, 1000);
    },
    [clear],
  );

  useEffect(() => () => clear(), [clear]);

  return { value, start, cancel: clear };
};
