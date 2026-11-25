# Planning examples

Illustrations of the formats in `.agents/PLANNING.md`, drawn from v0.4.1-style work. Copy the
shape, not the content; leave out any field that adds nothing.

## Active release roadmap (skeleton)

```markdown
# v0.4.2 roadmap (temporary)

Plan for the v0.4.2 release on `fix/v0.4.2`. Resume state: [`V0.4.2_HANDOFF.md`](V0.4.2_HANDOFF.md).
Both files are deleted before the merge, after reconciliation (`.agents/PLANNING.md`).
Source: the user's brief of 2026-12-01 (symptoms and leads, not diagnoses).

Legend: `[ ]` open · `[~]` in progress · `[?]` blocked · `[i]` implemented, verification owed ·
`[x]` done · `[>]` deferred · `[-]` dropped

## Rules for this release
- No merge, tag or release without the user's yes. No Windows device here: CI only.

## Decisions
| # | Question | Blocks | Answer |
|---|---|---|---|
| Q1 | May a dwell click on the preview re-select the target? (a) never (b) not under camera control. Rec.: (b) | A3 | |

## Phase A — Clicking under camera control
- [i] **A1 A gain change keeps the cursor where it is** (P0) — owed: A3
- [ ] **A2 Dwell never fires during a physical-button drag** (P0) …
- [ ] **A3 macOS user run** (P0): physical press-drag on both speed sliders (A1); hold a
      button still mid-drag (A2). Accept: … Verify: user, macOS, physical button and dwell
- [?] **A4 No re-select from a camera-driven click** (P0) — Q1

## Phase B — Windows packaging
- [ ] **B1 No console window in release builds** (P2) …

## Done
- [x] **A dwell click sets a slider** — kittest `a_same_pass_click_sets_the_slider`

## Leftovers
(entries as below)
```

## Non-trivial work item

```markdown
- [~] **A1 A gain change keeps the cursor where it is** (P0; S1, likely S4)
      Needs: —. Evidence: `map_raw_scaled` scales offset around the desktop centre; traced,
      not yet a failing test.
      Accept: changing gain_x/gain_y under camera control moves the cursor by < 1 px; dragging
      a speed slider with the head converges (no 1 ↔ 60 swing).
      Verify: controller-rig test changing the gain while Tracking (fails before);
      kittest speed-slider drag; macOS device run with a physical button (user).
      Out of scope: the smoothing slider (no mapping effect).
      Done: rig test fails before, passes after. Remaining: the fix, kittest, device run.
```

## Leftovers entry

```markdown
- [>] **Windows camera moves are invisible to raw-input apps** (was B3, from Phase B, P2)
      Reason: needs a Windows device and a CAD app; none available (user, 2026-12-03).
      Done: cause traced (`SetCursorPos` makes no raw-input motion); unit test of the
      `SendInput` event builder. Remaining: switch the adapter, device check.
      Needs: Windows device. Blocks release: no.
      Proposed: transfer to `docs/ROADMAP.md` App 1 (Windows on a device).
```

## Release reconciliation summary

```markdown
| Item | State | Proposal | Why |
|---|---|---|---|
| A2 | `[i]` owed macOS device | complete now | 5-min user check; P0 |
| B1 | `[ ]` | reactivate as blocker | console window on every Windows launch |
| raw moves | Leftovers | transfer → ROADMAP App 1 | no device; not a regression |
| C2 | `[?] device: Linux` | accept as known limitation | README "untested on Linux" |
| C4 | `[ ]` | drop as obsolete | superseded by A1 |
Needs your decision: B1 (blocker?), C2 (limitation wording).
Before deletion: Q1 → ADR-0006 amendment; A1 evidence lives in its test.
```

## Compact handoff

```markdown
# v0.4.2 handoff (temporary)
Plan and statuses: [`V0.4.2_ROADMAP.md`](V0.4.2_ROADMAP.md). Rewrite at every checkpoint.

## Resume here
- Mode: standard orchestration. Active item: A2.
- Next: write the kittest drag test (fails today), then the fix.
- Waiting on the user: Q1 (blocks A4); macOS device run for A1.
- Working tree: clean at `abc1234`, pushed.

## Recent findings (not yet in an item)
- egui batches a redraw's events into one pass (relevant to A5).

## Verification
- Last gates (session 4): fmt, clippy clean; tests app-core, pointer-core passed. CI green.
- Owed: device runs per item (`[i]` in the roadmap). Not run: Windows, Linux (CI only).

## Last session (4, 2026-12-04)
A1 fixed with the rig test (failed before); A4 blocked on Q1. Models: main Opus,
test engineer sonnet, review opus.
```
