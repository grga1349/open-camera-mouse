import { describe, expect, test } from "bun:test";
import { detectPlatform } from "./detect-platform";

const UA = {
  macSafari:
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15",
  macFirefox: "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.6; rv:132.0) Gecko/20100101 Firefox/132.0",
  winChrome:
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
  linuxFirefox: "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0",
  linuxArm: "Mozilla/5.0 (X11; Linux aarch64; rv:132.0) Gecko/20100101 Firefox/132.0",
  android:
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36",
  iphone:
    "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
  chromeOs:
    "Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
};

describe("detectPlatform", () => {
  test("desktop systems from the user agent string", () => {
    expect(detectPlatform({ userAgent: UA.macSafari })).toEqual({ kind: "supported", id: "macos" });
    expect(detectPlatform({ userAgent: UA.macFirefox })).toEqual({ kind: "supported", id: "macos" });
    expect(detectPlatform({ userAgent: UA.winChrome })).toEqual({ kind: "supported", id: "windows" });
    expect(detectPlatform({ userAgent: UA.linuxFirefox })).toEqual({ kind: "supported", id: "linux" });
  });

  test("never guesses the CPU: Intel-looking Macs and ARM Linux still map to the OS", () => {
    // Every Mac browser says "Intel Mac OS X", Apple Silicon included; the card notes say
    // what's supported instead.
    expect(detectPlatform({ userAgent: UA.macSafari }).kind).toBe("supported");
    expect(detectPlatform({ userAgent: UA.linuxArm })).toEqual({ kind: "supported", id: "linux" });
  });

  test("phones, tablets and ChromeOS are unsupported, Linux-based or not", () => {
    for (const ua of [UA.android, UA.iphone, UA.chromeOs]) {
      expect(detectPlatform({ userAgent: ua })).toEqual({ kind: "unsupported" });
    }
  });

  test("an iPad asking for the desktop site (Mac user agent with touch) is unsupported", () => {
    expect(detectPlatform({ userAgent: UA.macSafari, maxTouchPoints: 5 })).toEqual({
      kind: "unsupported",
    });
  });

  test("User-Agent Client Hints platform wins over the string", () => {
    expect(detectPlatform({ userAgent: UA.winChrome, uaPlatform: "Linux" })).toEqual({
      kind: "supported",
      id: "linux",
    });
    expect(detectPlatform({ userAgent: UA.linuxFirefox, uaPlatform: "Android" })).toEqual({
      kind: "unsupported",
    });
    expect(detectPlatform({ userAgent: UA.chromeOs, uaPlatform: "Chrome OS" })).toEqual({
      kind: "unsupported",
    });
  });

  test("anything else is unknown", () => {
    expect(detectPlatform({ userAgent: "" })).toEqual({ kind: "unknown" });
    expect(detectPlatform({ userAgent: "curl/8.7.1" })).toEqual({ kind: "unknown" });
    expect(detectPlatform({ userAgent: "", uaPlatform: "Fuchsia" })).toEqual({ kind: "unknown" });
  });
});
