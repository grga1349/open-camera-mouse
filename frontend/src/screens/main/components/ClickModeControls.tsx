import type {FC} from 'react';
import {Button} from '../../../components/Button';

type ClickModeControlsProps = {
    dwellEnabled: boolean;
    rightClickEnabled: boolean;
    onToggleDwell: () => void;
    onEnableDwellHoverStart: () => void;
    onEnableDwellHoverEnd: () => void;
    onToggleRightClick: () => void;
};

export const ClickModeControls: FC<ClickModeControlsProps> = ({
    dwellEnabled,
    rightClickEnabled,
    onToggleDwell,
    onEnableDwellHoverStart,
    onEnableDwellHoverEnd,
    onToggleRightClick,
}) => (
    <div className="grid grid-cols-2 gap-3">
        <Button
            fullWidth
            onClick={onToggleDwell}
            onMouseEnter={onEnableDwellHoverStart}
            onMouseLeave={onEnableDwellHoverEnd}
            title="Enable dwell clicking"
        >
            Dwell {dwellEnabled ? 'On' : 'Off'}
        </Button>
        <Button fullWidth onClick={onToggleRightClick}>
            Right Click {rightClickEnabled ? 'On' : 'Off'}
        </Button>
    </div>
);

