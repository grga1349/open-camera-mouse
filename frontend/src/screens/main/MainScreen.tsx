import { useCallback, type FC } from "react";
import { UpdateParams } from "../../../wailsjs/go/main/App";
import { config as backendConfig } from "../../../wailsjs/go/models";
import { ScreenShell } from "../../components/layout/ScreenShell";
import { useParams } from "../../state/useParams";
import { useRunning } from "../../state/useRunning";
import { useTelemetry } from "../../state/useTelemetry";
import { usePreview } from "../../state/usePreview";
import { CameraPreview } from "./components/CameraPreview";
import { ClickModeControls } from "./components/ClickModeControls";
import { PrimaryActions } from "./components/PrimaryActions";
import { StatusHeader } from "./components/StatusHeader";
import { useRecenter } from "./hooks/useRecenter";
import { useDwellHover } from "./hooks/useDwellHover";
import { usePickPoint } from "./hooks/usePickPoint";

type MainScreenProps = {
  onOpenSettings: () => void;
  onStart: () => Promise<void> | void;
  onStop: () => Promise<void> | void;
};

export const MainScreen: FC<MainScreenProps> = ({ onOpenSettings, onStart, onStop }) => {
  const { params, setParams } = useParams();
  const { isRunning } = useRunning();
  const { telemetry } = useTelemetry();
  const { preview } = usePreview();
  const { countdown, handleRecenter } = useRecenter();
  const { onHoverStart, onHoverEnd } = useDwellHover();
  const { onSelectPoint } = usePickPoint();

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

  const updateClicking = useCallback(
    async (updates: Partial<typeof params.clicking>) => {
      const next = { ...params, clicking: { ...params.clicking, ...updates } };
      setParams(next);
      try {
        await UpdateParams(next as unknown as backendConfig.AllParams);
      } catch (err) {
        console.error("update params failed", err);
      }
    },
    [params, setParams],
  );

  const toggleDwell = () => {
    void updateClicking({ dwellEnabled: !params.clicking.dwellEnabled });
  };

  const toggleRightClick = () => {
    void updateClicking({ rightClickToggle: !params.clicking.rightClickToggle });
  };

  return (
    <ScreenShell
      header={<StatusHeader lost={telemetry.lost} fps={telemetry.fps} onOpenSettings={onOpenSettings} />}
      mainClassName="gap-4"
    >
      <CameraPreview preview={preview} onSelectPoint={onSelectPoint} />
      <div className="grid gap-3 text-sm">
        <PrimaryActions
          isRunning={isRunning}
          recenterCountdown={countdown}
          onToggleRun={handleStartStop}
          onRecenter={handleRecenter}
        />
        <ClickModeControls
          dwellEnabled={params.clicking.dwellEnabled}
          rightClickEnabled={params.clicking.rightClickToggle}
          onToggleDwell={toggleDwell}
          onEnableDwellHoverStart={onHoverStart}
          onEnableDwellHoverEnd={onHoverEnd}
          onToggleRightClick={toggleRightClick}
        />
      </div>
    </ScreenShell>
  );
};
