export type TrackerState = "idle" | "tracking" | "lost";

export type Telemetry = {
  fps: number;
  score: number;
  state: TrackerState;
  trackingOn: boolean;
  lost: boolean;
  posX: number | null;
  posY: number | null;
};
