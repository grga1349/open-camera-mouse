import { describe, expect, test } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { expectedRows, parseDefaults, parseRanges, settingRows } from "./manual-defaults";

const repo = join(import.meta.dir, "../../..");
const rust = readFileSync(join(repo, "crates/app-core/src/settings.rs"), "utf8");
const manualDir = join(import.meta.dir, "../content/manual");
const pages = readdirSync(manualDir)
  .filter((f) => f.endsWith(".md"))
  .map((f) => ({ file: f, md: readFileSync(join(manualDir, f), "utf8") }));

describe("parsing the app's settings source", () => {
  test("reads ranges and default literals", () => {
    const src = `
pub const GAIN: RangeInclusive<f32> = 1.0..=60.0;
impl Default for Settings {
    fn default() -> Self {
        Self {
            // a comment
            gain_x: 20.0,
            theme: Theme::Carbon,
        }
    }
}
`;
    expect(parseRanges(src)).toEqual({ GAIN: [1, 60] });
    expect(parseDefaults(src, "Settings")).toEqual({ gain_x: "20.0", theme: "Theme::Carbon" });
  });

  test("a changed default changes the expected row", () => {
    const changed = rust.replace("gain_x: 20.0,", "gain_x: 25.0,");
    expect(changed).not.toBe(rust);
    expect(expectedRows(changed)["Horizontal speed"].def).toBe("25");
  });
});

describe("settingRows", () => {
  test("reads only tables headed Setting", () => {
    const md = "| Key | macOS |\n|---|---|\n| Smoothing | x |\n\n| Setting | Range | Default |\n|---|---|---|\n| Smoothing | 0–0.95 | 0.20 |\n";
    expect(settingRows(md)).toEqual([{ label: "Smoothing", cells: ["0–0.95", "0.20"] }]);
  });
});

describe("the manual matches the app", () => {
  const expected = expectedRows(rust);

  test("every setting row shows the app's range and default", () => {
    let checked = 0;
    for (const { file, md } of pages) {
      for (const { label, cells } of settingRows(md)) {
        const want = expected[label];
        if (!want) throw new Error(`${file}: "${label}" isn't a known setting; add it to expectedRows`);
        const [range, def] = cells;
        if (want.range !== undefined) expect(`${file}: ${label}: ${range}`).toBe(`${file}: ${label}: ${want.range}`);
        expect(`${file}: ${label}: ${def}`).toBe(`${file}: ${label}: ${want.def}`);
        checked++;
      }
    }
    expect(checked).toBeGreaterThan(0);
  });

  test("the settings page lists every setting", () => {
    const settings = pages.find((p) => p.file === "settings.md");
    expect(settings).toBeDefined();
    const labels = settingRows(settings!.md).map((r) => r.label);
    expect(labels.sort()).toEqual(Object.keys(expected).sort());
  });
});
