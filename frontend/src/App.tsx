import { useEffect, useState } from "react";
import { EventsOn } from "../wailsjs/runtime/runtime";
import { GetParams, SaveParams, Start, Stop } from "../wailsjs/go/main/App";
import { config as backendConfig } from "../wailsjs/go/models";
import { MainScreen } from "./screens/main/MainScreen";
import { SettingsScreen } from "./screens/settings/SettingsScreen";
import { useParams } from "./state/useParams";
import { useRunning } from "./state/useRunning";
import { useTelemetry } from "./state/useTelemetry";
import { usePreview } from "./state/usePreview";
import type { AllParams } from "./types/params";
import { deepClone } from "./lib/clone";

type Screen = "main" | "settings";

function App() {
  const [screen, setScreen] = useState<Screen>("main");
  const { setParams } = useParams();
  const { setRunning } = useRunning();
  const { setTelemetry } = useTelemetry();
  const { setPreview } = usePreview();

  useEffect(() => {
    let offPreview: (() => void) | undefined;
    let offTelemetry: (() => void) | undefined;
    let offParams: (() => void) | undefined;
    let offRunning: (() => void) | undefined;

    GetParams()
      .then((res) => setParams(deepClone(res) as unknown as AllParams))
      .catch((err) => console.error("failed to load params", err));

    offPreview = EventsOn("preview:frame", (frame) => {
      if (!frame?.data) return;
      setPreview({
        data: frame.data,
        width: frame.width ?? 0,
        height: frame.height ?? 0,
        timestamp: frame.timestamp ?? new Date().toISOString(),
      });
    });

    offTelemetry = EventsOn("telemetry:state", (payload) => {
      const lost = payload?.lost ?? false;
      const tracking = payload?.tracking ?? false;
      setTelemetry({
        fps: payload?.fps ?? 0,
        score: payload?.score ?? 0,
        state: lost ? "lost" : tracking ? "tracking" : "idle",
        trackingOn: tracking,
        lost,
        posX: payload?.posX ?? null,
        posY: payload?.posY ?? null,
      });
    });

    offParams = EventsOn("params:update", (payload) => {
      if (!payload) return;
      setParams(deepClone(payload) as unknown as AllParams);
    });

    offRunning = EventsOn("service:running", (payload) => {
      setRunning(Boolean(payload));
    });

    return () => {
      offPreview?.();
      offTelemetry?.();
      offParams?.();
      offRunning?.();
    };
  }, [setParams, setPreview, setTelemetry, setRunning]);

  const openSettings = () => setScreen("settings");
  const closeSettings = () => setScreen("main");

  const handleSettingsSave = async (next: AllParams) => {
    await SaveParams(next as unknown as backendConfig.AllParams);
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
