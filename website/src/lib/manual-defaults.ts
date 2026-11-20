// The manual's "Setting | Range | Default" tables must match the app. This module turns the
// app's settings source (crates/app-core/src/settings.rs: `ranges` and the `Default` impls)
// into the rows the manual should show, and reads the rows the manual does show. Pure; the
// test reads the files.

/** Range and default as the manual writes them; `range` undefined = not checked. */
export interface Row {
  range?: string;
  def: string;
}

/** `pub const NAME: RangeInclusive<T> = a..=b;` → NAME → [a, b]. */
export function parseRanges(src: string): Record<string, [number, number]> {
  const out: Record<string, [number, number]> = {};
  const re = /pub const (\w+): RangeInclusive<\w+> = ([\d.]+)\.\.=([\d.]+);/g;
  for (const m of src.matchAll(re)) out[m[1]] = [Number(m[2]), Number(m[3])];
  return out;
}

/** The `field: value,` lines of `impl Default for <name>`, as written in the source. */
export function parseDefaults(src: string, name: string): Record<string, string> {
  const start = src.indexOf(`impl Default for ${name} {`);
  if (start < 0) throw new Error(`no impl Default for ${name}`);
  const end = src.indexOf("\n}\n", start);
  const body = src.slice(start, end < 0 ? undefined : end);
  const out: Record<string, string> = {};
  for (const m of body.matchAll(/^\s*(\w+): (.+),\s*$/gm)) out[m[1]] = m[2].trim();
  return out;
}

const num = (s: string | undefined, what: string): number => {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`no number for ${what}: ${s}`);
  return n;
};

const onOff = (s: string | undefined, what: string): string => {
  if (s === "true") return "On";
  if (s === "false") return "Off";
  throw new Error(`no bool for ${what}: ${s}`);
};

const pick = <T>(table: Record<string, T>, key: string | undefined, what: string): T => {
  if (key === undefined || !(key in table)) throw new Error(`unknown ${what}: ${key}`);
  return table[key];
};

/** Setting label (as in the app and the manual) → the row the manual must show. */
export function expectedRows(src: string): Record<string, Row> {
  const r = parseRanges(src);
  const s = parseDefaults(src, "Settings");
  const d = parseDefaults(src, "DwellSettings");
  const range = (name: string): [number, number] => {
    const v = r[name];
    if (!v) throw new Error(`no range ${name}`);
    return v;
  };
  const dash = ([a, b]: [number, number], f: (n: number) => string, unit = "") =>
    `${f(a)}–${f(b)}${unit}`;
  const plain = (n: number) => String(n);
  const fixed = (digits: number) => (n: number) => n.toFixed(digits);
  const pct = (n: number) => String(Math.round(n * 100));
  const onOffRange = "On / Off";

  // ScreenLimits has no hand-written Default: its derived one is 0 on every side.
  const derived = /#\[derive\([^)]*\bDefault\b[^)]*\)\]\s*#\[serde\(default\)\]\s*pub struct ScreenLimits/;
  if (s.limits !== "ScreenLimits::default()" || !derived.test(src)) {
    throw new Error("Screen limits default is no longer the derived 0");
  }

  return {
    "Horizontal speed": { range: dash(range("GAIN"), plain), def: plain(num(s.gain_x, "gain_x")) },
    "Vertical speed": { range: dash(range("GAIN"), plain), def: plain(num(s.gain_y, "gain_y")) },
    "Keep both speeds the same": { range: onOffRange, def: onOff(s.link_gain, "link_gain") },
    Smoothing: { range: dash(range("SMOOTHING"), plain), def: fixed(2)(num(s.smoothing, "smoothing")) },
    "Dwell to click": { range: onOffRange, def: onOff(d.enabled, "dwell.enabled") },
    "Dwell time": {
      range: dash(range("DWELL_S"), fixed(1), " s"),
      def: `${fixed(1)(num(d.dwell_s, "dwell_s"))} s`,
    },
    "Pause after a click": {
      range: dash(range("REFRACTORY_S"), fixed(1), " s"),
      def: `${fixed(1)(num(d.refractory_s, "refractory_s"))} s`,
    },
    "Hold-still area": {
      range: dash(range("DWELL_RADIUS_FRAC"), pct, " %"),
      def: `${pct(num(d.radius_frac, "radius_frac"))} %`,
    },
    Camera: { def: pick({ None: "System default" }, s.camera, "camera default") },
    "Mirror preview": { range: onOffRange, def: onOff(s.mirror, "mirror") },
    "Find the target again": { range: onOffRange, def: onOff(s.loss_recovery, "loss_recovery") },
    "Screen limits (left, right, top, bottom)": {
      range: dash(range("LIMIT_PX"), plain, " px"),
      def: "0 px",
    },
    "Main monitor": {
      def: pick(
        { "MainMonitor::Primary": "Primary monitor", "MainMonitor::All": "Centre of all monitors" },
        s.recenter_display,
        "main monitor default",
      ),
    },
    Appearance: {
      range: "Dark / Light",
      def: pick({ "Theme::Carbon": "Dark", "Theme::Paper": "Light" }, s.theme, "theme default"),
    },
    "Larger controls": { range: onOffRange, def: onOff(s.larger_controls, "larger_controls") },
    "Show diagnostics": { range: onOffRange, def: onOff(s.show_diagnostics, "show_diagnostics") },
  };
}

/** Rows of every Markdown table whose header starts with "Setting": first cell → the rest. */
export function settingRows(md: string): Array<{ label: string; cells: string[] }> {
  const rows: Array<{ label: string; cells: string[] }> = [];
  let inTable = false;
  for (const line of md.split("\n")) {
    const t = line.trim();
    if (!t.startsWith("|")) {
      inTable = false;
      continue;
    }
    const cells = t.slice(1, t.endsWith("|") ? -1 : undefined).split("|").map((c) => c.trim());
    if (cells[0] === "Setting") inTable = true;
    else if (inTable && !/^-+$/.test(cells[0] ?? "")) rows.push({ label: cells[0], cells: cells.slice(1) });
  }
  return rows;
}
