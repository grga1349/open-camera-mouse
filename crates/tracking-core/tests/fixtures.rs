//! Fixture-level correctness tests (fast subset; the full comparison is `tests/baseline.rs`).
//!
//! These assert what must hold for any acceptable baseline, including *expected failures*
//! (a flat region must not be reported as tracked). Known gaps that Phase 4 is meant to fix
//! are documented in docs/research/TRACKING_BASELINE.md, not hidden here.

mod support;

use support::{e4_candidate, metrics, run, scene};
use tracking_core::lk::LkTracker;
use tracking_core::ncc::NccTracker;
use tracking_core::{HoldReason, TrackStatus};

/// The generator's ground truth must agree with its own pixels, independently of any
/// tracker: with no noise and integer motion, the window around truth(t) is identical to the
/// window around truth(0) (the target moved rigidly).
#[test]
fn generator_truth_matches_rendered_pixels() {
    let mut s = scene("translate_int");
    s.noise_sigma = 0.0;
    s.frames = 8;
    let seq = s.render(640, 480);
    let patch = |t: usize| {
        let (cx, cy) = (seq.truth[t].x.round() as i64, seq.truth[t].y.round() as i64);
        let f = &seq.frames[t];
        let mut v = Vec::new();
        for dy in -8..=8 {
            for dx in -8..=8 {
                v.push(f.get((cx + dx) as u32, (cy + dy) as u32).unwrap());
            }
        }
        v
    };
    let first = patch(0);
    for t in 1..8 {
        assert_eq!(patch(t), first, "frame {t}");
    }
}

#[test]
fn generator_is_deterministic() {
    let s = scene("head_sway");
    let (a, b) = (s.render(320, 240), s.render(320, 240));
    assert_eq!(a.frames, b.frames);
    assert_eq!(a.truth, b.truth);
}

#[test]
fn downsampled_truth_is_mapped_to_tracker_pixels() {
    let seq = scene("translate_int").render(1280, 720);
    assert_eq!(seq.size, (640, 360));
    // Selection at ref (332, 172) → device (664, 344) → (664.5 / 2 − 0.5, …) = (331.75, 171.75).
    assert_eq!((seq.truth[0].x, seq.truth[0].y), (331.75, 171.75));
}

#[test]
fn lk_tracks_smooth_motion_subpixel_at_all_resolutions() {
    for (w, h) in support::RESOLUTIONS {
        for name in ["translate_subpx", "head_sway"] {
            let mut s = scene(name);
            s.frames = s.frames.min(45);
            let seq = s.render(w, h);
            let m = metrics(&seq, &run(&mut LkTracker::default(), &seq));
            assert_eq!(m.tracked_ok, 1.0, "{name} {w}×{h}: {m:?}");
            assert!(m.epe_p95 < 0.25, "{name} {w}×{h}: {m:?}");
        }
    }
}

#[test]
fn ncc_tracks_integer_motion_to_the_pixel() {
    let seq = scene("translate_int").render(640, 480);
    let m = metrics(&seq, &run(&mut NccTracker::default(), &seq));
    assert_eq!(m.tracked_ok, 1.0, "{m:?}");
    assert!(m.epe_max <= 0.75, "{m:?}");
}

#[test]
fn textureless_target_is_never_reported_as_tracking() {
    let seq = scene("textureless").render(320, 240);
    for (alg, steps) in [
        ("LK18", run(&mut LkTracker::default(), &seq)),
        ("NCC", run(&mut NccTracker::default(), &seq)),
    ] {
        let tracking = steps[1..]
            .iter()
            .filter(|s| s.status == TrackStatus::Tracking)
            .count();
        assert_eq!(tracking, 0, "{alg} reported Tracking on a flat region");
    }
}

#[test]
fn lk_outlier_guard_holds_on_a_40px_jump_at_640() {
    let seq = scene("fast_40px").render(640, 480);
    let steps = run(&mut LkTracker::default(), &seq);
    assert!(
        steps
            .iter()
            .any(|s| s.status == TrackStatus::Held(HoldReason::MotionOutlier)),
        "40 px/frame > 35 px guard should produce Held(MotionOutlier)"
    );
}

/// Phase 4 E4 (ADR-0010): re-detection recovers the losses findings 2–4 describe, and never
/// reports `Tracking` on a lookalike, the occluder or a grating. 320×240 keeps it fast in
/// debug builds; `ablation_tables` covers all resolutions.
#[test]
fn e4_recovers_losses_without_silent_errors() {
    for (name, min_ok) in [
        ("fast_40px", 0.95),
        ("edge_left", 0.9),
        ("edge_top", 0.9),
        ("occlusion_full", 0.95),
        ("distractor_similar", 0.95),
        ("distractor_late_twin", 0.95),
        // Disarmed by the selection-time self-test: stays lost, as under the 2018 policy.
        ("distractor_twin", 0.55),
        ("repeated_texture", 0.0),
    ] {
        let seq = scene(name).render(320, 240);
        let m = metrics(&seq, &run(&mut LkTracker::new(e4_candidate()), &seq));
        assert_eq!(m.wrong_lock, 0, "{name}: {m:?}");
        assert_eq!(m.false_tracking, 0, "{name}: {m:?}");
        assert!(m.tracked_ok >= min_ok, "{name}: {m:?}");
    }
}

#[test]
fn trackers_are_deterministic_on_fixtures() {
    let seq = scene("sensor_noise").render(320, 240);
    assert_eq!(
        run(&mut LkTracker::default(), &seq),
        run(&mut LkTracker::default(), &seq)
    );
    assert_eq!(
        run(&mut NccTracker::default(), &seq),
        run(&mut NccTracker::default(), &seq)
    );
    let seq = scene("edge_left").render(320, 240);
    assert_eq!(
        run(&mut LkTracker::new(e4_candidate()), &seq),
        run(&mut LkTracker::new(e4_candidate()), &seq)
    );
}
