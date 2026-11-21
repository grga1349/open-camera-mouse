/**
 * Which download fits the visitor's system, from what the browser says about itself.
 * Operating system only: the CPU is never guessed (every Mac browser reports "Intel Mac OS
 * X", and architecture hints are unreliable), so the cards' requirements and notes tell
 * Intel Mac, ARM and Wayland users what isn't supported. Pure, for `bun test`; the DOM
 * wiring is in `DownloadEnhancer.astro`.
 */
import type { PlatformId } from "./platforms";

export type Detection =
  | { kind: "supported"; id: PlatformId }
  /** A phone, tablet or ChromeOS: Open Camera Mouse is a desktop app. */
  | { kind: "unsupported" }
  | { kind: "unknown" };

export interface BrowserInfo {
  userAgent: string;
  /** `navigator.userAgentData.platform` where available (Chromium), e.g. "macOS". */
  uaPlatform?: string;
  /** `navigator.maxTouchPoints`; tells an iPad in desktop mode from a Mac. */
  maxTouchPoints?: number;
}

const supported = (id: PlatformId): Detection => ({ kind: "supported", id });

export function detectPlatform({ userAgent, uaPlatform, maxTouchPoints = 0 }: BrowserInfo): Detection {
  const hint = uaPlatform?.trim().toLowerCase();
  if (hint) {
    if (hint === "macos") return supported("macos");
    if (hint === "windows") return supported("windows");
    if (hint === "linux") return supported("linux");
    if (["android", "ios", "chrome os", "chromeos"].includes(hint)) return { kind: "unsupported" };
    return { kind: "unknown" };
  }

  // Order matters: Android and ChromeOS also say "Linux", iOS also says "Mac OS X".
  if (/Android|iPhone|iPad|iPod|CrOS/.test(userAgent)) return { kind: "unsupported" };
  if (/Windows/.test(userAgent)) return supported("windows");
  if (/Macintosh|Mac OS X/.test(userAgent)) {
    return maxTouchPoints > 1 ? { kind: "unsupported" } : supported("macos");
  }
  if (/Linux|X11/.test(userAgent)) return supported("linux");
  return { kind: "unknown" };
}
