import { describe, expect, test } from "bun:test";
import { parseRelease } from "./release";

const DL = "https://github.com/grga1349/open-camera-mouse/releases/download";

/** The fields of a GitHub `releases/latest` response that the site reads. */
function apiRelease(tag: string, names: string[], extra: Record<string, unknown> = {}) {
  return {
    tag_name: tag,
    draft: false,
    prerelease: false,
    assets: names.map((name) => ({ name, browser_download_url: `${DL}/${tag}/${name}` })),
    ...extra,
  };
}

const V040 = [
  "open-camera-mouse-0.4.0-linux-x86_64.AppImage",
  "open-camera-mouse-0.4.0-linux-x86_64.tar.gz",
  "open-camera-mouse-0.4.0-macos-arm64.dmg",
  "open-camera-mouse-0.4.0-windows-x64-setup.exe",
  "SHA256SUMS",
];

describe("parseRelease", () => {
  test("maps every platform's assets, in the order platforms.ts prefers", () => {
    const r = parseRelease(apiRelease("v0.4.0", V040));
    expect(r).not.toBeNull();
    expect(r!.version).toBe("0.4.0");
    expect(r!.pageUrl).toBe("https://github.com/grga1349/open-camera-mouse/releases/tag/v0.4.0");
    expect(r!.downloads.macos.map((d) => d.url)).toEqual([
      `${DL}/v0.4.0/open-camera-mouse-0.4.0-macos-arm64.dmg`,
    ]);
    expect(r!.downloads.windows.map((d) => d.name)).toEqual([
      "open-camera-mouse-0.4.0-windows-x64-setup.exe",
    ]);
    expect(r!.downloads.linux.map((d) => d.label)).toEqual(["AppImage", "Archive (.tar.gz)"]);
  });

  test("a platform without its asset gets no downloads, the others keep theirs", () => {
    const r = parseRelease(apiRelease("v0.4.0", V040.filter((n) => !n.includes("macos"))));
    expect(r!.downloads.macos).toEqual([]);
    expect(r!.downloads.windows).toHaveLength(1);
  });

  test("ignores assets of another version and the old Go-era names", () => {
    const r = parseRelease(
      apiRelease("v0.4.0", [
        "open-camera-mouse-0.3.9-macos-arm64.dmg",
        "open-camera-mouse_v0.4.0_macOS.zip",
        "open-camera-mouse-0.4.0-windows-x64-setup.exe",
      ]),
    );
    expect(r!.downloads.macos).toEqual([]);
    expect(r!.downloads.windows).toHaveLength(1);
  });

  test("rejects a download URL that isn't this repository's release download", () => {
    const data = apiRelease("v0.4.0", V040);
    data.assets[2]!.browser_download_url = "https://example.com/open-camera-mouse-0.4.0-macos-arm64.dmg";
    expect(parseRelease(data)!.downloads.macos).toEqual([]);
  });

  test("returns null when nothing can be offered", () => {
    expect(parseRelease(null)).toBeNull();
    expect(parseRelease({ message: "API rate limit exceeded" })).toBeNull();
    expect(parseRelease(apiRelease("v0.4.0", []))).toBeNull();
    expect(parseRelease(apiRelease("v0.4.0", V040, { draft: true }))).toBeNull();
    expect(parseRelease(apiRelease("v0.4.0", V040, { prerelease: true }))).toBeNull();
  });

  test("returns null for a tag that isn't v<major>.<minor>.<patch>", () => {
    expect(parseRelease(apiRelease("latest", V040))).toBeNull();
    expect(parseRelease(apiRelease("0.4.0", V040))).toBeNull();
  });
});
