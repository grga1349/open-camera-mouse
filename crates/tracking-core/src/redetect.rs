//! Re-detection of a lost target against its selection-frame template (Phase 4 E4, ADR-0010).
//!
//! Templates: for each pyramid level `l` in `0..=search_level`, the bilinear window of half
//! size `hw_l` (`hw` at level 0, `max(1, hw >> l)` above) around `p / 2^l` in the *selection*
//! pyramid, zero-mean and unit-norm. Level 0 is sampled exactly like the E2 reference window.
//!
//! Search on a frame's pyramid:
//! 1. exhaustive integer ZNCC at the search level, over every position whose window fits;
//! 2. 3×3 local maxima, sorted by score, greedy suppression within `2·hw_L` → top `candidates`;
//! 3. each candidate refined level by level: `c ← 2c`, best of the ±2 px integer offsets;
//! 4. at level 0, a separable parabolic fit on the integer score gives the sub-pixel position.
//!
//! The best refined score `z1` must reach `min_zncc`. The best score of any other refined
//! candidate farther than `hw` from it, `z2`, must leave a margin: `z1 − z2 ≥ min_margin`.
//! Allocation happens in [`Searcher::prepare`]; [`Searcher::search`] is allocation-free.

use crate::Point2;
use crate::image::ImageF32;
use crate::pyramid::Pyramid;

/// Re-detection parameters (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RedetectParams {
    /// Minimum level-0 ZNCC of the accepted match.
    pub min_zncc: f32,
    /// Minimum `z1 − z2` between the best and the best distinct other peak.
    pub min_margin: f32,
    /// Pyramid level of the exhaustive search (clamped to the levels built).
    pub search_level: usize,
    /// Coarse candidates refined to level 0.
    pub candidates: usize,
    /// Spatial prior: only accept matches within this radius (fraction of frame width,
    /// ADR-0009 units) of the last known position. `None` = whole frame. The selection-time
    /// self-test always searches the whole frame.
    pub max_radius_frac: Option<f32>,
}

impl Default for RedetectParams {
    fn default() -> Self {
        Self {
            min_zncc: 0.9,
            min_margin: 0.1,
            search_level: 2,
            candidates: 16,
            max_radius_frac: None,
        }
    }
}

/// Outcome of one search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchResult {
    /// Confident and unambiguous match.
    Found { position: Point2, score: f32 },
    /// No acceptable match. `best`/`second` are the scores seen (−1 when absent).
    NotFound { best: f32, second: f32 },
}

#[derive(Debug, Clone, Default)]
pub struct Searcher {
    /// Per level: (half window, zero-mean unit-norm template). Empty = not ready.
    templates: Vec<(usize, Vec<f32>)>,
    scores: Vec<f32>,
    maxima: Vec<(f32, u32, u32)>,
    picked: Vec<(u32, u32)>,
    refined: Vec<(f32, Point2)>,
}

impl Searcher {
    /// Builds the templates from the selection pyramid. Leaves the searcher not ready
    /// ([`Self::is_ready`]) when any level's window is flat. Sizes the scratch for the
    /// pyramid's search level.
    pub fn prepare(&mut self, pyr: &Pyramid, p: Point2, hw: usize, params: &RedetectParams) {
        self.templates.clear();
        let top = params.search_level.min(pyr.len() - 1);
        for l in 0..=top {
            let hw_l = if l == 0 { hw } else { (hw >> l).max(1) };
            let s = 1.0 / (1u32 << l) as f32;
            let mut t = Vec::new();
            if !crate::lk::normalized_window(
                &pyr.level(l).img,
                Point2::new(p.x * s, p.y * s),
                hw_l,
                &mut t,
            ) {
                self.templates.clear();
                return;
            }
            self.templates.push((hw_l, t));
        }
        let img = &pyr.level(top).img;
        self.scores
            .resize(img.width() * img.height(), f32::NEG_INFINITY);
        // 3×3 strict-ish local maxima are at most one per 2×2 block.
        self.maxima
            .reserve((img.width() * img.height()).div_ceil(4) + 1);
        self.picked.reserve(params.candidates);
        self.refined.reserve(params.candidates);
    }

    pub fn is_ready(&self) -> bool {
        !self.templates.is_empty()
    }

    pub fn clear(&mut self) {
        self.templates.clear();
    }

    /// Searches `pyr` (same frame size as the selection) for the template. With `region =
    /// Some((c, r))`, only positions within `r` level-0 px of `c` are scored or accepted, and
    /// ambiguity is judged among the peaks inside that disc.
    pub fn search(
        &mut self,
        pyr: &Pyramid,
        params: &RedetectParams,
        region: Option<(Point2, f32)>,
    ) -> SearchResult {
        let none = SearchResult::NotFound {
            best: -1.0,
            second: -1.0,
        };
        if !self.is_ready() {
            return none;
        }
        let top = (self.templates.len() - 1).min(pyr.len() - 1);
        let (hw_t, ref t) = self.templates[top];
        let img = &pyr.level(top).img;
        let (w, h) = (img.width(), img.height());
        if w < 2 * hw_t + 1 || h < 2 * hw_t + 1 {
            return none;
        }

        // Scan bounds at the search level: positions whose window fits, inside the region.
        let k = (1u32 << top) as f32;
        let (mut x0, mut x1, mut y0, mut y1) = (hw_t, w - hw_t, hw_t, h - hw_t);
        if let Some((c, r)) = region {
            let clamp = |v: f32, lo: usize, hi: usize| (v.max(lo as f32) as usize).clamp(lo, hi);
            x0 = clamp(((c.x - r) / k).floor(), x0, x1);
            x1 = clamp(((c.x + r) / k).ceil() + 1.0, x0, x1);
            y0 = clamp(((c.y - r) / k).floor(), y0, y1);
            y1 = clamp(((c.y + r) / k).ceil() + 1.0, y0, y1);
        }
        let inside = |q: Point2| region.is_none_or(|(c, r)| (q.x - c.x).hypot(q.y - c.y) <= r);

        // 1. Exhaustive coarse score map.
        self.scores.resize(w * h, f32::NEG_INFINITY);
        self.scores.fill(f32::NEG_INFINITY);
        for y in y0..y1 {
            for x in x0..x1 {
                if inside(Point2::new(x as f32 * k, y as f32 * k)) {
                    self.scores[y * w + x] = zncc_int(img, x, y, hw_t, t);
                }
            }
        }

        // 2. Local maxima, sorted, greedy suppression.
        self.maxima.clear();
        for y in y0..y1 {
            for x in x0..x1 {
                let s = self.scores[y * w + x];
                if s == f32::NEG_INFINITY {
                    continue;
                }
                let mut is_max = true;
                'n: for dy in -1isize..=1 {
                    for dx in -1isize..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let (nx, ny) = (x as isize + dx, y as isize + dy);
                        let o = self.scores[ny as usize * w + nx as usize];
                        // Ties: the first in scan order wins.
                        if o > s || (o == s && (dy < 0 || (dy == 0 && dx < 0))) {
                            is_max = false;
                            break 'n;
                        }
                    }
                }
                if is_max {
                    self.maxima.push((s, x as u32, y as u32));
                }
            }
        }
        // Deterministic order: score desc, then position.
        self.maxima
            .sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.2.cmp(&b.2)).then(a.1.cmp(&b.1)));
        self.picked.clear();
        let sep = (2 * hw_t) as i64;
        for &(_, x, y) in &self.maxima {
            if self.picked.len() >= params.candidates {
                break;
            }
            let far = self.picked.iter().all(|&(px, py)| {
                let (dx, dy) = (i64::from(x) - i64::from(px), i64::from(y) - i64::from(py));
                dx * dx + dy * dy > sep * sep
            });
            if far {
                self.picked.push((x, y));
            }
        }

        // 3–4. Refine each candidate to level 0.
        self.refined.clear();
        for i in 0..self.picked.len() {
            let (mut cx, mut cy) = (self.picked[i].0 as isize, self.picked[i].1 as isize);
            let mut best = f32::NEG_INFINITY;
            for l in (0..top).rev() {
                let (hw_l, ref tl) = self.templates[l];
                let li = &pyr.level(l).img;
                let (bx, by, bs) = best_near(li, 2 * cx, 2 * cy, 2, hw_l, tl);
                if bs == f32::NEG_INFINITY {
                    best = bs;
                    break;
                }
                (cx, cy, best) = (bx, by, bs);
            }
            if top == 0 {
                best = self.scores[cy as usize * w + cx as usize];
            }
            if !best.is_finite() {
                continue;
            }
            let (hw0, ref t0) = self.templates[0];
            let l0 = &pyr.level(0).img;
            let at = |x: isize, y: isize| score_at(l0, x, y, hw0, t0);
            let sub = |m: f32, c: f32, p: f32| {
                let den = m - 2.0 * c + p;
                if m.is_finite() && p.is_finite() && den < 0.0 {
                    (0.5 * (m - p) / den).clamp(-0.5, 0.5)
                } else {
                    0.0
                }
            };
            let dx = sub(at(cx - 1, cy), best, at(cx + 1, cy));
            let dy = sub(at(cx, cy - 1), best, at(cx, cy + 1));
            let q = Point2::new(cx as f32 + dx, cy as f32 + dy);
            if inside(q) {
                self.refined.push((best, q));
            }
        }
        let Some(&(z1, pos)) = self.refined.iter().max_by(|a, b| a.0.total_cmp(&b.0)) else {
            return none;
        };
        let hw0 = self.templates[0].0 as f32;
        let z2 = self
            .refined
            .iter()
            .filter(|(_, q)| (q.x - pos.x).hypot(q.y - pos.y) > hw0)
            .map(|r| r.0)
            .fold(-1.0f32, f32::max);
        if z1 >= params.min_zncc && z1 - z2 >= params.min_margin {
            SearchResult::Found {
                position: pos,
                score: z1,
            }
        } else {
            SearchResult::NotFound {
                best: z1,
                second: z2,
            }
        }
    }
}

/// ZNCC at integer `(x, y)` (window must fit), `t` zero-mean unit-norm. Flat window → 0.
fn zncc_int(img: &ImageF32, x: usize, y: usize, hw: usize, t: &[f32]) -> f32 {
    let w = img.width();
    let data = img.as_slice();
    let side = 2 * hw + 1;
    let (mut s, mut s2, mut sr) = (0.0f32, 0.0f32, 0.0f32);
    for r in 0..side {
        let row = &data[(y - hw + r) * w + x - hw..][..side];
        let tr = &t[r * side..][..side];
        for (&v, &tv) in row.iter().zip(tr) {
            let v = v - 128.0;
            s += v;
            s2 += v * v;
            sr += tv * v;
        }
    }
    let n = (side * side) as f32;
    let var = s2 - s * s / n;
    if var.is_nan() || var <= 1e-6 * n {
        return 0.0;
    }
    (sr / var.sqrt()).clamp(-1.0, 1.0)
}

/// [`zncc_int`] with bounds: −∞ when the window doesn't fit.
fn score_at(img: &ImageF32, x: isize, y: isize, hw: usize, t: &[f32]) -> f32 {
    let hw_i = hw as isize;
    if x < hw_i || y < hw_i || x + hw_i >= img.width() as isize || y + hw_i >= img.height() as isize
    {
        return f32::NEG_INFINITY;
    }
    zncc_int(img, x as usize, y as usize, hw, t)
}

/// Best integer position within `±r` of `(cx, cy)`. Scan order breaks ties.
fn best_near(
    img: &ImageF32,
    cx: isize,
    cy: isize,
    r: isize,
    hw: usize,
    t: &[f32],
) -> (isize, isize, f32) {
    let mut best = (cx, cy, f32::NEG_INFINITY);
    for dy in -r..=r {
        for dx in -r..=r {
            let s = score_at(img, cx + dx, cy + dy, hw, t);
            if s > best.2 {
                best = (cx + dx, cy + dy, s);
            }
        }
    }
    best
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::GrayFrame;

    fn pattern(x: f32, y: f32) -> f32 {
        128.0
            + 25.0 * (x * 0.21).sin() * (y * 0.17).cos()
            + 30.0 * ((x + 2.0 * y) * 0.031).sin()
            + 30.0 * ((2.0 * x - y) * 0.023).cos()
    }

    /// Non-periodic smooth texture: bilinear value noise on an 8 px lattice (hash-seeded).
    pub(crate) fn noise(x: f32, y: f32) -> f32 {
        let lat = |i: i64, j: i64| {
            let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ (j as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z ^= z >> 29;
            z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
            (z >> 40) as f32 / (1u64 << 24) as f32
        };
        let (gx, gy) = (x / 8.0, y / 8.0);
        let (x0, y0) = (gx.floor(), gy.floor());
        let (fx, fy) = (gx - x0, gy - y0);
        let (i, j) = (x0 as i64, y0 as i64);
        let top = lat(i, j) + (lat(i + 1, j) - lat(i, j)) * fx;
        let bot = lat(i, j + 1) + (lat(i + 1, j + 1) - lat(i, j + 1)) * fx;
        40.0 + 170.0 * (top + (bot - top) * fy)
    }

    fn frame(w: u32, h: u32, f: impl Fn(f32, f32) -> f32) -> GrayFrame {
        let mut data = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                data.push(f(x as f32, y as f32).round().clamp(0.0, 255.0) as u8);
            }
        }
        GrayFrame::from_vec(w, h, data).unwrap()
    }

    fn pyr(f: &GrayFrame) -> Pyramid {
        let mut p = Pyramid::default();
        p.build(f, 3);
        p
    }

    #[test]
    fn finds_a_shifted_target_with_subpixel_accuracy() {
        let params = RedetectParams::default();
        let sel = Point2::new(200.0, 150.0);
        let mut s = Searcher::default();
        s.prepare(&pyr(&frame(320, 240, noise)), sel, 10, &params);
        assert!(s.is_ready());
        // Shift far beyond any LK guard: (−95.4, +41.7).
        let (dx, dy) = (-95.4f32, 41.7f32);
        let moved = frame(320, 240, |x, y| noise(x - dx, y - dy));
        match s.search(&pyr(&moved), &params, None) {
            SearchResult::Found { position, score } => {
                let err = (position.x - sel.x - dx).hypot(position.y - sel.y - dy);
                assert!(err < 0.5, "error {err} at {position:?}");
                assert!(score > 0.95, "{score}");
            }
            r => panic!("{r:?}"),
        }
    }

    #[test]
    fn refuses_a_periodic_target_as_ambiguous() {
        let params = RedetectParams::default();
        let grating = |x: f32, _y: f32| 128.0 + 60.0 * (x * std::f32::consts::TAU / 16.0).sin();
        let f = frame(320, 240, grating);
        let mut s = Searcher::default();
        s.prepare(&pyr(&f), Point2::new(160.0, 120.0), 10, &params);
        match s.search(&pyr(&f), &params, None) {
            SearchResult::NotFound { best, second } => {
                assert!(best - second < params.min_margin, "{best} {second}")
            }
            r => panic!("periodic target must be ambiguous, got {r:?}"),
        }
    }

    #[test]
    fn flat_selection_is_not_ready_and_never_finds() {
        let params = RedetectParams::default();
        let f = frame(64, 48, |_, _| 100.0);
        let mut s = Searcher::default();
        s.prepare(&pyr(&f), Point2::new(30.0, 20.0), 10, &params);
        assert!(!s.is_ready());
        assert!(matches!(
            s.search(&pyr(&f), &params, None),
            SearchResult::NotFound { .. }
        ));
    }

    #[test]
    fn tiny_frame_does_not_panic() {
        let params = RedetectParams::default();
        let f = frame(24, 18, pattern);
        let mut s = Searcher::default();
        s.prepare(&pyr(&f), Point2::new(12.0, 9.0), 10, &params);
        let _ = s.search(&pyr(&f), &params, None);
    }
}
