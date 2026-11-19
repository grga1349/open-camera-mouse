//! Appearance-change scenes (task 0009 slice 2b): illumination gradient, non-rigid warp,
//! scale + roll, and a combined head-turn stand-in, with exact ground truth. Pins what the
//! recommended tracker (ADR-0011) does on them today, so tracker work in slice 2d has a
//! regression gate: a change may move a number in the good direction (then update the pin
//! and say so in the commit), never the other way. Slice 2d added two slow-occluder scenes,
//! the adversary of an adaptive appearance reference, and made the anchored update
//! (ADR-0018) part of the recommended tracker; the plain gate keeps one baseline test.
//!
//! Print the current numbers:
//! `cargo test -p tracking-core --test appearance appearance_table -- --ignored --nocapture`

mod support;

use support::{Metrics, appearance_corpus, metrics, run, scene};
use tracking_core::lk::{LkParams, LkTracker};

fn measure_with(params: LkParams, name: &str, (w, h): (u32, u32)) -> Metrics {
    let seq = scene(name).render(w, h);
    metrics(&seq, &run(&mut LkTracker::new(params), &seq))
}

fn measure(name: &str) -> Metrics {
    measure_with(LkParams::recommended(), name, (640, 480))
}

/// The recommended tracker without the slice 2d anchored update (ADR-0018).
fn plain_gate() -> LkParams {
    LkParams {
        anchored: None,
        ..LkParams::recommended()
    }
}

#[test]
fn warp_keeps_the_selected_point_fixed_and_deforms_its_surroundings() {
    let s = scene("nonrigid_warp");
    let at = |t: usize, dx: f64, dy: f64| {
        let (x, y) = s.truth_ref(t);
        s.clean_at(t, x + dx, y + dy)
    };
    // Frame 0 has no warp; frame 45 has the most (5 ref px).
    for t in [0, 20, 45, 70] {
        assert!(
            (at(t, 0.0, 0.0) - at(0, 0.0, 0.0)).abs() < 1e-9,
            "frame {t}"
        );
    }
    let moved = (-10..=10)
        .flat_map(|dy| (-10..=10).map(move |dx| (f64::from(dx), f64::from(dy))))
        .filter(|&(dx, dy)| (at(45, dx, dy) - at(0, dx, dy)).abs() > 1.0)
        .count();
    assert!(moved > 200, "only {moved} of 441 neighbours changed");
}

#[test]
fn illumination_gradient_scales_values_across_the_target() {
    let s = scene("illum_gradient");
    // Frame 22 ≈ quarter period: gx ≈ +0.004. Right of the centre gets brighter, left darker,
    // relative to frame 0 (no gradient) at the same target-relative place.
    let rel = |t: usize, dx: f64| {
        let (x, y) = s.truth_ref(t);
        s.clean_at(t, x + dx, y)
    };
    let (sel_x, sel_y) = s.select_offset;
    let ratio = |dx: f64| rel(22, dx) / rel(0, dx);
    // Expected factor 1 + gx·dx_c + gy·dy_c, offsets from the target centre.
    let gx = 0.004 * (22.0 * std::f64::consts::TAU / 90.0).sin();
    let gy = 0.002 * (22.0 * std::f64::consts::TAU / 45.0).sin();
    for dx in [-30.0, 0.0, 20.0] {
        let want = 1.0 + gx * (sel_x + dx) + gy * sel_y;
        assert!(
            (ratio(dx) - want).abs() < 1e-6,
            "dx {dx}: {} vs {want}",
            ratio(dx)
        );
    }
}

/// Today's numbers (measured 2026-11-16, recommended tracker, 640×480): minimum share of
/// visible frames tracked within 2 % of the width, and EPE p95 (px) + 10 %.
/// `turn_extreme` was the slice 2d gap (31/90 with the plain gate); the anchored update
/// (ADR-0018) tracks it through the peak, where the turn's own error is largest (p95 4.15).
/// The two occluder scenes lose the point while it is covered, by design.
const PINS: [(&str, f64, f64); 8] = [
    ("illum_gradient", 1.0, 0.30),
    ("nonrigid_warp", 1.0, 0.10),
    ("scale_rotate", 1.0, 1.17),
    ("turn_moderate", 1.0, 1.03),
    ("turn_strong", 1.0, 2.37),
    ("turn_extreme", 1.0, 4.57),
    ("occluder_slide", 0.87, 1.17),
    ("occluder_creep", 0.89, 0.52),
];

#[test]
fn recommended_tracker_holds_its_appearance_change_numbers() {
    let names: Vec<&str> = appearance_corpus().iter().map(|s| s.name).collect();
    assert_eq!(names, PINS.map(|p| p.0), "every appearance scene has a pin");
    for (name, min_ok, max_p95) in PINS {
        let m = measure(name);
        assert_eq!(m.wrong_lock, 0, "{name}: silent wrong lock {m:?}");
        assert_eq!(m.false_tracking, 0, "{name}: {m:?}");
        assert!(m.tracked_ok >= min_ok - 1e-9, "{name}: tracked {m:?}");
        assert!(m.epe_p95 <= max_p95, "{name}: EPE p95 {m:?}");
        assert!(
            m.final_epe.is_some_and(|e| e < 0.5),
            "{name}: ends lost/off {m:?}"
        );
    }
}

/// The plain gate (ADR-0011, before ADR-0018) as measured on 2026-11-15: it loses the
/// extreme turn at its peak (31/90). Kept so the anchored update's gain stays visible and
/// the baseline can't silently change.
#[test]
fn plain_gate_baseline_loses_the_extreme_turn() {
    let m = measure_with(plain_gate(), "turn_extreme", (640, 480));
    assert_eq!(m.wrong_lock, 0, "{m:?}");
    assert!((m.tracked_ok - 31.0 / 90.0).abs() < 1e-9, "{m:?}");
}

#[test]
#[ignore = "prints the table; run with --nocapture"]
fn appearance_table() {
    println!("| scene | ok | EPE med | EPE p95 | EPE max | wrong-lock | final EPE |");
    println!("|---|---|---|---|---|---|---|");
    for s in appearance_corpus() {
        let m = measure(s.name);
        println!(
            "| {} | {:.1}% | {:.3} | {:.3} | {:.3} | {} | {} |",
            s.name,
            100.0 * m.tracked_ok,
            m.epe_median,
            m.epe_p95,
            m.epe_max,
            m.wrong_lock,
            m.final_epe.map_or("lost".into(), |e| format!("{e:.3}")),
        );
    }
}
