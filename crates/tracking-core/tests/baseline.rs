//! Phase 3 baseline: B-LK18 vs B-NCC on the full §8 corpus at three resolutions.
//!
//! Run: `cargo test -p tracking-core --release --test baseline -- --ignored --nocapture`
//! Prints markdown tables (copied into docs/research/TRACKING_BASELINE.md). Ignored by default
//! because the full corpus takes minutes.

mod support;

use support::{Metrics, RESOLUTIONS, corpus, metrics, run};
use tracking_core::lk::LkTracker;
use tracking_core::ncc::NccTracker;

fn fmt(v: f64) -> String {
    if v.is_nan() {
        "—".into()
    } else {
        format!("{v:.2}")
    }
}

fn opt(v: Option<usize>) -> String {
    v.map_or("—".into(), |v| v.to_string())
}

fn row(scene: &str, res: (u32, u32), alg: &str, m: &Metrics) -> String {
    format!(
        "| {scene} | {}×{} | {alg} | {:.0}% | {} | {} | {} | {} | {} | {} | {} | {} |",
        res.0,
        res.1,
        100.0 * m.tracked_ok,
        fmt(m.epe_median),
        fmt(m.epe_p95),
        fmt(m.epe_max),
        m.wrong_lock,
        m.false_tracking,
        opt(m.frames_to_detect_loss),
        opt(m.frames_to_recover),
        m.final_epe.map_or("lost".into(), fmt),
    )
}

#[test]
#[ignore = "full corpus; run explicitly in --release"]
fn baseline_tables() {
    let only = std::env::var("BASELINE_SCENE").ok();
    println!(
        "| scene | input | alg | ok | EPE med | EPE p95 | EPE max | wrong-lock | false-track | detect | recover | final EPE |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|");
    for scene in corpus() {
        if only.as_deref().is_some_and(|o| o != scene.name) {
            continue;
        }
        for res in RESOLUTIONS {
            if scene.name == "static_60s" && res != (640, 480) {
                continue;
            }
            let seq = scene.render(res.0, res.1);
            let lk = metrics(&seq, &run(&mut LkTracker::default(), &seq));
            let ncc = metrics(&seq, &run(&mut NccTracker::default(), &seq));
            println!("{}", row(scene.name, res, "LK18", &lk));
            println!("{}", row(scene.name, res, "NCC", &ncc));
        }
    }
}

/// Exact fingerprint of every B-LK18 output (positions as f32 bits + status) per scene and
/// resolution. Refactors that claim "no behavior change" (Phase 4 E0) must leave this output
/// unchanged; the markdown table's 2-decimal rounding could hide a drift.
///
/// Run: `cargo test -p tracking-core --release --test baseline lk_fingerprint -- --ignored --nocapture`
#[test]
#[ignore = "full corpus; run explicitly in --release"]
fn lk_fingerprint() {
    use std::hash::{DefaultHasher, Hash, Hasher};
    for scene in corpus() {
        for res in RESOLUTIONS {
            if scene.name == "static_60s" && res != (640, 480) {
                continue;
            }
            let seq = scene.render(res.0, res.1);
            let mut h = DefaultHasher::new();
            for s in run(&mut LkTracker::default(), &seq) {
                s.position.x.to_bits().hash(&mut h);
                s.position.y.to_bits().hash(&mut h);
                format!("{:?}", s.status).hash(&mut h);
            }
            println!("{} {}x{} {:016x}", scene.name, res.0, res.1, h.finish());
        }
    }
}
