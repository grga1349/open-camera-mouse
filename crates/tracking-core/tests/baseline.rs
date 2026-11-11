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
