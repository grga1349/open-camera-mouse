import { useCallback } from "react";
import type { MouseEvent } from "react";
import { PickPoint } from "../../../../wailsjs/go/main/App";
import { getLatestPreview } from "../../../lib/previewBus";
import { computeCoverTransform, containerToSourcePoint } from "../../../lib/coverTransform";

export const usePickPoint = () => {
  const onSelectPoint = useCallback(async (event: MouseEvent<HTMLDivElement>) => {
    const preview = getLatestPreview();
    if (!preview) return;
    const rect = event.currentTarget.getBoundingClientRect();
    const transform = computeCoverTransform(rect.width, rect.height, preview.width, preview.height);
    if (!transform) return;
    const relX = event.clientX - rect.left;
    const relY = event.clientY - rect.top;
    const source = containerToSourcePoint(transform, relX, relY);
    const x = Math.max(0, Math.min(preview.width, Math.round(source.x)));
    const y = Math.max(0, Math.min(preview.height, Math.round(source.y)));
    try {
      await PickPoint(x, y);
    } catch (err) {
      console.error("pick point failed", err);
    }
  }, []);

  return { onSelectPoint };
};
