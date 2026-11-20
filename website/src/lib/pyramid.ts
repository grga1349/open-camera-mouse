/**
 * The hero figure (`HeroPyramid.astro`): a stepped pyramid seen from above, a nod to the
 * image pyramid the tracker works on. Its top leans toward the pointer. Pure geometry in a
 * 400×400 viewBox, shared by the server render (top centred) and the client script.
 */

export const VIEW = 400;
const CENTER = VIEW / 2;
/** Half the base square's side. */
const BASE = 180;
/** How far the top may lean from the centre, in viewBox units. */
export const MAX_LEAN = 70;
/** Levels from the base (0) to the top (last), as a fraction of the way to the apex. */
const LEVELS = [0, 0.28, 0.5, 0.66, 0.77, 0.84];

export interface PyramidPaths {
  /** Outline of every level and the four edges, one `d` attribute. */
  lines: string;
  /** The filled top square. */
  top: string;
}

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

/**
 * The lean (in viewBox units) for a pointer at `nx`, `ny`: its offset from the figure's
 * centre divided by half the viewport, so -1…1 across the screen. Clamped to `MAX_LEAN`.
 */
export function leanFor(nx: number, ny: number): { dx: number; dy: number } {
  return { dx: clamp(nx, -1, 1) * MAX_LEAN, dy: clamp(ny, -1, 1) * MAX_LEAN };
}

const n = (v: number) => Math.round(v * 10) / 10;

/** Paths for a top leaning by `dx`, `dy` (clamped to `MAX_LEAN`). */
export function pyramidPaths(dx = 0, dy = 0): PyramidPaths {
  dx = clamp(dx, -MAX_LEAN, MAX_LEAN);
  dy = clamp(dy, -MAX_LEAN, MAX_LEAN);
  const squares = LEVELS.map((t) => {
    const cx = CENTER + dx * t;
    const cy = CENTER + dy * t;
    const h = BASE * (1 - t);
    return [
      [cx - h, cy - h],
      [cx + h, cy - h],
      [cx + h, cy + h],
      [cx - h, cy + h],
    ];
  });
  const square = (s: number[][]) => `M${s.map(([x, y]) => `${n(x)} ${n(y)}`).join("L")}Z`;
  const base = squares[0];
  const top = squares[squares.length - 1];
  const edges = base.map(([x, y], i) => `M${n(x)} ${n(y)}L${n(top[i][0])} ${n(top[i][1])}`).join("");
  return { lines: squares.slice(0, -1).map(square).join("") + edges, top: square(top) };
}
