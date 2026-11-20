import { describe, expect, test } from "bun:test";
import { MAX_LEAN, VIEW, leanFor, pyramidPaths } from "./pyramid";

const points = (d: string) => [...d.matchAll(/(-?[\d.]+) (-?[\d.]+)/g)].map((m) => [Number(m[1]), Number(m[2])]);

describe("pyramidPaths", () => {
  test("centred top is symmetric", () => {
    const { top } = pyramidPaths();
    const xs = points(top).map(([x]) => x);
    expect(Math.min(...xs) + Math.max(...xs)).toBeCloseTo(VIEW, 5);
  });

  test("the top leans toward the pointer and the base stays put", () => {
    const still = pyramidPaths();
    const leaning = pyramidPaths(40, -20);
    expect(points(leaning.lines).slice(0, 4)).toEqual(points(still.lines).slice(0, 4));
    const cx = (d: string) => points(d).reduce((s, [x]) => s + x, 0) / 4;
    const cy = (d: string) => points(d).reduce((s, [, y]) => s + y, 0) / 4;
    expect(cx(leaning.top)).toBeGreaterThan(cx(still.top));
    expect(cy(leaning.top)).toBeLessThan(cy(still.top));
  });

  test("never leaves the viewBox, however far the lean", () => {
    for (const [dx, dy] of [[1e6, 1e6], [-1e6, 1e6], [0, -1e6]]) {
      const { lines, top } = pyramidPaths(dx, dy);
      for (const [x, y] of points(lines + top)) {
        expect(x).toBeGreaterThanOrEqual(0);
        expect(x).toBeLessThanOrEqual(VIEW);
        expect(y).toBeGreaterThanOrEqual(0);
        expect(y).toBeLessThanOrEqual(VIEW);
      }
    }
  });
});

describe("leanFor", () => {
  test("scales and clamps the pointer offset", () => {
    expect(leanFor(0, 0)).toEqual({ dx: 0, dy: 0 });
    expect(leanFor(0.5, -0.5)).toEqual({ dx: MAX_LEAN / 2, dy: -MAX_LEAN / 2 });
    expect(leanFor(3, -3)).toEqual({ dx: MAX_LEAN, dy: -MAX_LEAN });
  });
});
