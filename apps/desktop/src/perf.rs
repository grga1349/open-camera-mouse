//! Frame-path timing for diagnostics (Phase 6b, test-drive lag investigation, ADR-0016).
//!
//! Off unless `OCM_PERF=1` is set. Every [`REPORT_EVERY`] the engine prints one line to
//! stderr with p50 / p95 / max of each stage, so "the cursor lags" can be answered with
//! numbers:
//!
//! - `in`: camera frame timestamp → the engine takes it (decode, hand-offs, engine busy).
//! - `track`: `Controller::handle(Frame)` (ingest, mirror, tracker, pointer mapping).
//! - `move`: performing the effects of that frame (the OS pointer move).
//! - `age`: camera frame timestamp → its pointer move is done. The latency the user feels,
//!   minus the camera's own sensor-to-timestamp delay.
//! - `upload`: preview texture upload on the UI thread (the image is built by the engine).
//! - `ui`: the whole `ui` pass (layout, paint).
//!
//! Counts: frames given to the controller per second, and frames the camera worker replaced
//! before the engine took them (engine behind; running total).
//!
//! Tracking (v0.4.1 slice 1, [`TrackLog`]): one `track` line per loss (reason, ZNCC against
//! the selection and against the lag window, residual, position in frame px) and per return
//! to tracking (frames lost, re-detected or re-selected), and a `frames` line with each
//! report counting frames per tracker status. Numbers only, no imagery.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use app_core::{HoldReason, TrackOutput, TrackStatus};

pub const REPORT_EVERY: Duration = Duration::from_secs(5);

/// Shared by the engine thread (frame stages, report) and the UI thread (upload, ui).
pub type SharedPerf = Arc<Mutex<Perf>>;

#[derive(Debug, Default)]
struct Stat(Vec<u64>);

impl Stat {
    fn add(&mut self, us: u64) {
        self.0.push(us);
    }

    fn line(&mut self, name: &str) -> String {
        if self.0.is_empty() {
            return format!("{name} -");
        }
        self.0.sort_unstable();
        let at = |q: f64| self.0[((self.0.len() - 1) as f64 * q).round() as usize];
        let ms = |us: u64| us as f64 / 1000.0;
        let s = format!(
            "{name} {:.1}/{:.1}/{:.1}",
            ms(at(0.5)),
            ms(at(0.95)),
            ms(*self.0.last().unwrap_or(&0))
        );
        self.0.clear();
        s
    }
}

#[derive(Debug)]
pub struct Perf {
    since: Instant,
    frames: u64,
    input: Stat,
    track: Stat,
    moves: Stat,
    age: Stat,
    upload: Stat,
    ui: Stat,
    start: Instant,
    status: TrackLog,
}

impl Perf {
    pub fn from_env() -> Option<SharedPerf> {
        let on = std::env::var("OCM_PERF").is_ok_and(|v| v == "1");
        on.then(|| {
            eprintln!(
                "OCM_PERF: every {} s, ms p50/p95/max (see perf.rs)",
                REPORT_EVERY.as_secs()
            );
            Arc::new(Mutex::new(Self {
                since: Instant::now(),
                frames: 0,
                input: Stat::default(),
                track: Stat::default(),
                moves: Stat::default(),
                age: Stat::default(),
                upload: Stat::default(),
                ui: Stat::default(),
                start: Instant::now(),
                status: TrackLog::default(),
            }))
        })
    }

    /// One frame through the engine; `input_us` and `age_us` are from its capture timestamp.
    pub fn frame(&mut self, input_us: u64, track: Duration, moves: Duration, age_us: u64) {
        self.frames += 1;
        self.input.add(input_us);
        self.track.add(us(track));
        self.moves.add(us(moves));
        self.age.add(age_us);
    }

    /// The controller's track after a frame ([`app_core::Controller::track`]).
    pub fn track(&mut self, track: Option<TrackOutput>) {
        let t = self.start.elapsed().as_secs_f64();
        if let Some(line) = self.status.frame(t, track) {
            eprintln!("{line}");
        }
    }

    /// The user selected a target.
    pub fn selected(&mut self) {
        self.status.selected();
    }

    pub fn upload(&mut self, d: Duration) {
        self.upload.add(us(d));
    }

    pub fn ui(&mut self, d: Duration) {
        self.ui.add(us(d));
    }

    /// Prints the report when due. `replaced` is the camera link's running total.
    pub fn report_if_due(&mut self, replaced: u64) {
        let now = Instant::now();
        let span = now - self.since;
        if span < REPORT_EVERY {
            return;
        }
        eprintln!(
            "perf {:.1} fps | replaced {replaced} | {} | {} | {} | {} | {} | {}",
            self.frames as f64 / span.as_secs_f64(),
            self.input.line("in"),
            self.track.line("track"),
            self.moves.line("move"),
            self.age.line("age"),
            self.upload.line("upload"),
            self.ui.line("ui"),
        );
        eprintln!("{}", self.status.report());
        self.frames = 0;
        self.since = now;
    }
}

/// Per-frame tracker statuses for `OCM_PERF` (module doc). Pure: the caller prints.
#[derive(Debug, Default)]
pub struct TrackLog {
    tracking: u64,
    held_edge: u64,
    held_fast: u64,
    /// `Held(AppearanceDegraded)`: a borderline appearance check (ADR-0018 amendment).
    held_look: u64,
    lost: u64,
    no_target: u64,
    /// Losses since start (numbered in the lines).
    losses: u64,
    /// Frames in the current loss, while lost.
    lost_for: Option<u64>,
    /// A selection was made during the current loss.
    reselected: bool,
}

impl TrackLog {
    /// One frame. `t_s` is seconds since the log started. Returns a line on a loss and on
    /// the end of one.
    pub fn frame(&mut self, t_s: f64, track: Option<TrackOutput>) -> Option<String> {
        let status = track.map(|t| t.status);
        match status {
            Some(TrackStatus::Tracking) => self.tracking += 1,
            Some(TrackStatus::Held(HoldReason::NearEdgeMovingOut)) => self.held_edge += 1,
            Some(TrackStatus::Held(HoldReason::MotionOutlier)) => self.held_fast += 1,
            Some(TrackStatus::Held(HoldReason::AppearanceDegraded)) => self.held_look += 1,
            Some(TrackStatus::Lost(_)) => self.lost += 1,
            None => self.no_target += 1,
        }
        match (track, self.lost_for) {
            (
                Some(
                    t @ TrackOutput {
                        status: TrackStatus::Lost(reason),
                        ..
                    },
                ),
                None,
            ) => {
                self.losses += 1;
                self.lost_for = Some(1);
                self.reselected = false;
                let d = t.diagnostics;
                Some(format!(
                    "track {t_s:.1} s: lost #{} {reason:?} at ({:.1}, {:.1}) | zncc {} | \
                     anchor {} | residual {}",
                    self.losses,
                    t.position.x,
                    t.position.y,
                    num(d.score),
                    num(d.anchor_score),
                    num(d.residual),
                ))
            }
            (
                Some(TrackOutput {
                    status: TrackStatus::Lost(_),
                    ..
                }),
                Some(n),
            ) => {
                self.lost_for = Some(n + 1);
                None
            }
            (_, Some(n)) => {
                self.lost_for = None;
                let how = match (status, self.reselected) {
                    (None, _) => "no target",
                    (Some(_), true) => "re-selected",
                    (Some(_), false) => "re-detected",
                };
                Some(format!(
                    "track {t_s:.1} s: lost #{} ended after {n} frames ({how})",
                    self.losses
                ))
            }
            (_, None) => None,
        }
    }

    pub fn selected(&mut self) {
        if self.lost_for.is_some() {
            self.reselected = true;
        }
    }

    /// Frame counts since the last report, then reset.
    pub fn report(&mut self) -> String {
        let s = format!(
            "frames tracking {} | held edge {} | held fast {} | held look {} | lost {} | \
             no target {} | losses {}",
            self.tracking,
            self.held_edge,
            self.held_fast,
            self.held_look,
            self.lost,
            self.no_target,
            self.losses
        );
        (
            self.tracking,
            self.held_edge,
            self.held_fast,
            self.held_look,
            self.lost,
            self.no_target,
        ) = (0, 0, 0, 0, 0, 0);
        s
    }
}

fn num(v: Option<f32>) -> String {
    v.map_or_else(|| "-".to_owned(), |x| format!("{x:.2}"))
}

fn us(d: Duration) -> u64 {
    d.as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_and_reset() {
        let mut s = Stat::default();
        for v in [5_000, 1_000, 3_000, 2_000, 4_000] {
            s.add(v);
        }
        assert_eq!(s.line("x"), "x 3.0/5.0/5.0");
        assert_eq!(s.line("x"), "x -");
    }

    use app_core::ports::Point2;
    use app_core::{Diagnostics, LossReason};

    fn out(status: TrackStatus) -> Option<TrackOutput> {
        Some(TrackOutput {
            position: Point2::new(320.0, 240.5),
            status,
            diagnostics: Diagnostics {
                score: Some(0.81),
                anchor_score: Some(0.64),
                ..Diagnostics::default()
            },
        })
    }

    #[test]
    fn track_log_reports_each_loss_and_its_end() {
        let mut log = TrackLog::default();
        let lost = out(TrackStatus::Lost(LossReason::AppearanceChanged));
        assert_eq!(log.frame(0.0, out(TrackStatus::Tracking)), None);
        assert_eq!(
            log.frame(1.25, lost).as_deref(),
            Some(
                "track 1.2 s: lost #1 AppearanceChanged at (320.0, 240.5) | zncc 0.81 | \
                 anchor 0.64 | residual -"
            )
        );
        assert_eq!(log.frame(1.3, lost), None);
        assert_eq!(
            log.frame(1.4, out(TrackStatus::Tracking)).as_deref(),
            Some("track 1.4 s: lost #1 ended after 2 frames (re-detected)")
        );
        log.frame(2.0, out(TrackStatus::Held(HoldReason::NearEdgeMovingOut)));
        log.frame(2.1, out(TrackStatus::Held(HoldReason::MotionOutlier)));
        log.frame(2.2, out(TrackStatus::Held(HoldReason::AppearanceDegraded)));
        assert_eq!(
            log.report(),
            "frames tracking 2 | held edge 1 | held fast 1 | held look 1 | lost 2 | \
             no target 0 | losses 1"
        );
        assert_eq!(
            log.report(),
            "frames tracking 0 | held edge 0 | held fast 0 | held look 0 | lost 0 | \
             no target 0 | losses 1"
        );
    }

    #[test]
    fn track_log_tells_reselection_and_no_target_apart() {
        let mut log = TrackLog::default();
        let lost = out(TrackStatus::Lost(LossReason::LowConfidence));
        log.selected(); // not lost: ignored
        assert!(log.frame(0.0, lost).is_some());
        log.selected();
        assert_eq!(
            log.frame(0.1, out(TrackStatus::Tracking)).as_deref(),
            Some("track 0.1 s: lost #1 ended after 1 frames (re-selected)")
        );
        assert!(log.frame(0.2, lost).is_some());
        assert_eq!(
            log.frame(0.3, None).as_deref(),
            Some("track 0.3 s: lost #2 ended after 1 frames (no target)")
        );
        assert_eq!(log.frame(0.4, None), None);
    }
}
