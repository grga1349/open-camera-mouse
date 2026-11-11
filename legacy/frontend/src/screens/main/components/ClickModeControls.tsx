import type { FC } from "react";
import { Button } from "../../../components/Button";

type ClickModeControlsProps = {
  dwellEnabled: boolean;
  onToggleDwell: () => void;
  rightClickEnabled: boolean;
  onToggleRightClick: () => void;
};

export const ClickModeControls: FC<ClickModeControlsProps> = ({
  dwellEnabled,
  onToggleDwell,
  rightClickEnabled,
  onToggleRightClick,
}) => (
  <div className="grid grid-cols-2 gap-3">
    <Button fullWidth onClick={onToggleDwell} title="Enable dwell clicking">
      Dwell {dwellEnabled ? "On" : "Off"}
    </Button>
    <Button fullWidth onClick={onToggleRightClick} title="Use right click instead of left click">
      Click: {rightClickEnabled ? "Right" : "Left"}
    </Button>
  </div>
);
