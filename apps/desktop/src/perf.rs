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

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
        self.frames = 0;
        self.since = now;
    }
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
}
