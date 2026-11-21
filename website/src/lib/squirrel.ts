/**
 * The hero figure (`HeroSquirrel.astro`): a cute, faceted squirrel seen from the front, that watches the pointer, a nod to what the app does (it follows your head). Pure
 * pose maths in a 400×400 viewBox, shared by the build-time render (looking ahead) and the
 * client script.
 *
 * The pointer's offset from the squirrel, as -1…1 across the screen, turns into: a curious
 * head tilt toward the pointer's side with a small head shift, the face (eyes, nose, mouth)
 * sliding a little further inside the head as if it turned, both pupils looking toward the
 * pointer (each within its own eye; the eyes differ in size), and the tail swaying the
 * other way.
 */

export const VIEW = 400;
/** Head tilt at full offset, degrees; and the point it tilts around (the neck). */
export const HEAD_TILT = 12;
export const NECK = { x: 200, y: 236 };
/** Head shift toward the pointer at full offset, viewBox units. */
export const HEAD_SHIFT = 6;
/** Extra shift of the face inside the head at full offset, viewBox units. */
export const FACE_SHIFT = 8;
/** How far each pupil may move from its eye's centre, viewBox units. */
export const BIG_REACH = 8;
export const SMALL_REACH = 5;
/** Tail sway at full offset, degrees; and the tail's base. */
export const TAIL_SWAY = 8;
export const TAIL_BASE = { x: 240, y: 340 };

export interface Pose {
  head: { tilt: number; x: number; y: number };
  face: { x: number; y: number };
  /** Where the pupils look, a direction of length ≤ 1. */
  look: { x: number; y: number };
  tail: number;
}

const clamp = (v: number) => Math.min(1, Math.max(-1, v));
const r = (v: number) => Math.round(v * 100) / 100 + 0; // + 0: no "-0" in attributes

/** The pose for a pointer at `nx`, `ny` (offset from the squirrel over half the viewport). */
export function squirrelPose(nx: number, ny: number): Pose {
  const x = clamp(nx);
  const y = clamp(ny);
  const len = Math.hypot(x, y);
  const k = len > 1 ? 1 / len : 1;
  return {
    head: { tilt: r(x * HEAD_TILT), x: r(x * HEAD_SHIFT), y: r(y * HEAD_SHIFT) },
    face: { x: r(x * FACE_SHIFT), y: r(y * FACE_SHIFT * 0.7) },
    look: { x: x * k, y: y * k },
    tail: r(-x * TAIL_SWAY),
  };
}

export type Part = "head" | "face" | "pupilBig" | "pupilSmall" | "tail";

/** SVG `transform` values for a pose. */
export function squirrelTransform(p: Pose): Record<Part, string> {
  const look = (reach: number) => `translate(${r(p.look.x * reach)} ${r(p.look.y * reach)})`;
  return {
    head: `translate(${p.head.x} ${p.head.y}) rotate(${p.head.tilt} ${NECK.x} ${NECK.y})`,
    face: `translate(${p.face.x} ${p.face.y})`,
    pupilBig: look(BIG_REACH),
    pupilSmall: look(SMALL_REACH),
    tail: `rotate(${p.tail} ${TAIL_BASE.x} ${TAIL_BASE.y})`,
  };
}
