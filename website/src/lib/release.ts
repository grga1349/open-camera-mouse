/**
 * Turns the GitHub `releases/latest` response into what the download UI offers: the version
 * and, per platform, the release assets matched by the suffixes in `platforms.ts`.
 *
 * Pure: the response is fetched once at build time (`latest-release.ts`, ADR-0023
 * amendment 2), never by the visitor's browser. Anything unexpected yields fewer downloads
 * or `null`, and the page then links to the Releases page instead.
 */
import { PLATFORMS, type Platform, type PlatformId } from "./platforms";
import { REPO_URL } from "./url";

export interface Download {
  suffix: string;
  label: string;
  /** Asset file name, e.g. `open-camera-mouse-0.4.0-macos-arm64.dmg`. */
  name: string;
  /** Direct download (`browser_download_url`). */
  url: string;
}

export interface Release {
  /** Without the `v`, e.g. `0.4.0`. */
  version: string;
  tag: string;
  /** The release's page on GitHub (notes, all files, SHA256SUMS). */
  pageUrl: string;
  /** Per platform, in `platforms.ts` preference order; empty when the release lacks them. */
  downloads: Record<PlatformId, Download[]>;
}

/** Tags the release workflow accepts: `v` + the Cargo version (release.yml "version" step). */
const TAG = /^v(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/;

const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

export function parseRelease(data: unknown, platforms: Platform[] = PLATFORMS): Release | null {
  if (!isRecord(data) || data.draft === true || data.prerelease === true) return null;
  const tag = typeof data.tag_name === "string" ? data.tag_name : "";
  const match = TAG.exec(tag);
  if (!match || !Array.isArray(data.assets)) return null;
  const version = match[1]!;

  // name → URL, only for assets whose URL is exactly this repository's download for this tag.
  const assets = new Map<string, string>();
  for (const a of data.assets) {
    if (!isRecord(a) || typeof a.name !== "string") continue;
    const expected = `${REPO_URL}/releases/download/${tag}/${a.name}`;
    if (a.browser_download_url === expected) assets.set(a.name, expected);
  }

  const downloads = {} as Record<PlatformId, Download[]>;
  let total = 0;
  for (const p of platforms) {
    downloads[p.id] = p.assets.flatMap(({ suffix, label }) => {
      const name = `open-camera-mouse-${version}-${suffix}`;
      const url = assets.get(name);
      return url ? [{ suffix, label, name, url }] : [];
    });
    total += downloads[p.id].length;
  }
  if (total === 0) return null;

  return { version, tag, pageUrl: `${REPO_URL}/releases/tag/${tag}`, downloads };
}
