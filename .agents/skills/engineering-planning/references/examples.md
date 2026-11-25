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
| D1 | May a dwell click on the preview re-select the target? | 1-F2 | |

## Slice 1 — Clicking under camera control (P0)
- [x] **1-F1 A gain change keeps the cursor where it is** …
- [?] **1-F2 No re-select from a camera-driven click** — D1
- [i] **1-F3 Dwell never fires during a physical-button drag** — owed: macOS device, physical button

## Slice 2 — Windows packaging (P2)
- [ ] **2.1 No console window in release builds** …

## Leftovers
(entries as below)
```

## Non-trivial work item

```markdown
- [~] **1-F1 A gain change keeps the cursor where it is** (S1, likely S4)
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
- [>] **2-F6 Windows camera moves are invisible to raw-input apps** (from slice 2, P2)
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
| 1-F3 | `[i]` owed macOS device | complete now | 5-min user check; P0 |
| 2.1 | `[ ]` | reactivate as blocker | console window on every Windows launch |
| 2-F6 | Leftovers | transfer → ROADMAP App 1 | no device; not a regression |
| 3.2 | `[?] device: Linux` | accept as known limitation | README "untested on Linux" |
| 3.4 | `[ ]` | drop as obsolete | superseded by 1-F1 |
Needs your decision: 2.1 (blocker?), 3.2 (limitation wording).
Before deletion: D1 → ADR-0006 amendment; 1-F1 evidence lives in its test.
```

## Compact handoff

```markdown
# v0.4.2 handoff (temporary)
Plan and statuses: [`V0.4.2_ROADMAP.md`](V0.4.2_ROADMAP.md). Rewrite at every checkpoint.

## Resume here
- Mode: standard orchestration. Active item: 1-F3.
- Next: write the kittest drag test (fails today), then the fix.
- Waiting on the user: D1 (blocks 1-F2); macOS device run for 1-F1.
- Working tree: clean at `abc1234`, pushed.

## Recent findings (not yet in an item)
- egui batches a redraw's events into one pass (relevant to 1-F5).

## Verification
- Last gates (session 4): fmt, clippy clean; tests app-core, pointer-core passed. CI green.
- Owed: device runs per item (`[i]` in the roadmap). Not run: Windows, Linux (CI only).

## Last session (4, 2026-12-04)
1-F1 fixed with the rig test (failed before); 1-F2 blocked on D1. Models: main Opus,
test engineer sonnet, review opus.
```
