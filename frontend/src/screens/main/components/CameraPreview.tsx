import type { FC, MouseEvent } from "react";
import type { PreviewFrame } from "../../../state/useAppStore";

type CameraPreviewProps = {
  preview: PreviewFrame | null;
  onSelectPoint: (event: MouseEvent<HTMLDivElement>) => void;
};

export const CameraPreview: FC<CameraPreviewProps> = ({ preview, onSelectPoint }) => {
  const previewSrc = preview ? `data:image/jpeg;base64,${preview.data}` : null;
  return (
    <div className="flex justify-center">
      <div
        className="relative w-full max-w-[360px] overflow-hidden rounded-3xl border border-zinc-900 bg-zinc-950"
        style={{ aspectRatio: "4 / 3" }}
        onClick={onSelectPoint}
      >
        {previewSrc ? (
          <img src={previewSrc} alt="camera preview" className="h-full w-full object-cover" />
        ) : (
          <div className="flex h-full items-center justify-center text-sm text-zinc-500">Preview unavailable</div>
        )}
      </div>
    </div>
  );
};
