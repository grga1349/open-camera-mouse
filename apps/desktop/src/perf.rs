//! Frame-path timing for diagnostics (Phase 6b, test-drive lag investigation).
//!
//! Off unless `OCM_PERF=1` is set. Every [`REPORT_EVERY`] it prints one line to stderr with
//! p50 / p95 / max of each stage, so "the preview lags" can be answered with numbers:
//!
//! - `age`: camera frame timestamp → the controller gets it (decode, two hand-offs, waiting
//!   for the UI loop). This is the latency the pointer inherits.
//! - `track`: `Controller::handle(Frame)` (ingest, mirror, tracker, pointer mapping).
//! - `move`: performing the effects of that frame (the OS pointer move).
//! - `upload`: preview `ColorImage` build + texture upload.
//! - `ui`: the whole `ui` pass (layout, paint).
//! - `gap`: time between two `logic` calls (how often the loop runs at all).
//!
//! Counts: frames given to the controller, frames the worker dropped (UI behind), frames
//! coalesced away in one poll (several arrived between two passes).

use std::time::{Duration, Instant};

pub const REPORT_EVERY: Duration = Duration::from_secs(5);

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
    last_logic: Option<Instant>,
    frames: u64,
    age: Stat,
    track: Stat,
    moves: Stat,
    upload: Stat,
    ui: Stat,
    gap: Stat,
}

impl Perf {
    pub fn from_env() -> Option<Self> {
        let on = std::env::var("OCM_PERF").is_ok_and(|v| v == "1");
        on.then(|| {
            eprintln!(
                "OCM_PERF: every {} s, ms p50/p95/max (see perf.rs)",
                REPORT_EVERY.as_secs()
            );
            Self {
                since: Instant::now(),
                last_logic: None,
                frames: 0,
                age: Stat::default(),
                track: Stat::default(),
                moves: Stat::default(),
                upload: Stat::default(),
                ui: Stat::default(),
                gap: Stat::default(),
            }
        })
    }

    pub fn frame(&mut self, age_us: u64, track: Duration, moves: Duration) {
        self.frames += 1;
        self.age.add(age_us);
        self.track.add(us(track));
        self.moves.add(us(moves));
    }

    pub fn upload(&mut self, d: Duration) {
        self.upload.add(us(d));
    }

    pub fn ui(&mut self, d: Duration) {
        self.ui.add(us(d));
    }

    /// Call at the start of every `logic`; prints the report when due. `dropped` and
    /// `coalesced` are the camera link's running totals.
    pub fn logic(&mut self, dropped: u64, coalesced: u64) {
        let now = Instant::now();
        if let Some(prev) = self.last_logic.replace(now) {
            self.gap.add(us(now - prev));
        }
        let span = now - self.since;
        if span < REPORT_EVERY {
            return;
        }
        eprintln!(
            "perf {:.1} fps | dropped {dropped} coalesced {coalesced} | {} | {} | {} | {} | {} | {}",
            self.frames as f64 / span.as_secs_f64(),
            self.age.line("age"),
            self.track.line("track"),
            self.moves.line("move"),
            self.upload.line("upload"),
            self.ui.line("ui"),
            self.gap.line("gap"),
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
