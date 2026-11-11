import { useEffect, useState } from "react";
import { EventsOn } from "../wailsjs/runtime/runtime";
import { GetParams, Start, Stop } from "../wailsjs/go/main/App";
import { MainScreen } from "./screens/main/MainScreen";
import { SettingsScreen } from "./screens/settings/SettingsScreen";
import { useParams } from "./state/useParams";
import { useParamsSync } from "./state/useParamsSync";
import { useRunning } from "./state/useRunning";
import { useAppError } from "./state/useAppError";
import { publishPreview } from "./lib/previewBus";
import { fromBackendParams } from "./lib/params";
import { useStatus } from "./state/useStatus";
import type { Params } from "./types/params";
import { ErrorBanner } from "./components/ErrorBanner";

type Screen = "main" | "settings";

function App() {
  const [screen, setScreen] = useState<Screen>("main");
  const { setParams } = useParams();
  const { confirmParams } = useParamsSync();
  const { setRunning } = useRunning();
  const { setStatus } = useStatus();
  const { error, reportError, clearError } = useAppError();

  useEffect(() => {
    let offPreview: (() => void) | undefined;
    let offStatus: (() => void) | undefined;
    let offRunning: (() => void) | undefined;

    GetParams()
      .then((res) => setParams(fromBackendParams(res)))
      .catch((err) => {
        console.error("failed to load params", err);
        reportError("Could not load settings.");
      });

    offPreview = EventsOn("preview:frame", (frame) => {
      if (!frame?.dataUrl) return;
      publishPreview({
        dataUrl: frame.dataUrl,
        width: frame.width ?? 0,
        height: frame.height ?? 0,
        tracking: frame.tracking ?? null,
      });
    });

    offStatus = EventsOn("status:update", (payload) => {
      setStatus({ lost: payload?.lost ?? false });
    });

    offRunning = EventsOn("service:running", (payload) => {
      setRunning(Boolean(payload));
    });

    return () => {
      offPreview?.();
      offStatus?.();
      offRunning?.();
    };
  }, [setParams, setStatus, setRunning, reportError]);

  const openSettings = () => setScreen("settings");
  const closeSettings = () => setScreen("main");

  const handleSettingsSave = async (next: Params) => {
    await confirmParams(next);
    closeSettings();
  };

  // isRunning is driven entirely by the "service:running" event above, not
  // set optimistically here — the backend is the single source of truth,
  // so a failed Start/Stop can't leave the UI showing the wrong state.
  const handleStart = async () => {
    try {
      await Start();
      clearError();
    } catch (err) {
      console.error("start failed", err);
      reportError("Could not start tracking.");
      throw err;
    }
  };

  const handleStop = async () => {
    try {
      await Stop();
      clearError();
    } catch (err) {
      console.error("stop failed", err);
      reportError("Could not stop tracking.");
      throw err;
    }
  };

  return (
    <>
      {error && (
        <div className="fixed inset-x-0 top-0 z-50 mx-auto max-w-sm px-5 pt-4">
          <ErrorBanner message={error} onDismiss={clearError} />
        </div>
      )}
      {screen === "main" ? (
        <MainScreen onOpenSettings={openSettings} onStart={handleStart} onStop={handleStop} />
      ) : (
        <SettingsScreen onCancel={closeSettings} onSave={handleSettingsSave} />
      )}
    </>
  );
}

export default App;
