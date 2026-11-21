/**
 * The latest release, read once per build (ADR-0023 amendment 2). Visitors' browsers never
 * call the GitHub API; the version and download URLs are baked into the HTML, and the
 * site is rebuilt when a release is published (`website.yml`).
 *
 * - `src/data/latest-release.json` (gitignored), when present: the API response, written by
 *   a CI step of its own (`gh api`), so the token never reaches the build's dependencies.
 * - Otherwise (local builds): fetch `releases/latest` anonymously.
 *
 * Any failure logs a warning and returns `null`; the page then links to the Releases page,
 * as before Phase 4.
 */
import { parseRelease, type Release } from "./release";

const API = "https://api.github.com/repos/grga1349/open-camera-mouse/releases/latest";
const FILE = "src/data/latest-release.json";

// Empty when the file doesn't exist; Vite resolves this at build time, no Node API needed.
const fromCi = Object.values(
  import.meta.glob<unknown>("../data/latest-release.json", { eager: true, import: "default" }),
);

let cached: Promise<Release | null> | undefined;

/** Shared by every component in one build, so the API is asked at most once. */
export function latestRelease(): Promise<Release | null> {
  cached ??= load();
  return cached;
}

async function load(): Promise<Release | null> {
  const file = fromCi.length > 0;
  try {
    const data: unknown = file ? fromCi[0] : await fetchLatest();
    const release = parseRelease(data);
    if (release) console.info(`[release] latest is v${release.version}`);
    else console.warn("[release] no usable latest release; downloads link to the Releases page");
    return release;
  } catch (error) {
    console.warn(`[release] ${file ? FILE : API}: ${String(error)}; downloads link to the Releases page`);
    return null;
  }
}

async function fetchLatest(): Promise<unknown> {
  const response = await fetch(API, {
    headers: { Accept: "application/vnd.github+json" },
    signal: AbortSignal.timeout(10_000),
  });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}
