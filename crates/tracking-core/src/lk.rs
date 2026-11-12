//! B-LK18: one-point pyramidal Lucas–Kanade (Bouguet formulation), 2018 preset.
//!
//! Math, per level `L` from coarsest to finest, with `u_L = u / 2^L` and guess `g_L`:
//! - window `W` = `(2·hw + 1)²` offsets around `u_L`, sampled bilinearly in the previous
//!   level image `I` and its gradients `Ix, Iy`;
//! - spatial matrix `G = Σ [Ix² IxIy; IxIy Iy²]`; minimum eigenvalue `λ_min(G) / |W|` is the
//!   texture test (`min_eigen`);
//! - iterate `b = Σ (I(u_L + w) − J(u_L + g + v + w)) · [Ix Iy]ᵀ`, `η = G⁻¹ b`, `v += η`
//!   until `|η| < epsilon` or `max_iter`;
//! - `g_{L−1} = 2 (g_L + v)`. Final displacement is `g_0 + v` at level 0.
//!
//! A degenerate window at a coarse level skips that level's refinement and keeps the guess
//! (as OpenCV does); at level 0 it is a failure. No NaN/Inf leaves this module.

use crate::image::GradPatch;
use crate::pyramid::Pyramid;
use crate::{
    Diagnostics, GrayFrame, HoldReason, LossReason, Point2, TrackError, TrackOutput, TrackStatus,
    Tracker, check_selection,
};

/// LK parameters. Defaults are the 2018 preset (BEHAVIOR_SPEC §2.4, ADR-0006, ADR-0009).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LkParams {
    /// Half window: the window is `(2·half_window + 1)²`. 2018 passed `win = 10×10` to the
    /// OpenCV 2.1 C API, which treats it as a half size (21×21 patch) — INFERRED, ADR-0009.
    pub half_window: usize,
    /// Coarsest pyramid level index (`3` → levels 0..=3).
    pub max_level: usize,
    pub max_iter: u32,
    /// Stop when the update step is shorter than this (level pixels).
    pub epsilon: f32,
    /// Minimum `λ_min(G) / |W|` (intensity² per px²) for a trackable window. Not a 2018 value
    /// (2018 only rejected a numerically singular G); chosen on the Phase 3 corpus.
    pub min_eigen: f32,
    /// Outlier guard as a fraction of frame width (Q-R1, ADR-0009). 35/640 = 35 px at 640.
    pub max_motion_frac: f32,
    /// Edge margin for the loss policy as a fraction of frame width. 10/640 = 10 px at 640.
    pub edge_margin_frac: f32,
    /// Forward-backward validation (Phase 4 E1): after tracking `a → b`, track `b` back to
    /// `a'` and reject the step when `|a − a'|` exceeds this (tracker px, absolute: it bounds
    /// solver precision, not motion). `None` = off = B-LK18.
    pub fb_max_error: Option<f32>,
    /// Appearance gate (Phase 4 E2): reject a step when the ZNCC between the level-0 window
    /// at the new position and the window sampled from the *selection* frame (R-T3) is below
    /// this. ZNCC is gain/bias invariant, so lighting changes pass; a different surface
    /// (occluder, background) does not. `None` = off = B-LK18.
    pub min_ref_zncc: Option<f32>,
}

impl Default for LkParams {
    fn default() -> Self {
        Self {
            half_window: 10,
            max_level: 3,
            max_iter: 20,
            epsilon: 0.03,
            min_eigen: 1.0,
            max_motion_frac: 35.0 / 640.0,
            edge_margin_frac: 10.0 / 640.0,
            fb_max_error: None,
            min_ref_zncc: None,
        }
    }
}

/// Raw solver outcome for one point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowResult {
    Found {
        position: Point2,
        diagnostics: Diagnostics,
    },
    /// No reliable estimate. `estimate` is the last iterate when one exists.
    Failed {
        reason: LossReason,
        estimate: Option<Point2>,
    },
}

/// Reusable per-point scratch (window samples). Allocated once per window size.
#[derive(Debug, Clone, Default)]
pub struct FlowScratch {
    i: Vec<f32>,
    ix: Vec<f32>,
    iy: Vec<f32>,
    grad: GradPatch,
}

/// Tracks one point from `prev` to `curr`. Both pyramids must come from same-sized frames.
pub fn track_point(
    prev: &Pyramid,
    curr: &Pyramid,
    p: Point2,
    params: &LkParams,
    scratch: &mut FlowScratch,
) -> FlowResult {
    let hw = params.half_window as isize;
    let n = (2 * params.half_window + 1).pow(2);
    scratch.i.resize(n, 0.0);
    scratch.ix.resize(n, 0.0);
    scratch.iy.resize(n, 0.0);
    let area = n as f32;
    let eps2 = params.epsilon * params.epsilon;

    let levels = prev.len().min(curr.len()).min(params.max_level + 1);
    let mut g = Point2::default();

    for l in (0..levels).rev() {
        let scale = 1.0 / (1u32 << l) as f32;
        let (pl, cl) = (prev.level(l), curr.level(l));
        let u = Point2::new(p.x * scale, p.y * scale);
        let (w, h) = (pl.img.width() as f32, pl.img.height() as f32);
        let outside = |q: Point2| {
            q.x < -(hw as f32)
                || q.y < -(hw as f32)
                || q.x > w - 1.0 + hw as f32
                || q.y > h - 1.0 + hw as f32
        };
        if outside(u) {
            if l == 0 {
                return FlowResult::Failed {
                    reason: LossReason::OutOfFrame,
                    estimate: None,
                };
            }
            g = Point2::new(2.0 * g.x, 2.0 * g.y);
            continue;
        }

        // Gradients for the window only, one pixel of slack for bilinear neighbours.
        let reach = hw as f32 + 1.0;
        scratch.grad.fill(
            &pl.img,
            (u.x - reach).floor() as isize,
            (u.x + reach).floor() as isize + 1,
            (u.y - reach).floor() as isize,
            (u.y + reach).floor() as isize + 1,
        );
        let (mut gxx, mut gxy, mut gyy) = (0.0f32, 0.0f32, 0.0f32);
        let mut k = 0;
        for dy in -hw..=hw {
            for dx in -hw..=hw {
                let (sx, sy) = (u.x + dx as f32, u.y + dy as f32);
                let (ix, iy) = scratch.grad.sample(sx, sy);
                scratch.i[k] = pl.img.sample(sx, sy);
                scratch.ix[k] = ix;
                scratch.iy[k] = iy;
                gxx += ix * ix;
                gxy += ix * iy;
                gyy += iy * iy;
                k += 1;
            }
        }
        let det = gxx * gyy - gxy * gxy;
        let min_eig = (gxx + gyy - ((gxx - gyy).powi(2) + 4.0 * gxy * gxy).sqrt()) / (2.0 * area);
        // Written so NaN (from a non-finite window) also counts as degenerate.
        let trackable = min_eig >= params.min_eigen && det > f32::EPSILON;
        if !trackable {
            if l == 0 {
                return FlowResult::Failed {
                    reason: LossReason::LowConfidence,
                    estimate: None,
                };
            }
            g = Point2::new(2.0 * g.x, 2.0 * g.y);
            continue;
        }
        let inv = 1.0 / det;

        let mut v = Point2::default();
        let mut iters = 0;
        while iters < params.max_iter {
            iters += 1;
            let q = Point2::new(u.x + g.x + v.x, u.y + g.y + v.y);
            if outside(q) {
                if l == 0 {
                    return FlowResult::Failed {
                        reason: LossReason::OutOfFrame,
                        estimate: Some(Point2::new(p.x + g.x + v.x, p.y + g.y + v.y)),
                    };
                }
                break;
            }
            let (mut bx, mut by) = (0.0f32, 0.0f32);
            let mut k = 0;
            for dy in -hw..=hw {
                for dx in -hw..=hw {
                    let j = cl.img.sample(q.x + dx as f32, q.y + dy as f32);
                    let diff = scratch.i[k] - j;
                    bx += diff * scratch.ix[k];
                    by += diff * scratch.iy[k];
                    k += 1;
                }
            }
            let eta = Point2::new(inv * (gyy * bx - gxy * by), inv * (gxx * by - gxy * bx));
            v.x += eta.x;
            v.y += eta.y;
            if eta.x * eta.x + eta.y * eta.y < eps2 {
                break;
            }
        }

        if l == 0 {
            let iterations = iters;
            let est = Point2::new(p.x + g.x + v.x, p.y + g.y + v.y);
            if !est.x.is_finite() || !est.y.is_finite() {
                return FlowResult::Failed {
                    reason: LossReason::LowConfidence,
                    estimate: None,
                };
            }
            if !pl.img.contains(est.x, est.y) {
                return FlowResult::Failed {
                    reason: LossReason::OutOfFrame,
                    estimate: Some(est),
                };
            }
            let mut sad = 0.0;
            let mut k = 0;
            for dy in -hw..=hw {
                for dx in -hw..=hw {
                    sad +=
                        (scratch.i[k] - cl.img.sample(est.x + dx as f32, est.y + dy as f32)).abs();
                    k += 1;
                }
            }
            return FlowResult::Found {
                position: est,
                diagnostics: Diagnostics {
                    residual: Some(sad / area),
                    min_eigen: Some(min_eig),
                    iterations: Some(iterations),
                    score: None,
                    fb_error: None,
                },
            };
        }
        g = Point2::new(2.0 * (g.x + v.x), 2.0 * (g.y + v.y));
    }
    unreachable!("level 0 always returns")
}

/// Symmetric "near an edge and moving outward" predicate (R-T4; fixes the 2018 x-axis swap).
pub fn near_edge_moving_out(a: Point2, b: Point2, width: f32, height: f32, margin: f32) -> bool {
    (a.x <= margin && b.x < a.x)
        || (a.x >= width - 1.0 - margin && b.x > a.x)
        || (a.y <= margin && b.y < a.y)
        || (a.y >= height - 1.0 - margin && b.y > a.y)
}

/// The 2018 tracker: LK + loss policy + outlier guard.
///
/// Statuses: `Tracking`; `Held(MotionOutlier)` when the step exceeds the motion bound (point
/// kept); `Held(NearEdgeMovingOut)` when LK fails (or, with `fb_max_error` set, fails the
/// forward-backward check or, with `min_ref_zncc` set, the appearance gate) while leaving through an edge (point kept, 2018 default);
/// `Lost(_)` otherwise — the reference is dropped and the next steps report
/// `Lost(NoSelection)` until `select` is called again (2018 invalidates the track point).
#[derive(Debug, Clone, Default)]
pub struct LkTracker {
    params: LkParams,
    prev: Pyramid,
    curr: Pyramid,
    scratch: FlowScratch,
    point: Option<Point2>,
    size: (u32, u32),
    /// Zero-mean, unit-norm level-0 window from the selection frame (only with `min_ref_zncc`).
    reference: Vec<f32>,
}

/// Fills `out` with the `(2·hw + 1)²` bilinear window around `c`, zero-mean and unit-norm.
/// Returns `false` (and leaves `out` unspecified) when the window is flat or non-finite.
fn normalized_window(
    img: &crate::image::ImageF32,
    c: Point2,
    hw: usize,
    out: &mut Vec<f32>,
) -> bool {
    let hw = hw as isize;
    out.clear();
    for dy in -hw..=hw {
        for dx in -hw..=hw {
            out.push(img.sample(c.x + dx as f32, c.y + dy as f32));
        }
    }
    let mean = out.iter().sum::<f32>() / out.len() as f32;
    let mut ss = 0.0f32;
    for v in out.iter_mut() {
        *v -= mean;
        ss += *v * *v;
    }
    // Flat window: ZNCC is undefined. Also catches NaN.
    if ss.is_nan() || ss <= 1e-6 * out.len() as f32 {
        return false;
    }
    let inv = 1.0 / ss.sqrt();
    out.iter_mut().for_each(|v| *v *= inv);
    true
}

/// ZNCC in [−1, 1] between a [`normalized_window`] reference and the window around `c`.
/// A flat or non-finite current window scores 0. Allocation-free.
fn zncc_against(img: &crate::image::ImageF32, c: Point2, hw: usize, reference: &[f32]) -> f32 {
    let hw = hw as isize;
    let (mut s, mut s2, mut sr) = (0.0f32, 0.0f32, 0.0f32);
    let mut k = 0;
    for dy in -hw..=hw {
        for dx in -hw..=hw {
            // Offset by mid-grey to limit f32 cancellation in the one-pass variance.
            let v = img.sample(c.x + dx as f32, c.y + dy as f32) - 128.0;
            s += v;
            s2 += v * v;
            // The reference is zero-mean, so Σ r·(v − mean) = Σ r·v (offset included).
            sr += reference[k] * v;
            k += 1;
        }
    }
    let n = k as f32;
    let var = s2 - s * s / n;
    if var.is_nan() || var <= 1e-6 * n {
        return 0.0;
    }
    (sr / var.sqrt()).clamp(-1.0, 1.0)
}

impl LkTracker {
    pub fn new(params: LkParams) -> Self {
        Self {
            params,
            ..Self::default()
        }
    }

    pub fn params(&self) -> &LkParams {
        &self.params
    }

    /// Forward-backward check (E1). Call after the buffer swap: `prev` holds the current
    /// frame and `curr` the previous one. Records the error in `diagnostics`.
    fn fb_rejects(&mut self, a: Point2, b: Point2, diagnostics: &mut Diagnostics) -> bool {
        let Some(max) = self.params.fb_max_error else {
            return false;
        };
        let back = track_point(&self.prev, &self.curr, b, &self.params, &mut self.scratch);
        let err = match back {
            FlowResult::Found { position: a2, .. } => {
                ((a2.x - a.x).powi(2) + (a2.y - a.y).powi(2)).sqrt()
            }
            FlowResult::Failed { .. } => f32::INFINITY,
        };
        diagnostics.fb_error = Some(err);
        // NaN (non-finite window) also rejects.
        err.is_nan() || err > max
    }

    /// Appearance gate (E2). Call after the buffer swap (`prev` = current frame). Records the
    /// score in `diagnostics.score`.
    fn appearance_rejects(&self, b: Point2, diagnostics: &mut Diagnostics) -> bool {
        let Some(min) = self.params.min_ref_zncc else {
            return false;
        };
        // No usable reference (flat selection window): LK's own texture test governs.
        if self.reference.is_empty() {
            return false;
        }
        let z = zncc_against(
            &self.prev.level(0).img,
            b,
            self.params.half_window,
            &self.reference,
        );
        diagnostics.score = Some(z);
        // NaN also rejects.
        z.is_nan() || z < min
    }

    /// 2018 loss policy for a step without a usable estimate: near an edge and moving out →
    /// `Held` (point kept); otherwise `Lost` and the reference is dropped.
    fn reject(
        &mut self,
        a: Point2,
        b: Point2,
        w: f32,
        h: f32,
        reason: LossReason,
        diagnostics: Diagnostics,
    ) -> TrackOutput {
        let margin = self.params.edge_margin_frac * w;
        let status = if near_edge_moving_out(a, b, w, h, margin) {
            TrackStatus::Held(HoldReason::NearEdgeMovingOut)
        } else {
            self.point = None;
            TrackStatus::Lost(reason)
        };
        TrackOutput {
            position: a,
            status,
            diagnostics,
        }
    }
}

impl Tracker for LkTracker {
    fn select(&mut self, frame: &GrayFrame, point: Point2) -> Result<(), TrackError> {
        check_selection(frame, point)?;
        // R-T3: the reference is the selection frame itself.
        self.prev.build(frame, self.params.max_level);
        self.point = Some(point);
        self.size = (frame.width(), frame.height());
        if self.params.min_ref_zncc.is_some()
            && !normalized_window(
                &self.prev.level(0).img,
                point,
                self.params.half_window,
                &mut self.reference,
            )
        {
            self.reference.clear();
        }
        Ok(())
    }

    fn step(&mut self, frame: &GrayFrame) -> Result<TrackOutput, TrackError> {
        let Some(a) = self.point else {
            return Ok(TrackOutput {
                position: Point2::default(),
                status: TrackStatus::Lost(LossReason::NoSelection),
                diagnostics: Diagnostics::default(),
            });
        };
        let size = (frame.width(), frame.height());
        if size != self.size {
            return Err(TrackError::FrameSizeChanged {
                from: self.size,
                to: size,
            });
        }
        self.curr.build(frame, self.params.max_level);
        let (w, h) = (size.0 as f32, size.1 as f32);
        let result = track_point(&self.prev, &self.curr, a, &self.params, &mut self.scratch);
        // 2018 step 8: buffers swap every frame, whatever the outcome.
        std::mem::swap(&mut self.prev, &mut self.curr);

        let out = match result {
            FlowResult::Found {
                position: b,
                mut diagnostics,
            } => {
                let motion = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
                if motion > self.params.max_motion_frac * w {
                    TrackOutput {
                        position: a,
                        status: TrackStatus::Held(HoldReason::MotionOutlier),
                        diagnostics,
                    }
                } else if self.fb_rejects(a, b, &mut diagnostics) {
                    self.reject(a, b, w, h, LossReason::Inconsistent, diagnostics)
                } else if self.appearance_rejects(b, &mut diagnostics) {
                    self.reject(a, b, w, h, LossReason::AppearanceChanged, diagnostics)
                } else {
                    self.point = Some(b);
                    TrackOutput {
                        position: b,
                        status: TrackStatus::Tracking,
                        diagnostics,
                    }
                }
            }
            FlowResult::Failed { reason, estimate } => {
                let b = estimate.unwrap_or(a);
                self.reject(a, b, w, h, reason, Diagnostics::default())
            }
        };
        Ok(out)
    }

    fn reset(&mut self) {
        self.point = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smooth, well-textured analytic pattern (not the fixture generator; unit-test only).
    fn pattern(x: f32, y: f32) -> f32 {
        128.0
            + 25.0 * (x * 0.21).sin() * (y * 0.17).cos()
            + 30.0 * ((x + 2.0 * y) * 0.031).sin()
            + 30.0 * ((2.0 * x - y) * 0.023).cos()
    }

    fn frame(w: u32, h: u32, shift: Point2) -> GrayFrame {
        let mut data = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                let v = pattern(x as f32 - shift.x, y as f32 - shift.y);
                data.push(v.round().clamp(0.0, 255.0) as u8);
            }
        }
        GrayFrame::from_vec(w, h, data).unwrap()
    }

    #[test]
    fn recovers_integer_and_subpixel_translation() {
        for shift in [
            Point2::new(5.0, -3.0),
            Point2::new(1.4, 0.7),
            Point2::new(-12.0, 9.0),
        ] {
            let mut t = LkTracker::new(LkParams::default());
            let start = Point2::new(160.0, 120.0);
            t.select(&frame(320, 240, Point2::default()), start)
                .unwrap();
            let out = t.step(&frame(320, 240, shift)).unwrap();
            assert_eq!(out.status, TrackStatus::Tracking, "{shift:?}");
            let err = ((out.position.x - start.x - shift.x).powi(2)
                + (out.position.y - start.y - shift.y).powi(2))
            .sqrt();
            assert!(err < 0.1, "shift {shift:?}: error {err}");
        }
    }

    #[test]
    fn flat_region_is_lost_not_tracked() {
        let flat = GrayFrame::filled(320, 240, 90).unwrap();
        let mut t = LkTracker::new(LkParams::default());
        t.select(&flat, Point2::new(100.0, 100.0)).unwrap();
        let out = t.step(&flat).unwrap();
        assert_eq!(out.status, TrackStatus::Lost(LossReason::LowConfidence));
        assert!(out.position.x.is_finite());
        // Reference dropped (2018 invalidation).
        let next = t.step(&flat).unwrap();
        assert_eq!(next.status, TrackStatus::Lost(LossReason::NoSelection));
    }

    #[test]
    fn large_jump_is_held_by_outlier_guard() {
        let params = LkParams {
            max_motion_frac: 3.0 / 320.0, // 3 px at 320 wide
            ..LkParams::default()
        };
        let mut t = LkTracker::new(params);
        let start = Point2::new(160.0, 120.0);
        t.select(&frame(320, 240, Point2::default()), start)
            .unwrap();
        let out = t.step(&frame(320, 240, Point2::new(6.0, 0.0))).unwrap();
        assert_eq!(out.status, TrackStatus::Held(HoldReason::MotionOutlier));
        assert_eq!(out.position, start);
    }

    #[test]
    fn edge_predicate_is_symmetric_on_all_four_edges() {
        let (w, h, m) = (100.0, 80.0, 5.0);
        let p = Point2::new;
        assert!(near_edge_moving_out(p(3.0, 40.0), p(1.0, 40.0), w, h, m)); // left, out
        assert!(!near_edge_moving_out(p(3.0, 40.0), p(4.0, 40.0), w, h, m)); // left, in
        assert!(near_edge_moving_out(p(96.0, 40.0), p(98.0, 40.0), w, h, m)); // right, out
        assert!(!near_edge_moving_out(p(96.0, 40.0), p(95.0, 40.0), w, h, m)); // right, in
        assert!(near_edge_moving_out(p(50.0, 2.0), p(50.0, 0.5), w, h, m)); // top, out
        assert!(!near_edge_moving_out(p(50.0, 2.0), p(50.0, 3.0), w, h, m)); // top, in
        assert!(near_edge_moving_out(p(50.0, 77.0), p(50.0, 78.5), w, h, m)); // bottom, out
        assert!(!near_edge_moving_out(p(50.0, 77.0), p(50.0, 76.0), w, h, m)); // bottom, in
        // Mid-frame horizontal motion is *not* "near edge" (the 2018 x-swap defect).
        assert!(!near_edge_moving_out(p(50.0, 40.0), p(45.0, 40.0), w, h, m));
        assert!(!near_edge_moving_out(p(50.0, 40.0), p(55.0, 40.0), w, h, m));
    }

    #[test]
    fn forward_backward_off_by_default_and_passes_clean_translation() {
        assert_eq!(LkParams::default().fb_max_error, None);
        let params = LkParams {
            fb_max_error: Some(0.5),
            ..LkParams::default()
        };
        let mut t = LkTracker::new(params);
        t.select(
            &frame(320, 240, Point2::default()),
            Point2::new(160.0, 120.0),
        )
        .unwrap();
        let out = t.step(&frame(320, 240, Point2::new(3.3, -1.7))).unwrap();
        assert_eq!(out.status, TrackStatus::Tracking);
        let fb = out.diagnostics.fb_error.expect("validation ran");
        assert!(fb < 0.1, "fb error {fb}");
        // Off: no backward pass, no error recorded.
        let mut t = LkTracker::default();
        t.select(
            &frame(320, 240, Point2::default()),
            Point2::new(160.0, 120.0),
        )
        .unwrap();
        let out = t.step(&frame(320, 240, Point2::new(3.3, -1.7))).unwrap();
        assert_eq!(out.diagnostics.fb_error, None);
    }

    #[test]
    fn forward_backward_rejects_a_texture_swap() {
        // The window's content is replaced by an unrelated texture (a full occlusion onset).
        let other = {
            let (w, h) = (320u32, 240u32);
            let mut data = Vec::with_capacity((w * h) as usize);
            for y in 0..h {
                for x in 0..w {
                    let (x, y) = (x as f32, y as f32);
                    let v = 128.0 + 60.0 * (x * 0.13 + 1.0).cos() * (y * 0.29 + 2.0).sin()
                        - 35.0 * ((x - 3.0 * y) * 0.05).sin();
                    data.push(v.round().clamp(0.0, 255.0) as u8);
                }
            }
            GrayFrame::from_vec(w, h, data).unwrap()
        };
        let params = LkParams {
            fb_max_error: Some(0.5),
            max_motion_frac: 1.0, // guard off: isolate the FB check
            ..LkParams::default()
        };
        let mut t = LkTracker::new(params);
        t.select(
            &frame(320, 240, Point2::default()),
            Point2::new(160.0, 120.0),
        )
        .unwrap();
        let out = t.step(&other).unwrap();
        assert_eq!(
            out.status,
            TrackStatus::Lost(LossReason::Inconsistent),
            "{out:?}"
        );
        assert_eq!(out.position, Point2::new(160.0, 120.0));
        // 2018 policy: the reference is dropped.
        assert_eq!(
            t.step(&other).unwrap().status,
            TrackStatus::Lost(LossReason::NoSelection)
        );
    }

    #[test]
    fn appearance_gate_off_by_default_and_gain_invariant() {
        assert_eq!(LkParams::default().min_ref_zncc, None);
        let params = LkParams {
            min_ref_zncc: Some(0.9),
            ..LkParams::default()
        };
        let mut t = LkTracker::new(params);
        t.select(
            &frame(320, 240, Point2::default()),
            Point2::new(160.0, 120.0),
        )
        .unwrap();
        // Translated and contrast-halved around mid-grey: same surface, still tracked.
        let shifted = frame(320, 240, Point2::new(2.2, -1.1));
        let dim: Vec<u8> = shifted
            .as_bytes()
            .iter()
            .map(|&v| (0.5 * f32::from(v) + 64.0).round() as u8)
            .collect();
        let out = t
            .step(&GrayFrame::from_vec(320, 240, dim).unwrap())
            .unwrap();
        assert_eq!(out.status, TrackStatus::Tracking, "{out:?}");
        let z = out.diagnostics.score.expect("gate ran");
        // Not ~1: plain LK's position is biased by the gain change (TRACKING_BASELINE
        // finding 5) and halving doubles 8-bit quantisation. Undimmed it scores > 0.97.
        assert!(z > 0.9, "zncc {z}");
        // Off: no score recorded.
        let mut t = LkTracker::default();
        t.select(
            &frame(320, 240, Point2::default()),
            Point2::new(160.0, 120.0),
        )
        .unwrap();
        let out = t.step(&shifted).unwrap();
        assert_eq!(out.diagnostics.score, None);
    }

    #[test]
    fn zncc_is_one_on_itself_and_zero_on_flat() {
        let f = frame(64, 64, Point2::default());
        let mut img = crate::image::ImageF32::default();
        crate::image::gray_to_f32_into(&f, &mut img);
        let mut r = Vec::new();
        assert!(normalized_window(&img, Point2::new(30.5, 31.25), 5, &mut r));
        let z = zncc_against(&img, Point2::new(30.5, 31.25), 5, &r);
        assert!((z - 1.0).abs() < 1e-4, "{z}");
        let mut flat = crate::image::ImageF32::default();
        crate::image::gray_to_f32_into(&GrayFrame::filled(64, 64, 77).unwrap(), &mut flat);
        assert_eq!(zncc_against(&flat, Point2::new(30.0, 30.0), 5, &r), 0.0);
        assert!(!normalized_window(
            &flat,
            Point2::new(30.0, 30.0),
            5,
            &mut r
        ));
    }

    #[test]
    fn step_without_selection_reports_no_selection() {
        let mut t = LkTracker::default();
        let out = t.step(&GrayFrame::filled(32, 32, 0).unwrap()).unwrap();
        assert_eq!(out.status, TrackStatus::Lost(LossReason::NoSelection));
    }

    #[test]
    fn frame_size_change_is_an_error() {
        let mut t = LkTracker::default();
        t.select(&frame(64, 64, Point2::default()), Point2::new(32.0, 32.0))
            .unwrap();
        assert!(matches!(
            t.step(&frame(65, 64, Point2::default())),
            Err(TrackError::FrameSizeChanged { .. })
        ));
    }

    #[test]
    fn deterministic_across_runs() {
        let run = || {
            let mut t = LkTracker::default();
            t.select(
                &frame(320, 240, Point2::default()),
                Point2::new(150.0, 110.0),
            )
            .unwrap();
            (1..6)
                .map(|i| {
                    t.step(&frame(
                        320,
                        240,
                        Point2::new(i as f32 * 1.3, i as f32 * -0.6),
                    ))
                    .unwrap()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
    }
}
