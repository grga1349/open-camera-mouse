//! 1€ filter (Casiez, Roussel, Vogel, CHI 2012) for the tracked point, in camera space before
//! mapping (ADR-0017, test drive #5).
//!
//! A first-order low-pass whose cutoff rises with speed: `cutoff = min_cutoff + beta·|v̂|`,
//! with `v̂` the low-passed speed (cutoff `d_cutoff`). At rest the cutoff is `min_cutoff` and
//! tracker noise / head tremor is averaged away; while the head moves the cutoff is high and
//! the lag small. Speed is the 2D magnitude so both axes share one cutoff (a diagonal move
//! isn't filtered differently from a straight one). Smoothing factors come from the real
//! `dt`, so the response is the same at 15 or 30 fps.

use std::f32::consts::TAU;

use tracking_core::Point2;

/// Parameters in camera pixels (canonical 640-wide frame) and Hz.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OneEuroParams {
    /// Cutoff at rest (Hz). Lower = steadier, slower to settle.
    pub min_cutoff: f32,
    /// Cutoff increase per camera px/s of speed.
    pub beta: f32,
    /// Cutoff of the speed estimate (Hz).
    pub d_cutoff: f32,
}

impl Default for OneEuroParams {
    /// Chosen on the test-drive failure points (ADR-0017): at rest ≈ 0.08 px rms tremor must
    /// not move the cursor more than 1 screen px per frame at ≈ 24 screen px per camera px,
    /// and a head turn must settle within the existing controller lag tests.
    fn default() -> Self {
        Self {
            min_cutoff: 1.0,
            beta: 0.15,
            d_cutoff: 4.0,
        }
    }
}

fn alpha(cutoff_hz: f32, dt_s: f32) -> f32 {
    let tau = 1.0 / (TAU * cutoff_hz.max(1e-3));
    1.0 / (1.0 + tau / dt_s)
}

#[derive(Debug, Clone, Default)]
pub struct OneEuro {
    /// Last filtered position, last raw position, filtered speed (px/s), time.
    state: Option<(Point2, Point2, f32, u64)>,
}

impl OneEuro {
    /// Forgets the history: the next sample passes through unchanged.
    pub fn reset(&mut self) {
        self.state = None;
    }

    /// The last filtered position, if any sample arrived since the last reset.
    pub fn last(&self) -> Option<Point2> {
        self.state.map(|(prev, ..)| prev)
    }

    pub fn update(&mut self, p: Point2, now_us: u64, params: &OneEuroParams) -> Point2 {
        let Some((prev, prev_raw, speed, t)) = self.state else {
            self.state = Some((p, p, 0.0, now_us));
            return p;
        };
        if now_us <= t {
            // Same timestamp (or clock went back): nothing to integrate over.
            return prev;
        }
        let dt = (now_us - t) as f32 / 1e6;
        let v = (p.x - prev_raw.x).hypot(p.y - prev_raw.y) / dt;
        let speed = speed + alpha(params.d_cutoff, dt) * (v - speed);
        let a = alpha(params.min_cutoff + params.beta * speed, dt);
        let out = Point2::new(prev.x + a * (p.x - prev.x), prev.y + a * (p.y - prev.y));
        self.state = Some((out, p, speed, now_us));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME_US: u64 = 33_333;

    /// Deterministic uniform noise in [-0.5, 0.5).
    fn noise(i: u64, seed: u64) -> f32 {
        let mut z =
            i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ seed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z ^= z >> 31;
        z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 29;
        (z >> 40) as f32 / (1u64 << 24) as f32 - 0.5
    }

    fn run(path: impl Fn(u64) -> Point2, frames: u64, dt: u64) -> Vec<Point2> {
        let mut f = OneEuro::default();
        let p = OneEuroParams::default();
        (0..frames).map(|i| f.update(path(i), i * dt, &p)).collect()
    }

    #[test]
    fn first_sample_and_constant_input_pass_through_exactly() {
        let c = Point2::new(320.0, 240.0);
        let out = run(|_| c, 30, FRAME_US);
        assert!(out.iter().all(|&o| o == c));
    }

    #[test]
    fn rest_tremor_is_reduced_at_least_four_times() {
        let raw = |i| Point2::new(320.0 + 0.28 * noise(i, 1), 240.0 + 0.28 * noise(i, 2));
        let out = run(raw, 300, FRAME_US);
        let step_rms = |v: &[Point2]| {
            let s: Vec<f32> = v[30..]
                .windows(2)
                .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y))
                .collect();
            (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
        };
        let raw_v: Vec<Point2> = (0..300).map(raw).collect();
        let (a, b) = (step_rms(&raw_v), step_rms(&out));
        assert!(b * 4.0 < a, "frame-to-frame rms {a:.4} → {b:.4}");
    }

    #[test]
    fn a_moving_head_is_followed_with_little_lag() {
        // 60 camera px/s (a moderate head turn): the filtered point trails by less than one
        // frame of motion (2 px = 33 ms).
        let out = run(|i| Point2::new(320.0 + 2.0 * i as f32, 240.0), 60, FRAME_US);
        let last = out.last().unwrap();
        let lag = 320.0 + 2.0 * 59.0 - last.x;
        assert!((0.0..2.0).contains(&lag), "lag {lag:.2} px");
    }

    #[test]
    fn a_step_settles_without_overshoot() {
        let out = run(
            |i| Point2::new(if i < 10 { 320.0 } else { 330.0 }, 240.0),
            60,
            FRAME_US,
        );
        assert!(out.windows(2).all(|w| w[1].x >= w[0].x), "monotonic");
        assert!(out.iter().all(|o| o.x <= 330.0), "no overshoot");
        assert!(out.last().unwrap().x > 329.5, "{:?}", out.last());
    }

    #[test]
    fn the_response_is_the_same_at_15_and_30_fps() {
        // The same 1 s ramp + hold sampled at 30 and 15 fps ends in the same place.
        let path = |t_s: f32| Point2::new(320.0 + 40.0 * t_s.min(1.0), 240.0);
        let at = |fps: u64| {
            let dt = 1_000_000 / fps;
            let mut f = OneEuro::default();
            let p = OneEuroParams::default();
            let mut out = Point2::new(0.0, 0.0);
            for i in 0..=(fps * 2) {
                out = f.update(path((i * dt) as f32 / 1e6), i * dt, &p);
            }
            out.x
        };
        let (a, b) = (at(30), at(15));
        assert!((a - b).abs() < 0.3, "30 fps {a:.2} vs 15 fps {b:.2}");
    }

    #[test]
    fn reset_and_repeated_timestamps() {
        let p = OneEuroParams::default();
        let mut f = OneEuro::default();
        f.update(Point2::new(0.0, 0.0), 0, &p);
        let a = f.update(Point2::new(10.0, 0.0), FRAME_US, &p);
        assert_eq!(
            f.update(Point2::new(50.0, 0.0), FRAME_US, &p),
            a,
            "no dt, no change"
        );
        f.reset();
        let jump = Point2::new(100.0, 100.0);
        assert_eq!(
            f.update(jump, 2 * FRAME_US, &p),
            jump,
            "after reset: pass through"
        );
    }
}
