import { useCallback } from "react";
import type { MouseEvent } from "react";
import { PickPoint } from "../../../../wailsjs/go/main/App";
import { usePreview } from "../../../state/usePreview";

export const usePickPoint = () => {
  const { preview } = usePreview();

  const onSelectPoint = useCallback(
    async (event: MouseEvent<HTMLDivElement>) => {
      if (!preview) return;
      const rect = event.currentTarget.getBoundingClientRect();
      const relX = event.clientX - rect.left;
      const relY = event.clientY - rect.top;
      const xRatio = preview.width > 0 ? preview.width / rect.width : 1;
      const yRatio = preview.height > 0 ? preview.height / rect.height : 1;
      const x = Math.max(0, Math.min(preview.width, Math.round(relX * xRatio)));
      const y = Math.max(0, Math.min(preview.height, Math.round(relY * yRatio)));
      try {
        await PickPoint(x, y);
      } catch (err) {
        console.error("pick point failed", err);
      }
    },
    [preview],
  );

  return { onSelectPoint };
};
