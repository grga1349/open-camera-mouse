import type { FC } from "react";
import { Button } from "../../../components/Button";

type PrimaryActionsProps = {
  isRunning: boolean;
  isTransitioning: boolean;
  recenterCountdown: number;
  onToggleRun: () => void;
  onRecenter: () => void;
};

export const PrimaryActions: FC<PrimaryActionsProps> = ({
  isRunning,
  isTransitioning,
  recenterCountdown,
  onToggleRun,
  onRecenter,
}) => (
  <div className="grid gap-3">
    <Button variant="action" fullWidth onClick={onToggleRun} disabled={isTransitioning}>
      {isRunning ? "Stop" : "Start"}
    </Button>
    <Button fullWidth onClick={onRecenter} disabled={recenterCountdown > 0}>
      {recenterCountdown > 0 ? `Recenter in ${recenterCountdown}` : "Recenter"}
    </Button>
  </div>
);
