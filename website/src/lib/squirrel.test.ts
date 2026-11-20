import { describe, expect, test } from "bun:test";
import { BIG_REACH, HEAD_TILT, SMALL_REACH, TAIL_SWAY, squirrelPose, squirrelTransform } from "./squirrel";

const shift = (t: string) => t.match(/^translate\((-?[\d.]+) (-?[\d.]+)\)$/)!.slice(1).map(Number);

describe("squirrelPose", () => {
  test("looks ahead with the pointer at the centre", () => {
    expect(squirrelPose(0, 0)).toEqual({
      head: { tilt: 0, x: 0, y: 0 },
      face: { x: 0, y: 0 },
      look: { x: 0, y: 0 },
      tail: 0,
    });
  });

  test("tilts the head toward the pointer's side", () => {
    expect(squirrelPose(1, 0).head.tilt).toBe(HEAD_TILT);
    expect(squirrelPose(-1, 0).head.tilt).toBe(-HEAD_TILT);
  });

  test("the tail sways away from the pointer and far pointers are clamped", () => {
    expect(squirrelPose(1, 0).tail).toBe(-TAIL_SWAY);
    expect(squirrelPose(9, -9)).toEqual(squirrelPose(1, -1));
  });
});

describe("squirrelTransform", () => {
  test("each pupil stays inside its own eye", () => {
    for (const [x, y] of [[1, 0], [1, 1], [-1, 1], [5, -5], [0.3, 0.2]]) {
      const t = squirrelTransform(squirrelPose(x, y));
      expect(Math.hypot(...shift(t.pupilBig))).toBeLessThanOrEqual(BIG_REACH + 0.02);
      expect(Math.hypot(...shift(t.pupilSmall))).toBeLessThanOrEqual(SMALL_REACH + 0.02);
    }
  });

  test("both pupils look the same way, the big one further", () => {
    const t = squirrelTransform(squirrelPose(1, 0));
    expect(shift(t.pupilBig)).toEqual([BIG_REACH, 0]);
    expect(shift(t.pupilSmall)).toEqual([SMALL_REACH, 0]);
  });

  test("SVG transforms with no -0", () => {
    expect(squirrelTransform(squirrelPose(0, 0))).toEqual({
      head: "translate(0 0) rotate(0 200 232)",
      face: "translate(0 0)",
      pupilBig: "translate(0 0)",
      pupilSmall: "translate(0 0)",
      tail: "rotate(0 240 340)",
    });
  });
});
