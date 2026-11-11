//! B-NCC: fixed-template zero-mean normalised cross-correlation, legacy preset
//! (`legacy/internal/tracking/tracking.go`, BEHAVIOR_SPEC §3.4).
//!
//! Score at offset `(u, v)` is OpenCV's `TM_CCOEFF_NORMED`:
//! `Σ T'·I' / sqrt(Σ T'² · Σ I'²)` with `T' = T − mean(T)` and `I'` the zero-mean patch.
//! Because `Σ T' = 0`, `Σ T'·I' = Σ T'·I`; patch sums come from integral images. A flat
//! template or patch scores 0 (no NaN).

use crate::{
    Diagnostics, GrayFrame, LossReason, Point2, TrackError, TrackOutput, TrackStatus, Tracker,
    check_selection,
};

/// Legacy preset defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NccParams {
    /// Square template side in pixels (legacy default 45).
    pub template_size: u32,
    /// Search half-extent = `template_size · search_margin_mult` around the last centre.
    pub search_margin_mult: u32,
    /// Minimum best score for a match (legacy 0.68).
    pub threshold: f32,
}

impl Default for NccParams {
    fn default() -> Self {
        Self {
            template_size: 45,
            search_margin_mult: 2,
            threshold: 0.68,
        }
    }
}

/// The legacy tracker. The template is cut once from the selection frame and never updated.
/// A frame whose best score is below threshold reports `Lost(LowConfidence)` at the last good
/// centre, and the tracker **keeps** the template and retries next frame (legacy behaviour).
/// Output positions are integer pixel centres.
#[derive(Debug, Clone, Default)]
pub struct NccTracker {
    params: NccParams,
    template: Vec<f32>, // zero-mean
    t_norm2: f64,
    tw: usize,
    th: usize,
    center: Option<(i64, i64)>,
    size: (u32, u32),
    integ: Vec<f64>,
    integ2: Vec<f64>,
}

/// Legacy `clampCenter`: keep the whole template inside `[0, dim)`, or the frame centre if it
/// can't fit.
fn clamp_center(v: i64, half: i64, dim: i64) -> i64 {
    let (lo, hi) = (half, dim - half - 1);
    if hi < lo { dim / 2 } else { v.clamp(lo, hi) }
}

impl NccTracker {
    pub fn new(params: NccParams) -> Self {
        Self {
            params,
            ..Self::default()
        }
    }

    fn lost(&self, reason: LossReason) -> TrackOutput {
        let position = self
            .center
            .map(|(x, y)| Point2::new(x as f32, y as f32))
            .unwrap_or_default();
        TrackOutput {
            position,
            status: TrackStatus::Lost(reason),
            diagnostics: Diagnostics::default(),
        }
    }
}

impl Tracker for NccTracker {
    fn select(&mut self, frame: &GrayFrame, point: Point2) -> Result<(), TrackError> {
        check_selection(frame, point)?;
        let (w, h) = (i64::from(frame.width()), i64::from(frame.height()));
        let size = i64::from(self.params.template_size.max(1));
        let half = size / 2;
        let cx = clamp_center(point.x.round() as i64, half, w);
        let cy = clamp_center(point.y.round() as i64, half, h);
        // Legacy crops the template to the frame when it doesn't fit.
        let (x0, y0) = ((cx - half).max(0), (cy - half).max(0));
        let (x1, y1) = ((cx - half + size).min(w), (cy - half + size).min(h));
        self.tw = (x1 - x0) as usize;
        self.th = (y1 - y0) as usize;
        let bytes = frame.as_bytes();
        self.template.clear();
        for y in y0..y1 {
            let row = (y * w) as usize;
            self.template.extend(
                bytes[row + x0 as usize..row + x1 as usize]
                    .iter()
                    .map(|&b| f32::from(b)),
            );
        }
        let mean =
            self.template.iter().map(|&v| f64::from(v)).sum::<f64>() / self.template.len() as f64;
        for v in &mut self.template {
            *v -= mean as f32;
        }
        self.t_norm2 = self.template.iter().map(|&v| f64::from(v).powi(2)).sum();
        self.center = Some((cx, cy));
        self.size = (frame.width(), frame.height());
        Ok(())
    }

    fn step(&mut self, frame: &GrayFrame) -> Result<TrackOutput, TrackError> {
        let Some((cx, cy)) = self.center else {
            return Ok(self.lost(LossReason::NoSelection));
        };
        let size = (frame.width(), frame.height());
        if size != self.size {
            return Err(TrackError::FrameSizeChanged {
                from: self.size,
                to: size,
            });
        }
        let (w, h) = (i64::from(size.0), i64::from(size.1));
        let m = i64::from(self.params.template_size * self.params.search_margin_mult);
        let (sx0, sy0) = ((cx - m).clamp(0, w), (cy - m).clamp(0, h));
        let (sx1, sy1) = ((cx + m).clamp(0, w), (cy + m).clamp(0, h));
        let (sw, sh) = ((sx1 - sx0) as usize, (sy1 - sy0) as usize);
        let (tw, th) = (self.tw, self.th);
        if sw < tw || sh < th {
            return Ok(self.lost(LossReason::OutOfFrame));
        }

        // Integral images of the search region, (sw+1) × (sh+1), f64 for exact-enough sums.
        let iw = sw + 1;
        self.integ.clear();
        self.integ.resize(iw * (sh + 1), 0.0);
        self.integ2.clear();
        self.integ2.resize(iw * (sh + 1), 0.0);
        let bytes = frame.as_bytes();
        for y in 0..sh {
            let src = &bytes[((sy0 as usize + y) * w as usize + sx0 as usize)..][..sw];
            let (mut row, mut row2) = (0.0f64, 0.0f64);
            for (x, &b) in src.iter().enumerate() {
                let v = f64::from(b);
                row += v;
                row2 += v * v;
                self.integ[(y + 1) * iw + x + 1] = self.integ[y * iw + x + 1] + row;
                self.integ2[(y + 1) * iw + x + 1] = self.integ2[y * iw + x + 1] + row2;
            }
        }
        let rect = |t: &[f64], x: usize, y: usize| {
            t[(y + th) * iw + x + tw] - t[y * iw + x + tw] - t[(y + th) * iw + x] + t[y * iw + x]
        };

        let n = (tw * th) as f64;
        let (mut best, mut best_at) = (f32::NEG_INFINITY, (0usize, 0usize));
        for v in 0..=(sh - th) {
            for u in 0..=(sw - tw) {
                let s = rect(&self.integ, u, v);
                let var = rect(&self.integ2, u, v) - s * s / n;
                let denom = var * self.t_norm2;
                let score = if denom > 1e-6 {
                    let mut cross = 0.0f32;
                    for ty in 0..th {
                        let off = (sy0 as usize + v + ty) * w as usize + sx0 as usize + u;
                        let img = &bytes[off..off + tw];
                        let tpl = &self.template[ty * tw..(ty + 1) * tw];
                        for (a, &b) in tpl.iter().zip(img) {
                            cross += a * f32::from(b);
                        }
                    }
                    (f64::from(cross) / denom.sqrt()) as f32
                } else {
                    0.0
                };
                // Strict `>`: first maximum in row-major order, like OpenCV minMaxLoc.
                if score > best {
                    best = score;
                    best_at = (u, v);
                }
            }
        }

        let diagnostics = Diagnostics {
            score: Some(best),
            ..Diagnostics::default()
        };
        if best < self.params.threshold {
            let mut out = self.lost(LossReason::LowConfidence);
            out.diagnostics = diagnostics;
            return Ok(out);
        }
        let ncx = sx0 + best_at.0 as i64 + (tw / 2) as i64;
        let ncy = sy0 + best_at.1 as i64 + (th / 2) as i64;
        self.center = Some((ncx, ncy));
        Ok(TrackOutput {
            position: Point2::new(ncx as f32, ncy as f32),
            status: TrackStatus::Tracking,
            diagnostics,
        })
    }

    fn reset(&mut self) {
        self.center = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textured(w: u32, h: u32, sx: i64, sy: i64) -> GrayFrame {
        let mut d = Vec::with_capacity((w * h) as usize);
        for y in 0..h as i64 {
            for x in 0..w as i64 {
                let (u, v) = ((x - sx) as u64, (y - sy) as u64);
                // Integer hash texture: shifts exactly with (sx, sy).
                let mut z =
                    u.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ v.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
                z ^= z >> 29;
                d.push((z & 0xFF) as u8);
            }
        }
        GrayFrame::from_vec(w, h, d).unwrap()
    }

    #[test]
    fn finds_integer_translation_with_perfect_score() {
        let mut t = NccTracker::default();
        t.select(&textured(320, 240, 0, 0), Point2::new(160.0, 120.0))
            .unwrap();
        let out = t.step(&textured(320, 240, 7, -4)).unwrap();
        assert_eq!(out.status, TrackStatus::Tracking);
        assert_eq!(out.position, Point2::new(167.0, 116.0));
        assert!((out.diagnostics.score.unwrap() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn low_score_is_lost_but_template_is_kept() {
        let mut t = NccTracker::default();
        let base = textured(200, 200, 0, 0);
        t.select(&base, Point2::new(100.0, 100.0)).unwrap();
        let out = t.step(&GrayFrame::filled(200, 200, 50).unwrap()).unwrap();
        assert_eq!(out.status, TrackStatus::Lost(LossReason::LowConfidence));
        assert_eq!(out.position, Point2::new(100.0, 100.0));
        // Retries with the same template and recovers.
        assert_eq!(t.step(&base).unwrap().status, TrackStatus::Tracking);
    }

    #[test]
    fn selection_is_clamped_so_template_fits() {
        let mut t = NccTracker::default();
        t.select(&textured(100, 100, 0, 0), Point2::new(2.0, 99.0))
            .unwrap();
        assert_eq!(t.center, Some((22, 77)));
    }
}
