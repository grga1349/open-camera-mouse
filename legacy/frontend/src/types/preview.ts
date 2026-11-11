export type TrackingOverlay = {
  x: number;
  y: number;
  templateSizePx: number;
  lost: boolean;
};

export type PreviewFrame = {
  dataUrl: string;
  width: number;
  height: number;
  tracking: TrackingOverlay | null;
};
