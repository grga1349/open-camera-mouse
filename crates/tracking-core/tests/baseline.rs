//! Phase 3 baseline: B-LK18 vs B-NCC on the full §8 corpus at three resolutions.
//!
//! Run: `cargo test -p tracking-core --release --test baseline -- --ignored --nocapture`
//! Prints markdown tables (copied into docs/research/TRACKING_BASELINE.md). Ignored by default
//! because the full corpus takes minutes.

mod support;

use support::{Metrics, RESOLUTIONS, corpus, metrics, run};
use tracking_core::lk::{LkParams, LkTracker};
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

/// Phase 4 ablation variants. `LK18` (all experiment flags off) must stay first: every other
/// row is compared against it. Add one entry per experiment setting; keep rejected ones so
/// their numbers stay reproducible.
fn variants() -> Vec<(&'static str, LkParams)> {
    let d = LkParams::default();
    let fb = |e: f32| LkParams {
        fb_max_error: Some(e),
        ..d
    };
    vec![
        ("LK18", d),
        ("E1 fb≤0.25", fb(0.25)),
        ("E1 fb≤0.5", fb(0.5)),
        ("E1 fb≤1", fb(1.0)),
        ("E1 fb≤2", fb(2.0)),
    ]
}

/// Task 0006 gate scenes: kept experiments may not regress EPE p95 by > 0.1 px here.
const LK_GOOD: [&str; 9] = [
    "translate_int",
    "translate_subpx",
    "head_sway",
    "stall",
    "scale_10",
    "rotate_15",
    "sensor_noise",
    "static_60s",
    "motion_blur",
];

/// Phase 4 ablation: each `variants()` entry vs B-LK18 on the same corpus and metrics.
///
/// Run: `cargo test -p tracking-core --release --test baseline ablation_tables -- --ignored --nocapture`
/// Optional filters: `BASELINE_SCENE=<scene>`, `ABLATION=<substring of variant name>` (LK18
/// always runs).
#[test]
#[ignore = "full corpus; run explicitly in --release"]
fn ablation_tables() {
    let only = std::env::var("BASELINE_SCENE").ok();
    let pick = std::env::var("ABLATION").ok();
    let vars: Vec<_> = variants()
        .into_iter()
        .enumerate()
        .filter(|(i, (n, _))| *i == 0 || pick.as_deref().is_none_or(|p| n.contains(p)))
        .map(|(_, v)| v)
        .collect();
    // Per variant: Σ wrong-lock, Σ false-track, Σ ok fraction, worst good-scene Δp95 and
    // where, scenes with more wrong-lock frames than LK18.
    struct Sum {
        wrong: usize,
        false_t: usize,
        ok: f64,
        rows: usize,
        worst: (f64, String),
        new_wrong: Vec<String>,
    }
    let mut sums: Vec<Sum> = vars
        .iter()
        .map(|_| Sum {
            wrong: 0,
            false_t: 0,
            ok: 0.0,
            rows: 0,
            worst: (f64::NEG_INFINITY, String::new()),
            new_wrong: Vec::new(),
        })
        .collect();
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
            let ms: Vec<Metrics> = vars
                .iter()
                .map(|(_, p)| metrics(&seq, &run(&mut LkTracker::new(*p), &seq)))
                .collect();
            let tag = format!("{} {}×{}", scene.name, res.0, res.1);
            for (i, ((name, _), m)) in vars.iter().zip(&ms).enumerate() {
                println!("{}", row(scene.name, res, name, m));
                let s = &mut sums[i];
                s.wrong += m.wrong_lock;
                s.false_t += m.false_tracking;
                s.ok += m.tracked_ok;
                s.rows += 1;
                if LK_GOOD.contains(&scene.name) {
                    // A variant that stops tracking a good scene entirely is an infinite regression.
                    let d = if m.epe_p95.is_nan() {
                        f64::INFINITY
                    } else {
                        m.epe_p95 - ms[0].epe_p95
                    };
                    if d > s.worst.0 {
                        s.worst = (d, tag.clone());
                    }
                }
                if m.wrong_lock > ms[0].wrong_lock {
                    s.new_wrong.push(tag.clone());
                }
            }
        }
    }
    println!();
    println!(
        "| alg | mean ok | Σ wrong-lock | Σ false-track | worst good-scene Δp95 | more wrong-lock than LK18 |"
    );
    println!("|---|---|---|---|---|---|");
    for ((name, _), s) in vars.iter().zip(&sums) {
        println!(
            "| {name} | {:.1}% | {} | {} | {:+.2} ({}) | {} |",
            100.0 * s.ok / s.rows.max(1) as f64,
            s.wrong,
            s.false_t,
            s.worst.0,
            s.worst.1,
            if s.new_wrong.is_empty() {
                "none".into()
            } else {
                s.new_wrong.join(", ")
            },
        );
    }
}
