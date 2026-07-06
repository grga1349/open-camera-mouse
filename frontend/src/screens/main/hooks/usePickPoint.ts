import { useCallback } from "react";
import type { MouseEvent } from "react";
import { PickPoint } from "../../../../wailsjs/go/main/App";
import { getLatestPreview } from "../../../lib/previewBus";
import { computeCoverTransform, containerToSourcePoint } from "../../../lib/coverTransform";
import { useAppError } from "../../../state/useAppError";
import { PREVIEW_WIDTH_PX, PREVIEW_HEIGHT_PX } from "../previewLayout";

export const usePickPoint = () => {
  const { reportError, clearError } = useAppError();

  const onSelectPoint = useCallback(
    async (event: MouseEvent<HTMLDivElement>) => {
      const preview = getLatestPreview();
      if (!preview) return;
      // Use the fixed layout size (not the measured DOM rect) so this exactly
      // matches the transform CameraPreview uses to draw the overlay box —
      // any drift between "declared" and "measured" container size would
      // otherwise throw off the click position relative to the drawn box.
      const rect = event.currentTarget.getBoundingClientRect();
      const transform = computeCoverTransform(PREVIEW_WIDTH_PX, PREVIEW_HEIGHT_PX, preview.width, preview.height);
      if (!transform) return;
      const relX = event.clientX - rect.left;
      const relY = event.clientY - rect.top;
      const source = containerToSourcePoint(transform, relX, relY);
      const x = Math.max(0, Math.min(preview.width - 1, Math.round(source.x)));
      const y = Math.max(0, Math.min(preview.height - 1, Math.round(source.y)));
      try {
        await PickPoint(x, y);
        clearError();
      } catch (err) {
        console.error("pick point failed", err);
        reportError("Could not set tracking point — is tracking running?");
      }
    },
    [reportError, clearError],
  );

  return { onSelectPoint };
};
