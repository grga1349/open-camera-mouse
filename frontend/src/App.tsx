import { useEffect, useState } from "react";
import { EventsOn } from "../wailsjs/runtime/runtime";
import { GetParams, Start, Stop, UpdateParams } from "../wailsjs/go/main/App";
import { config as backendConfig } from "../wailsjs/go/models";
import { MainScreen } from "./screens/main/MainScreen";
import { SettingsScreen } from "./screens/settings/SettingsScreen";
import { useParams } from "./state/useParams";
import { useRunning } from "./state/useRunning";
import { usePreview } from "./state/usePreview";
import { useStatus } from "./state/useStatus";
import type { Params } from "./types/params";
import { deepClone } from "./lib/clone";

type Screen = "main" | "settings";

function App() {
  const [screen, setScreen] = useState<Screen>("main");
  const { setParams } = useParams();
  const { setRunning } = useRunning();
  const { setPreview } = usePreview();
  const { setStatus } = useStatus();

  useEffect(() => {
    let offPreview: (() => void) | undefined;
    let offStatus: (() => void) | undefined;
    let offRunning: (() => void) | undefined;

    GetParams()
      .then((res) => setParams(deepClone(res) as unknown as Params))
      .catch((err) => console.error("failed to load params", err));

    offPreview = EventsOn("preview:frame", (frame) => {
      if (!frame?.dataUrl) return;
      setPreview({
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
  }, [setParams, setPreview, setStatus, setRunning]);

  const openSettings = () => setScreen("settings");
  const closeSettings = () => setScreen("main");

  const handleSettingsSave = async (next: Params) => {
    await UpdateParams(next as unknown as backendConfig.Params);
    setParams(next);
    closeSettings();
  };

  const handleStart = async () => {
    await Start();
    setRunning(true);
  };

  const handleStop = async () => {
    await Stop();
    setRunning(false);
  };

  return screen === "main" ? (
    <MainScreen onOpenSettings={openSettings} onStart={handleStart} onStop={handleStop} />
  ) : (
    <SettingsScreen onCancel={closeSettings} onSave={handleSettingsSave} />
  );
}

export default App;
