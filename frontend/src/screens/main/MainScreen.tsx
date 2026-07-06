import { useCallback, type FC } from "react";
import { UpdateParams } from "../../../wailsjs/go/main/App";
import { config as backendConfig } from "../../../wailsjs/go/models";
import { ScreenShell } from "../../components/ScreenShell";
import { useParams } from "../../state/useParams";
import { useRunning } from "../../state/useRunning";
import { useStatus } from "../../state/useStatus";
import { CameraPreview } from "./components/CameraPreview";
import { ClickModeControls } from "./components/ClickModeControls";
import { PrimaryActions } from "./components/PrimaryActions";
import { StatusHeader } from "./components/StatusHeader";
import { useRecenter } from "./hooks/useRecenter";
import { useDwellHover } from "./hooks/useDwellHover";

type MainScreenProps = {
  onOpenSettings: () => void;
  onStart: () => Promise<void> | void;
  onStop: () => Promise<void> | void;
};

export const MainScreen: FC<MainScreenProps> = ({ onOpenSettings, onStart, onStop }) => {
  const { params, setParams } = useParams();
  const { isRunning } = useRunning();
  const { status } = useStatus();
  const { countdown, handleRecenter } = useRecenter();
  const { onHoverStart, onHoverEnd } = useDwellHover();

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

  const toggleDwell = useCallback(async () => {
    const next = { ...params, dwellEnabled: !params.dwellEnabled };
    setParams(next);
    try {
      await UpdateParams(next as unknown as backendConfig.Params);
    } catch (err) {
      console.error("update params failed", err);
    }
  }, [params, setParams]);

  return (
    <ScreenShell header={<StatusHeader lost={status.lost} onOpenSettings={onOpenSettings} />} mainClassName="gap-4">
      <CameraPreview />
      <div className="grid gap-3 text-sm">
        <PrimaryActions
          isRunning={isRunning}
          recenterCountdown={countdown}
          onToggleRun={handleStartStop}
          onRecenter={handleRecenter}
        />
        <ClickModeControls
          dwellEnabled={params.dwellEnabled}
          onToggleDwell={toggleDwell}
          onEnableDwellHoverStart={onHoverStart}
          onEnableDwellHoverEnd={onHoverEnd}
        />
      </div>
    </ScreenShell>
  );
};
