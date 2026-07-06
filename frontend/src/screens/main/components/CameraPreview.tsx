import { useEffect, useMemo, useRef, useState, type FC } from "react";
import type { PreviewFrame, TrackingOverlay } from "../../../types/preview";
import { getLatestPreview, subscribePreview } from "../../../lib/previewBus";
import { computeCoverTransform, sourceToContainerPoint, type CoverTransform } from "../../../lib/coverTransform";
import { usePickPoint } from "../hooks/usePickPoint";

const PREVIEW_WIDTH_PX = 360;
const PREVIEW_HEIGHT_PX = 270;

const applyOverlay = (el: HTMLDivElement, transform: CoverTransform | null, tracking: TrackingOverlay | null) => {
  if (!transform || !tracking) {
    el.style.display = "none";
    return;
  }
  const boxSize = tracking.templateSizePx * transform.scale;
  const topLeft = sourceToContainerPoint(
    transform,
    tracking.x - tracking.templateSizePx / 2,
    tracking.y - tracking.templateSizePx / 2,
  );
  el.style.display = "block";
  el.style.left = `${topLeft.x}px`;
  el.style.top = `${topLeft.y}px`;
  el.style.width = `${boxSize}px`;
  el.style.height = `${boxSize}px`;
  el.style.borderColor = tracking.lost ? "#f87171" : "#34d399";
};

export const CameraPreview: FC = () => {
  const imgRef = useRef<HTMLImageElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);
  const placeholderRef = useRef<HTMLDivElement>(null);
  const hasFrameRef = useRef(false);
  const sourceSizeRef = useRef({ width: 0, height: 0 });
  const [sourceSize, setSourceSize] = useState({ width: 0, height: 0 });
  const { onSelectPoint } = usePickPoint();

  const transform = useMemo(
    () => computeCoverTransform(PREVIEW_WIDTH_PX, PREVIEW_HEIGHT_PX, sourceSize.width, sourceSize.height),
    [sourceSize],
  );

  useEffect(() => {
    const applyFrame = (frame: PreviewFrame) => {
      if (imgRef.current) imgRef.current.src = frame.dataUrl;

      if (!hasFrameRef.current) {
        hasFrameRef.current = true;
        if (placeholderRef.current) placeholderRef.current.style.display = "none";
      }

      if (frame.width !== sourceSizeRef.current.width || frame.height !== sourceSizeRef.current.height) {
        sourceSizeRef.current = { width: frame.width, height: frame.height };
        setSourceSize(sourceSizeRef.current);
        return;
      }

      if (overlayRef.current) applyOverlay(overlayRef.current, transform, frame.tracking);
    };

    const initial = getLatestPreview();
    if (initial) applyFrame(initial);

    return subscribePreview(applyFrame);
  }, [transform]);

  return (
    <div className="flex justify-center">
      <div
        className="relative overflow-hidden rounded-3xl border border-zinc-900 bg-zinc-950"
        style={{ width: PREVIEW_WIDTH_PX, height: PREVIEW_HEIGHT_PX }}
        onClick={onSelectPoint}
      >
        <img ref={imgRef} alt="camera preview" className="absolute inset-0 h-full w-full object-cover" />
        <div ref={overlayRef} className="pointer-events-none absolute border-2" style={{ display: "none" }} />
        <div
          ref={placeholderRef}
          className="absolute inset-0 flex items-center justify-center bg-zinc-950 text-sm text-zinc-500"
        >
          Preview unavailable
        </div>
      </div>
    </div>
  );
};
