import type { FC, MouseEvent } from "react";
import type { PreviewFrame } from "../../../types/preview";

type CameraPreviewProps = {
  preview: PreviewFrame | null;
  onSelectPoint: (event: MouseEvent<HTMLDivElement>) => void;
};

export const CameraPreview: FC<CameraPreviewProps> = ({ preview, onSelectPoint }) => {
  const tracking = preview?.tracking ?? null;

  return (
    <div className="flex justify-center">
      <div
        className="relative w-full max-w-[360px] overflow-hidden rounded-3xl border border-zinc-900 bg-zinc-950"
        style={{ aspectRatio: "4 / 3" }}
        onClick={onSelectPoint}
      >
        {preview ? (
          <>
            <img src={preview.dataUrl} alt="camera preview" className="h-full w-full object-cover" />
            {tracking && preview.width > 0 && (
              <div
                className="pointer-events-none absolute border-2"
                style={{
                  borderColor: tracking.lost ? "#f87171" : "#34d399",
                  left: `${((tracking.x - tracking.templateSizePx / 2) / preview.width) * 100}%`,
                  top: `${((tracking.y - tracking.templateSizePx / 2) / preview.height) * 100}%`,
                  width: `${(tracking.templateSizePx / preview.width) * 100}%`,
                  height: `${(tracking.templateSizePx / preview.height) * 100}%`,
                }}
              />
            )}
          </>
        ) : (
          <div className="flex h-full items-center justify-center text-sm text-zinc-500">Preview unavailable</div>
        )}
      </div>
    </div>
  );
};
