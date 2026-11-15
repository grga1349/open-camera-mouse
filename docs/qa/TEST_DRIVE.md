# Test drive — issue log (Phase 6b, task 0009, ADR-0013)

The user uses the app on the dev Mac for real work. Every issue they report gets one row here,
in the order reported. A row is closed when it has a fix commit (with its regression test, or a
reason it can't be tested headless) or a deferral to a named phase with a reason.

Run it: `export PATH="$HOME/.cargo/bin:$PATH"` then `cargo run -p open-camera-mouse` (dev build; user decision 2026-11-15).
For a tracking problem add `OCM_RECORD=<dir outside the repo>` (≈9 MB/s, contains your face).

Severity: **blocker** (can't use the app for real work), **major** (works around it, but it
hurts), **minor** (cosmetic or rare).

| # | Date | Build | What happened (steps) | Severity | Triage | Resolution |
|---|---|---|---|---|---|---|
| 1 | 2026-11-15 | `fc884e5` dev | The camera picture has square corners; it should have lightly rounded ones. | minor | UI polish, user-requested. The frame was already rounded (8), but the picture and the Dimmed/Frozen scrim were painted square on top of it. | Fixed: picture + scrim rounded to `PREVIEW_RADIUS` 8 in `preview.rs`. Checked visually, not by a headless test (egui doesn't expose rounding to kittest). |
| 2 | 2026-11-15 | `fc884e5` dev | The tracking point is lost too easily. | major | Tracking. Replay of an `OCM_RECORD` session: every loss is `Lost(AppearanceChanged)`, the ZNCC vs the selection sliding 0.97 → <0.7 over ~1 s as the head turns (LK drift on smooth skin, caught by the E2 gate). Contrast won't help (ZNCC + gain/bias are already contrast-invariant). First tuning pass (`replay_tune`): hw 15 + gate 0.6 = 9 losses vs 15 for recommended, no added drift; recording is a poor benchmark (face often out of view). | **Open, part 2.** Needs a scripted benchmark recording + research (see `docs/research/TEST_DRIVE_RESEARCH_PROMPT.md`). User: still "loss is easy, needs to improve a lot". |
| 3 | 2026-11-15 | `fc884e5` dev | With "Find the target again" on, it rarely finds the point again. | major | Root cause: re-detection never armed on a face — best 0.99 vs best distinct other peak 0.90–0.94, so the 0.1 margin self-test always failed. | Fixed (commit below): recommended margin 0.03, 3 consecutive confirming finds, 41×41 context check ≥ 0.7 (ADR-0011 amendment 2026-11-15). Replay: 15 automatic recoveries, all but one on the nose; the earlier wrong locks (hand, shirt) refused. Tests: `lk.rs` `recommended_redetect_confirms_over_three_frames`, `redetect_context_check_refuses_a_lookalike_in_other_surroundings`; `controller.rs` `recovery_*`. User: "ok for now, could be better" → part 2. |
| 4 | 2026-11-15 | `fc884e5` dev | With several monitors, want to choose which monitor the cursor starts on. | — | **New behaviour** (user-approved). User chose: recenter/pause put the cursor at the chosen display's centre; every display stays reachable. ADR-0014. | Done: Options → Pointer → "Main monitor" (shown only with 2+ monitors, any number), default "Centre of all monitors" (2018). Tests: `controller.rs` `recenter_on_*` + 2 more, `app_tests.rs` `main_monitor_*`. On-device check by the user pending. |
| 5 | 2026-11-15 | `fc884e5` dev | The cursor jitters and drifts while the head is held still. | major | Pointer noise = tracker noise × mapping scale. **Likely amplifier: the mapping scales by the whole desktop** (user: 2560 + 2560 wide → 6 × 5120 / 640 ≈ 48 screen px per camera px, so 0.2 px of tracker noise ≈ 10 px). | Partial: "Steady zone" option (rubber-band dead zone, ADR-0015, off by default) shipped with tests; **user: "looks bad, cursor still moves 10–20 px"**. Open, part 2: gain per display, better smoothing (1€ filter), sub-pixel noise, larger zone. |

## Notes per issue

(Root cause, evidence and test names, one short section per row once it is worked on.)
