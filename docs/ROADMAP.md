# Roadmap

Permanent backlog, roughly by priority: deferred work, known limitations, tech debt,
investigations, platform work, outstanding verification, ideas not yet approved. Entries
authorise nothing (`.agents/PLANNING.md`); remove an entry when it lands.
How the Rust rewrite got here (v0.4.0, 2026-11-20): ADRs 0001–0022 and git history.

## App

1. **Windows and Linux on a device:** pointer (`SendInput`, UIPI P8, mixed DPI P9), primary
   monitor, camera (P3), hotkeys, installer / AppImage; Manual QA in `docs/TESTING.md`.
   Windows is being tested by the user for v0.4.1; Linux has had no device run yet.
   Still to organise after v0.4.1: a structured `OCM_PERF` session on Windows (loss
   reasons, re-detection, edges at 1920×1080, `docs/TESTING.md` Diagnostics), like the
   macOS runs that drove the v0.4.1 tracking fix. Check the v0.4.1 borderline hold
   (ADR-0018 amendment 2026-11-24): fewer losses on hard moves, the `held look` count, and
   whether users see the point skew onto a nearby spot (the anchored gate's known limit).
2. **External webcams** (P1): at least one USB webcam per OS; panics must stay caught.
3. **Reduced motion:** the Searching animation ignores the OS setting (Manual QA row 21).
4. **Re-measure on recordings:** re-detection arming, the R-P4 jump bound (0.05 × shorter
   desktop side, provisional), dwell radius; all tracker thresholds so far are fitted on
   synthetic data. The rest tests (`the_cursor_does_not_move_at_rest_*`) are not calibrated
   against recordings.
5. **ADR-0018 limit:** a change slower than the gate allows in 10 frames is followed down to
   the selection floor (0.3), so very slow hand creep over the target can be followed.
6. **Accessibility:** screen readers per OS (P15); font is egui's default with faux bold
   (Inter is a candidate).
7. **Signing:** Developer ID + notarization (macOS) and a Windows certificate; the workflow
   hooks are ready behind secrets. Until then the ad-hoc identifier-only signature means any
   ad-hoc binary claiming the bundle id matches the permission grants. The `.icns` lacks
   Retina `@2x` slots.
8. **Dependencies:** `nokhwa`'s macOS bindings pull `block` 0.1.6 (future-incompat warning);
   MJPEG-only cameras are unsupported (ADR-0012).
9. **Wayland:** pointer and hotkeys via the RemoteDesktop / GlobalShortcuts portals (P10–P12),
   if users need it.
10. **Multi-monitor:** with uneven monitors the cursor sticks to edges where the desktop's
    bounding box has dead zones (snap). Fine for v1.

## Website

Live since 2026-11-21 (PR #2, ADR-0023); how it's built and its rules: `docs/WEBSITE.md`.
Open:

1. **Copy read-through** by the user over the whole site (wording isn't approved per change).
2. **Checks by hand** (done during use): keyboard walk-through, VoiceOver/NVDA, 200% zoom,
   the real reduced-motion setting, a real phone, Safari and Firefox; in particular the hero
   V at 320px, the two-line headline with Windows and Linux fonts, video playback and the
   play button, the squirrel with a real head pointer, OS detection on Windows and Linux.
3. **Release-triggered rebuild:** verify on the next published release that the site
   rebuilds and shows the new version (`website.yml` `rebuild` job).
4. Optional, only with the user's OK: section transitions and accents beyond the hero.
