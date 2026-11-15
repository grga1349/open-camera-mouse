//! Replays an `OCM_RECORD` recording through the real controller (test drive, task 0009).
//!
//! Ignored by default: recordings contain the user's face and never enter the repository.
//! Run with
//!
//! ```text
//! OCM_REPLAY=<recording dir> cargo test -p app-core --test replay -- --ignored --nocapture
//! ```
//!
//! Optional: `OCM_REPLAY_RECOVERY=0|1` (default 1), `OCM_REPLAY_OUT=<file.csv>` for a per-frame
//! trace. Selections come from the recording's `events.csv` (`TargetSelected`), applied before
//! the first frame captured at or after the event time.

use std::fs;
use std::path::{Path, PathBuf};

use app_core::ports::{CameraFrame, CaptureFormat, Point2};
use app_core::{AppState, Command, Controller, DesktopRect, Input, PointerAccess, Settings};
use tracking_core::{GrayFrame, TrackStatus};

const DESK: DesktopRect = DesktopRect {
    x: 0.0,
    y: 0.0,
    width: 1920.0,
    height: 1080.0,
};

fn read_pgm(path: &Path) -> GrayFrame {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    // "P5\n{w} {h}\n255\n" as the recorder writes it.
    let mut fields = Vec::new();
    let mut i = 0;
    while fields.len() < 4 {
        let start = i;
        while !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        fields.push(String::from_utf8_lossy(&bytes[start..i]).into_owned());
        i += 1;
    }
    assert_eq!(fields[0], "P5");
    let w: u32 = fields[1].parse().unwrap();
    let h: u32 = fields[2].parse().unwrap();
    GrayFrame::from_vec(w, h, bytes[i..].to_vec()).unwrap()
}

fn selections(dir: &Path) -> Vec<(u64, Point2)> {
    let text = fs::read_to_string(dir.join("events.csv")).unwrap_or_default();
    text.lines()
        .skip(1)
        .filter_map(|l| {
            let (t, ev) = l.split_once(',')?;
            let rest = ev.strip_prefix("\"TargetSelected { at: Point2 { x: ")?;
            let (x, rest) = rest.split_once(", y: ")?;
            let y = rest.split(' ').next()?;
            Some((
                t.parse().ok()?,
                Point2::new(x.parse().ok()?, y.parse().ok()?),
            ))
        })
        .collect()
}

#[derive(Debug, Default)]
pub struct Summary {
    pub frames: usize,
    pub tracking: usize,
    pub lost: usize,
    pub losses: Vec<(u64, String)>,
    pub recoveries: usize,
}

fn status_name(s: Option<TrackStatus>) -> String {
    match s {
        None => "-".into(),
        Some(s) => format!("{s:?}"),
    }
}

#[test]
#[ignore = "needs a local OCM_RECORD recording (OCM_REPLAY=<dir>)"]
fn replay_recording() {
    let Some(dir) = std::env::var_os("OCM_REPLAY").map(PathBuf::from) else {
        eprintln!("OCM_REPLAY not set; nothing to replay");
        return;
    };
    let recovery = std::env::var("OCM_REPLAY_RECOVERY").map_or(true, |v| v != "0");
    let steady_px = std::env::var("OCM_REPLAY_STEADY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let settings = Settings {
        loss_recovery: recovery,
        steady_px,
        ..Settings::default()
    };
    let mut c = Controller::new(settings, DESK).unwrap();
    let frames_csv = fs::read_to_string(dir.join("frames.csv")).expect("frames.csv");
    let mut sel = selections(&dir).into_iter().peekable();
    let mut out = String::from("t_us,state,status,x,y,zncc,residual,min_eigen,armed,ptr_x,ptr_y\n");

    let mut now = 0u64;
    let mut started = false;
    let mut sum = Summary::default();
    let mut prev_state = AppState::Stopped;
    for line in frames_csv.lines().skip(1) {
        let mut it = line.split(',');
        let seq: u64 = it.next().unwrap().parse().unwrap();
        let ts: u64 = it.next().unwrap().parse().unwrap();
        now = ts;
        if !started {
            c.handle(Input::Command(Command::ToggleStart), now);
            c.handle(Input::PointerAccess(PointerAccess::Granted), now);
            c.handle(
                Input::CameraOpened(CaptureFormat {
                    width: 640,
                    height: 480,
                    fps: 30.0,
                }),
                now,
            );
            started = true;
        }
        let luma = read_pgm(&dir.join(format!("frame_{seq:06}.pgm")));
        // A selection made before this frame was captured goes in first.
        while sel.peek().is_some_and(|(t, _)| *t <= ts) {
            let (_, at) = sel.next().unwrap();
            c.handle(Input::Command(Command::SelectTarget(at)), now);
        }
        let step = c.handle(
            Input::Frame(CameraFrame {
                luma,
                rgb: None,
                timestamp_us: ts,
            }),
            now,
        );
        // Report the cursor where the controller put it, so no handover is detected.
        let ptr = step.pointer_target();
        c.handle(Input::PointerObserved(ptr), now);

        let v = c.view(now);
        sum.frames += 1;
        match v.state {
            AppState::Tracking => sum.tracking += 1,
            AppState::Lost => sum.lost += 1,
            _ => {}
        }
        if v.state != prev_state {
            if v.state == AppState::Lost {
                sum.losses.push((now, status_name(v.track)));
            }
            if prev_state == AppState::Lost && v.state == AppState::Tracking {
                sum.recoveries += 1;
            }
            prev_state = v.state;
        }
        let d = v.diagnostics;
        let f = |o: Option<f32>| o.map_or(String::new(), |x| format!("{x:.3}"));
        let (mx, my) = v.marker.map_or((String::new(), String::new()), |m| {
            (
                format!("{:.2}", m.position.x),
                format!("{:.2}", m.position.y),
            )
        });
        let (px, py) = ptr.map_or((String::new(), String::new()), |p| {
            (format!("{:.2}", p.x), format!("{:.2}", p.y))
        });
        out.push_str(&format!(
            "{now},{:?},{},{mx},{my},{},{},{},{},{px},{py}\n",
            v.state,
            status_name(v.track),
            f(d.score),
            f(d.residual),
            f(d.min_eigen),
            v.redetect_armed,
        ));
    }
    let _ = now;
    if let Some(p) = std::env::var_os("OCM_REPLAY_OUT") {
        fs::write(&p, out).unwrap();
    }
    println!("{sum:#?}");
}

/// Re-detection diagnostics (test drive #3): the selection self-test's scores per selection,
/// and the search's best/second scores on every later frame, whole frame and within the
/// spatial prior.
#[test]
#[ignore = "needs a local OCM_RECORD recording (OCM_REPLAY=<dir>)"]
fn replay_redetect_scores() {
    use tracking_core::pyramid::Pyramid;
    use tracking_core::redetect::{RedetectParams, SearchResult, Searcher};
    let Some(dir) = std::env::var_os("OCM_REPLAY").map(PathBuf::from) else {
        return;
    };
    let frames_csv = fs::read_to_string(dir.join("frames.csv")).expect("frames.csv");
    let frames: Vec<(u64, u64)> = frames_csv
        .lines()
        .skip(1)
        .map(|l| {
            let mut it = l.split(',');
            (
                it.next().unwrap().parse().unwrap(),
                it.next().unwrap().parse().unwrap(),
            )
        })
        .collect();
    let load = |seq: u64| {
        let mut f = read_pgm(&dir.join(format!("frame_{seq:06}.pgm")));
        f.mirror_horizontal();
        f
    };
    let rp = RedetectParams {
        max_radius_frac: Some(0.25),
        ..RedetectParams::default()
    };
    let show = |r: SearchResult| match r {
        SearchResult::Found { position, score } => {
            format!("Found {score:.3} at ({:.1},{:.1})", position.x, position.y)
        }
        SearchResult::NotFound { best, second } => {
            format!("NotFound best {best:.3} second {second:.3}")
        }
    };
    let stride: usize = std::env::var("OCM_REPLAY_STRIDE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    for (t, at) in selections(&dir) {
        let i = frames.iter().position(|f| f.1 >= t).unwrap();
        let mut pyr = Pyramid::default();
        pyr.build(&load(frames[i].0), 3);
        let mut s = Searcher::default();
        s.prepare(&pyr, at, 10, &rp);
        println!(
            "selection at ({:.1},{:.1}) frame {}: ready {}; self-test whole frame: {}; within prior: {}",
            at.x,
            at.y,
            frames[i].0,
            s.is_ready(),
            show(s.search(&pyr, &rp, None)),
            show(s.search(&pyr, &rp, Some((at, 160.0)))),
        );
        for (seq, _) in frames.iter().skip(i + 1).step_by(stride).take(40) {
            pyr.build(&load(*seq), 3);
            println!(
                "  frame {seq}: {}",
                show(s.search(&pyr, &rp, Some((at, 160.0))))
            );
        }
    }
}

/// Where the best match lands with no ambiguity margin, over the first lost stretch
/// (test drive #3): does the top peak stay on the selected spot?
#[test]
#[ignore = "needs a local OCM_RECORD recording (OCM_REPLAY=<dir>)"]
fn replay_redetect_best_positions() {
    use tracking_core::pyramid::Pyramid;
    use tracking_core::redetect::{RedetectParams, SearchResult, Searcher};
    let Some(dir) = std::env::var_os("OCM_REPLAY").map(PathBuf::from) else {
        return;
    };
    let frames_csv = fs::read_to_string(dir.join("frames.csv")).expect("frames.csv");
    let frames: Vec<(u64, u64)> = frames_csv
        .lines()
        .skip(1)
        .map(|l| {
            let mut it = l.split(',');
            (
                it.next().unwrap().parse().unwrap(),
                it.next().unwrap().parse().unwrap(),
            )
        })
        .collect();
    let load = |seq: u64| {
        let mut f = read_pgm(&dir.join(format!("frame_{seq:06}.pgm")));
        f.mirror_horizontal();
        f
    };
    let rp = RedetectParams {
        max_radius_frac: Some(0.25),
        min_margin: 0.0,
        min_zncc: 0.0,
        ..RedetectParams::default()
    };
    let sels = selections(&dir);
    let (t, at) = sels[0];
    let i = frames.iter().position(|f| f.1 >= t).unwrap();
    let mut pyr = Pyramid::default();
    pyr.build(&load(frames[i].0), 3);
    let mut s = Searcher::default();
    s.prepare(&pyr, at, 10, &rp);
    let to: u64 = std::env::var("OCM_REPLAY_TO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2000);
    for (seq, _) in frames.iter().skip(i + 1).step_by(20) {
        if *seq > to {
            break;
        }
        pyr.build(&load(*seq), 3);
        if let SearchResult::Found { position, score } = s.search(&pyr, &rp, Some((at, 160.0))) {
            println!(
                "frame {seq}: {score:.3} at ({:.1},{:.1})",
                position.x, position.y
            );
        }
    }
    if let Some((_, again)) = sels.get(1) {
        println!("user re-selected at ({:.1},{:.1})", again.x, again.y);
    }
}

/// Larger-context ZNCC between the selection frame and a recovery position (test drive #3):
/// `OCM_REPLAY_CHECK="seq,x,y,ref_seq,ref_x,ref_y;…"`, prints the score per half window.
#[test]
#[ignore = "needs a local OCM_RECORD recording (OCM_REPLAY=<dir>)"]
fn replay_context_zncc() {
    let Some(dir) = std::env::var_os("OCM_REPLAY").map(PathBuf::from) else {
        return;
    };
    let load = |seq: u64| {
        let mut f = read_pgm(&dir.join(format!("frame_{seq:06}.pgm")));
        f.mirror_horizontal();
        f
    };
    let win = |f: &GrayFrame, x: f32, y: f32, hw: i64| -> Option<Vec<f32>> {
        let (xi, yi) = (x.round() as i64, y.round() as i64);
        let (w, h) = (i64::from(f.width()), i64::from(f.height()));
        if xi - hw < 0 || yi - hw < 0 || xi + hw >= w || yi + hw >= h {
            return None;
        }
        let b = f.as_bytes();
        let mut v = Vec::new();
        for yy in yi - hw..=yi + hw {
            for xx in xi - hw..=xi + hw {
                v.push(f32::from(b[(yy * w + xx) as usize]));
            }
        }
        Some(v)
    };
    let zncc = |a: &[f32], b: &[f32]| {
        let n = a.len() as f32;
        let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
        let (mut ab, mut aa, mut bb) = (0.0, 0.0, 0.0);
        for (x, y) in a.iter().zip(b) {
            ab += (x - ma) * (y - mb);
            aa += (x - ma) * (x - ma);
            bb += (y - mb) * (y - mb);
        }
        ab / (aa * bb).sqrt().max(1e-6)
    };
    let spec = std::env::var("OCM_REPLAY_CHECK").unwrap_or_default();
    for item in spec.split(';').filter(|s| !s.is_empty()) {
        let v: Vec<f32> = item.split(',').map(|s| s.parse().unwrap()).collect();
        let (cur, refr) = (load(v[0] as u64), load(v[3] as u64));
        let scores: Vec<String> = [10, 20, 30, 40]
            .iter()
            .map(
                |&hw| match (win(&cur, v[1], v[2], hw), win(&refr, v[4], v[5], hw)) {
                    (Some(a), Some(b)) => format!("hw{hw} {:.3}", zncc(&a, &b)),
                    _ => format!("hw{hw} -"),
                },
            )
            .collect();
        println!("{item}: {}", scores.join("  "));
    }
}

/// Tracker tuning on a recording (test drive #2): runs tracker variants over the same frames
/// and selections and prints, per variant, the losses, the share of frames tracked and a
/// drift proxy (share of tracked frames whose 41×41 surroundings score below 0.5 against the
/// selection: the point has probably slid off the selected spot).
/// `OCM_REPLAY_FRAMES` caps the frames (default all).
#[test]
#[ignore = "needs a local OCM_RECORD recording (OCM_REPLAY=<dir>)"]
fn replay_tune() {
    use tracking_core::Tracker;
    use tracking_core::lk::{LkParams, LkTracker};
    let Some(dir) = std::env::var_os("OCM_REPLAY").map(PathBuf::from) else {
        return;
    };
    let cap: usize = std::env::var("OCM_REPLAY_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);
    let frames_csv = fs::read_to_string(dir.join("frames.csv")).expect("frames.csv");
    let frames: Vec<(u64, u64)> = frames_csv
        .lines()
        .skip(1)
        .take(cap)
        .map(|l| {
            let mut it = l.split(',');
            (
                it.next().unwrap().parse().unwrap(),
                it.next().unwrap().parse().unwrap(),
            )
        })
        .collect();
    let load = |seq: u64| {
        let mut f = read_pgm(&dir.join(format!("frame_{seq:06}.pgm")));
        f.mirror_horizontal();
        f
    };
    let sels = selections(&dir);
    let ctx = |f: &GrayFrame, p: Point2| -> Option<Vec<f32>> {
        let (xi, yi) = (p.x.round() as i64, p.y.round() as i64);
        let (w, h) = (i64::from(f.width()), i64::from(f.height()));
        let hw = 20;
        if xi - hw < 0 || yi - hw < 0 || xi + hw >= w || yi + hw >= h {
            return None;
        }
        let b = f.as_bytes();
        let mut v = Vec::with_capacity(41 * 41);
        for yy in yi - hw..=yi + hw {
            for xx in xi - hw..=xi + hw {
                v.push(f32::from(b[(yy * w + xx) as usize]));
            }
        }
        let m = v.iter().sum::<f32>() / v.len() as f32;
        let n = v
            .iter()
            .map(|x| (x - m) * (x - m))
            .sum::<f32>()
            .sqrt()
            .max(1e-6);
        Some(v.iter().map(|x| (x - m) / n).collect())
    };

    let rec = LkParams::recommended();
    let with = |f: &dyn Fn(&mut LkParams)| {
        let mut p = rec;
        f(&mut p);
        p
    };
    let variants: Vec<(&str, LkParams)> = vec![
        ("recommended (gate 0.7, hw 10)", rec),
        ("gate 0.6", with(&|p| p.min_ref_zncc = Some(0.6))),
        ("gate 0.5", with(&|p| p.min_ref_zncc = Some(0.5))),
        ("gate off", with(&|p| p.min_ref_zncc = None)),
        ("hw 15", with(&|p| p.half_window = 15)),
        (
            "hw 15, gate 0.6",
            with(&|p| {
                p.half_window = 15;
                p.min_ref_zncc = Some(0.6);
            }),
        ),
        (
            "hw 12, gate 0.6",
            with(&|p| {
                p.half_window = 12;
                p.min_ref_zncc = Some(0.6);
            }),
        ),
        ("hw 20", with(&|p| p.half_window = 20)),
        (
            "hw 20, gate 0.6",
            with(&|p| {
                p.half_window = 20;
                p.min_ref_zncc = Some(0.6);
            }),
        ),
        (
            "hw 15, gate 0.5",
            with(&|p| {
                p.half_window = 15;
                p.min_ref_zncc = Some(0.5);
            }),
        ),
        (
            "motion guard 70/640",
            with(&|p| p.max_motion_frac = 70.0 / 640.0),
        ),
        ("min_eigen 0.5", with(&|p| p.min_eigen = 0.5)),
    ];
    // Decode once: frames are reused by every variant.
    let decoded: Vec<GrayFrame> = frames.iter().map(|(s, _)| load(*s)).collect();
    println!(
        "{:<32} {:>6} {:>8} {:>8}",
        "variant", "losses", "tracked", "drift"
    );
    for (name, params) in variants {
        let mut t = LkTracker::new(params);
        let mut sel = sels.iter().peekable();
        let mut sel_ctx: Option<Vec<f32>> = None;
        let (mut losses, mut tracked, mut drift, mut checked) = (0, 0, 0, 0);
        let mut was_tracking = false;
        let mut active = false;
        for (i, (_, ts)) in frames.iter().enumerate() {
            let f = &decoded[i];
            while sel.peek().is_some_and(|(t0, _)| *t0 <= *ts) {
                let (_, at) = sel.next().unwrap();
                t.select(f, *at).unwrap();
                sel_ctx = ctx(f, *at);
                active = true;
                was_tracking = true;
            }
            if !active {
                continue;
            }
            let out = t.step(f).unwrap();
            let ok = matches!(out.status, TrackStatus::Tracking | TrackStatus::Held(_));
            if ok {
                tracked += 1;
                if i % 10 == 0
                    && let (Some(r), Some(c)) = (&sel_ctx, ctx(f, out.position))
                {
                    checked += 1;
                    let z: f32 = r.iter().zip(&c).map(|(a, b)| a * b).sum();
                    if z < 0.5 {
                        drift += 1;
                    }
                }
            }
            if was_tracking && !ok {
                losses += 1;
            }
            was_tracking = ok;
        }
        let n = frames.len().max(1);
        println!(
            "{name:<32} {losses:>6} {:>7.0}% {:>7.0}%",
            100.0 * tracked as f32 / n as f32,
            100.0 * drift as f32 / checked.max(1) as f32
        );
    }
}
