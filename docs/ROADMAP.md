# Roadmap

## Done: the Rust rewrite (v0.4.0)

Phases 0–7 of the rewrite plan are complete (2026-11-11 → 2026-11-19): research of Camera
Mouse 2018 and the Go app, the workspace (ADR-0007), a pure-Rust tracker without OpenCV
(ADR-0009…0011, -0018), the approved design, the desktop app (ADR-0012, -0016), two device
test-drive and tuning rounds plus calibrated pointer behaviour (ADR-0013…0015, -0017, -0019,
-0020), and packaging and the release workflow (ADR-0021). The cleanup that ended it is
ADR-0022. Planning files, research notes and session logs were removed then; they are in git
history (see ADR-0022 for the commit).

Phase 7 closed with these exceptions (user decisions 2026-11-19): the Windows and Linux
on-device checks and the external-webcam check were **deferred** (no such devices); the
separate release-docs slice was dropped (its content is in the README).

## Next: hardening and product readiness (Phase 8)

Goal: a written readiness decision based on evidence: stability, CPU/RAM and latency
budgets, failure recovery, accessibility, real-user workflows, support docs.

Open items, roughly by priority:

1. **First release run (v0.4.0).** The Windows and Linux packaging steps have never run;
   watch the tag's release workflow and fix what fails.
2. **Windows and Linux on a device** (when one is available): pointer (`SendInput`, UIPI P8,
   mixed DPI P9), primary monitor, camera (P3), hotkeys, installer / AppImage; Manual QA in
   `docs/TESTING.md`.
3. **External webcams** (P1): at least one USB webcam per OS; panics must stay caught.
4. **Reduced motion:** the Searching animation ignores the OS setting (Manual QA row 21).
5. **Re-measure on recordings:** re-detection arming, the R-P4 jump bound (0.05 × shorter
   desktop side, provisional), dwell radius; all tracker thresholds so far are fitted on
   synthetic data. The rest tests (`the_cursor_does_not_move_at_rest_*`) are not calibrated
   against recordings.
6. **ADR-0018 limit:** a change slower than the gate allows in 10 frames is followed down to
   the selection floor (0.3), so very slow hand creep over the target can be followed.
7. **Accessibility:** screen readers per OS (P15); font is egui's default with faux bold
   (Inter is a candidate).
8. **Signing:** Developer ID + notarization (macOS) and a Windows certificate; the workflow
   hooks are ready behind secrets. Until then the ad-hoc identifier-only signature means any
   ad-hoc binary claiming the bundle id matches the permission grants. The `.icns` lacks
   Retina `@2x` slots.
9. **Dependencies:** `nokhwa`'s macOS bindings pull `block` 0.1.6 (future-incompat warning);
   MJPEG-only cameras are unsupported (ADR-0012).
10. **Wayland:** pointer and hotkeys via the RemoteDesktop / GlobalShortcuts portals (P10–P12),
    if users need it.
11. **Multi-monitor:** with uneven monitors the cursor sticks to edges where the desktop's
    bounding box has dead zones (snap). Fine for v1.

## Website

Scaffold on `feature/website` (ADR-0023, `docs/WEBSITE.md`). Open:

1. **Landing page design and content** (`docs/WEBSITE_DESIGN.md`, "Initial page structure").
2. **Enable GitHub Pages** (source GitHub Actions, variable `WEBSITE_DEPLOY=true`); the
   first deployment has not run.
3. **Theme switch** (Carbon/Paper follow the OS for now; a switch needs JS and storage).

New work arrives as prompts (see `AGENTS.md`); add an item here only when it is deferred.
