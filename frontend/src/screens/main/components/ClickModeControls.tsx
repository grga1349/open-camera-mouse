import type { FC } from "react";
import { Button } from "../../../components/Button";

type ClickModeControlsProps = {
  dwellEnabled: boolean;
  onToggleDwell: () => void;
  onEnableDwellHoverStart: () => void;
  onEnableDwellHoverEnd: () => void;
};

export const ClickModeControls: FC<ClickModeControlsProps> = ({
  dwellEnabled,
  onToggleDwell,
  onEnableDwellHoverStart,
  onEnableDwellHoverEnd,
}) => (
  <div className="grid grid-cols-1 gap-3">
    <Button
      fullWidth
      onClick={onToggleDwell}
      onMouseEnter={onEnableDwellHoverStart}
      onMouseLeave={onEnableDwellHoverEnd}
      title="Enable dwell clicking"
    >
      Dwell {dwellEnabled ? "On" : "Off"}
    </Button>
  </div>
);
