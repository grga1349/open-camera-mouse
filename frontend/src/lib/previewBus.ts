import type { PreviewFrame } from "../types/preview";

type Listener = (frame: PreviewFrame) => void;

let latest: PreviewFrame | null = null;
const listeners = new Set<Listener>();

export const publishPreview = (frame: PreviewFrame) => {
  latest = frame;
  listeners.forEach((listener) => {
    try {
      listener(frame);
    } catch (err) {
      console.error("previewBus: listener threw", err);
    }
  });
};

export const subscribePreview = (listener: Listener): (() => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

export const getLatestPreview = (): PreviewFrame | null => latest;
