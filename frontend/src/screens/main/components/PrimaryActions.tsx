import type { FC } from "react";
import { Button } from "../../../components/Button";

type PrimaryActionsProps = {
  isRunning: boolean;
  recenterCountdown: number;
  onToggleRun: () => void;
  onRecenter: () => void;
};

export const PrimaryActions: FC<PrimaryActionsProps> = ({ isRunning, recenterCountdown, onToggleRun, onRecenter }) => (
  <div className="grid gap-3">
    <Button variant="action" fullWidth onClick={onToggleRun}>
      {isRunning ? "Pause" : "Start"}
    </Button>
    <Button fullWidth onClick={onRecenter} disabled={recenterCountdown > 0}>
      {recenterCountdown > 0 ? `Recenter in ${recenterCountdown}` : "Recenter"}
    </Button>
  </div>
);
