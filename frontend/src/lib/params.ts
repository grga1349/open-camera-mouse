import type { config as backendConfig } from "../../wailsjs/go/models";
import type { Params } from "../types/params";

export const fromBackendParams = (params: backendConfig.Params): Params => ({
  templateSizePx: params.templateSizePx,
  gainMultiplier: params.gainMultiplier,
  smoothing: params.smoothing,
  dwellEnabled: params.dwellEnabled,
  dwellTimeMs: params.dwellTimeMs,
  autoStart: params.autoStart,
  rightClickEnabled: params.rightClickEnabled,
});

export const toBackendParams = (params: Params): backendConfig.Params => ({
  templateSizePx: params.templateSizePx,
  gainMultiplier: params.gainMultiplier,
  smoothing: params.smoothing,
  dwellEnabled: params.dwellEnabled,
  dwellTimeMs: params.dwellTimeMs,
  autoStart: params.autoStart,
  rightClickEnabled: params.rightClickEnabled,
});
