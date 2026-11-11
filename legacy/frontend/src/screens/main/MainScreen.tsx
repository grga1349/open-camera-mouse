import { useCallback, useState, type FC } from "react";
import { ScreenShell } from "../../components/ScreenShell";
import { useParams } from "../../state/useParams";
import { useParamsSync } from "../../state/useParamsSync";
import { useRunning } from "../../state/useRunning";
import { useStatus } from "../../state/useStatus";
import { CameraPreview } from "./components/CameraPreview";
import { ClickModeControls } from "./components/ClickModeControls";
import { PrimaryActions } from "./components/PrimaryActions";
import { StatusHeader } from "./components/StatusHeader";
import { useRecenter } from "./hooks/useRecenter";

type MainScreenProps = {
  onOpenSettings: () => void;
  onStart: () => Promise<void> | void;
  onStop: () => Promise<void> | void;
};

export const MainScreen: FC<MainScreenProps> = ({ onOpenSettings, onStart, onStop }) => {
  const { params } = useParams();
  const { setParamsOptimistic } = useParamsSync();
  const { isRunning } = useRunning();
  const { status } = useStatus();
  const { countdown, isRecentering, handleRecenter } = useRecenter();
  const [isTransitioning, setIsTransitioning] = useState(false);

  const handleStartStop = useCallback(async () => {
    if (isTransitioning) return;
    setIsTransitioning(true);
    try {
      if (isRunning) {
        await onStop();
      } else {
        await onStart();
      }
    } catch (err) {
      console.error("start/stop failed", err);
    } finally {
      setIsTransitioning(false);
    }
  }, [isTransitioning, isRunning, onStart, onStop]);

  const toggleDwell = useCallback(async () => {
    const next = { ...params, dwellEnabled: !params.dwellEnabled };
    try {
      await setParamsOptimistic(next, "Could not update dwell clicking.");
    } catch (err) {
      console.error("update params failed", err);
    }
  }, [params, setParamsOptimistic]);

  const toggleRightClick = useCallback(async () => {
    const next = { ...params, rightClickEnabled: !params.rightClickEnabled };
    try {
      await setParamsOptimistic(next, "Could not update click button.");
    } catch (err) {
      console.error("update params failed", err);
    }
  }, [params, setParamsOptimistic]);

  return (
    <ScreenShell header={<StatusHeader lost={status.lost} onOpenSettings={onOpenSettings} />} mainClassName="gap-4">
      <CameraPreview isRecentering={isRecentering} />
      <div className="grid gap-3 text-sm">
        <PrimaryActions
          isRunning={isRunning}
          isTransitioning={isTransitioning}
          recenterCountdown={countdown}
          onToggleRun={handleStartStop}
          onRecenter={handleRecenter}
        />
        <ClickModeControls
          dwellEnabled={params.dwellEnabled}
          onToggleDwell={toggleDwell}
          rightClickEnabled={params.rightClickEnabled}
          onToggleRightClick={toggleRightClick}
        />
      </div>
    </ScreenShell>
  );
};
