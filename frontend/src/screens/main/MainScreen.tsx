import { useEffect, useRef, useCallback, type FC, type MouseEvent } from "react";
import { ScreenShell } from "../../components/layout/ScreenShell";
import { useAppStore } from "../../state/useAppStore";
import { config as backendConfig } from "../../../wailsjs/go/models";
import { Recenter, SetPickPoint, ToggleTracking, UpdateParams } from "../../../wailsjs/go/main/App";
import { EventsOn } from "../../../wailsjs/runtime/runtime";
import { CameraPreview } from "./components/CameraPreview";
import { ClickModeControls } from "./components/ClickModeControls";
import { PrimaryActions } from "./components/PrimaryActions";
import { StatusHeader } from "./components/StatusHeader";
import { useRecenterCountdown } from "./hooks/useRecenterCountdown";

type MainScreenProps = {
  onOpenSettings: () => void;
  onStart: () => Promise<void> | void;
  onStop: () => Promise<void> | void;
};

export const MainScreen: FC<MainScreenProps> = ({ onOpenSettings, onStart, onStop }) => {
  const { params, telemetry, preview, isRunning, setParams } = useAppStore();
  const dwellHoverRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const { value: recenterCountdown, start: startCountdown } = useRecenterCountdown();

  useEffect(
    () => () => {
      if (dwellHoverRef.current) {
        clearTimeout(dwellHoverRef.current);
      }
    },
    [],
  );

  const handleStartStop = async () => {
    try {
      if (isRunning) {
        await onStop();
      } else {
        await onStart();
      }
    } catch (err) {
      console.error("start/stop failed", err);
    }
  };

  const handleRecenter = useCallback(async () => {
    if (recenterCountdown > 0) {
      return;
    }

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
  }, [recenterCountdown, telemetry.trackingOn, startCountdown]);

  useEffect(() => {
    const off = EventsOn("recenter:hotkey", () => {
      void handleRecenter();
    });
    return off;
  }, [handleRecenter]);

  const handlePreviewClick = async (event: MouseEvent<HTMLDivElement>) => {
    if (!preview) {
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    const relX = event.clientX - rect.left;
    const relY = event.clientY - rect.top;
    const xRatio = preview.width > 0 ? preview.width / rect.width : 1;
    const yRatio = preview.height > 0 ? preview.height / rect.height : 1;
    const x = Math.max(0, Math.min(preview.width, Math.round(relX * xRatio)));
    const y = Math.max(0, Math.min(preview.height, Math.round(relY * yRatio)));
    try {
      await SetPickPoint(x, y);
    } catch (err) {
      console.error("set pick point failed", err);
    }
  };

  const updateClicking = async (updates: Partial<typeof params.clicking>) => {
    const next = {
      ...params,
      clicking: {
        ...params.clicking,
        ...updates,
      },
    };
    setParams(next);
    try {
      await UpdateParams(next as unknown as backendConfig.AllParams);
    } catch (err) {
      console.error("update params failed", err);
    }
  };

  const toggleDwell = () => {
    if (dwellHoverRef.current) {
      clearTimeout(dwellHoverRef.current);
      dwellHoverRef.current = null;
    }
    void updateClicking({ dwellEnabled: !params.clicking.dwellEnabled });
  };

  const enableDwell = () => {
    if (!params.clicking.dwellEnabled) {
      void updateClicking({ dwellEnabled: true });
    }
  };

  const toggleRightClick = () => {
    void updateClicking({ rightClickToggle: !params.clicking.rightClickToggle });
  };

  const handleDwellHoverStart = () => {
    if (params.clicking.dwellEnabled || dwellHoverRef.current) {
      return;
    }
    dwellHoverRef.current = window.setTimeout(() => {
      dwellHoverRef.current = null;
      enableDwell();
    }, 500);
  };

  const handleDwellHoverEnd = () => {
    if (dwellHoverRef.current) {
      clearTimeout(dwellHoverRef.current);
      dwellHoverRef.current = null;
    }
  };

  return (
    <ScreenShell
      header={<StatusHeader lost={telemetry.lost} fps={telemetry.fps} onOpenSettings={onOpenSettings} />}
      mainClassName="gap-4"
    >
      <CameraPreview preview={preview} onSelectPoint={handlePreviewClick} />
      <div className="grid gap-3 text-sm">
        <PrimaryActions
          isRunning={isRunning}
          recenterCountdown={recenterCountdown}
          onToggleRun={handleStartStop}
          onRecenter={handleRecenter}
        />
        <ClickModeControls
          dwellEnabled={params.clicking.dwellEnabled}
          rightClickEnabled={params.clicking.rightClickToggle}
          onToggleDwell={toggleDwell}
          onEnableDwellHoverStart={handleDwellHoverStart}
          onEnableDwellHoverEnd={handleDwellHoverEnd}
          onToggleRightClick={toggleRightClick}
        />
      </div>
    </ScreenShell>
  );
};
