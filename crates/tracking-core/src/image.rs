//! f32 image primitives for the trackers.
//!
//! Coordinate convention (used everywhere in tracking-core): pixel `(i, j)` has its centre at
//! the continuous point `(x = i, y = j)`. `x` grows right, `y` grows down, origin at the
//! top-left pixel centre. Sampling outside the image clamps to the nearest edge pixel.

use crate::GrayFrame;

/// Row-major f32 image. Reused across frames via the `*_into` functions (no steady-state
/// allocation once sizes are stable).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImageF32 {
    width: usize,
    height: usize,
    data: Vec<f32>,
}

impl ImageF32 {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; width * height],
        }
    }

    /// Resizes in place, keeping the allocation when capacity allows. Contents are unspecified.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.data.resize(width * height, 0.0);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    #[inline]
    pub fn at(&self, x: usize, y: usize) -> f32 {
        self.data[y * self.width + x]
    }

    #[inline]
    fn at_clamped(&self, x: isize, y: isize) -> f32 {
        let x = x.clamp(0, self.width as isize - 1) as usize;
        let y = y.clamp(0, self.height as isize - 1) as usize;
        self.data[y * self.width + x]
    }

    /// Bilinear sample at a continuous point, clamp-to-edge. Callers pass finite coordinates.
    #[inline]
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let x0 = x.floor();
        let y0 = y.floor();
        let fx = x - x0;
        let fy = y - y0;
        let (xi, yi) = (x0 as isize, y0 as isize);
        let a = self.at_clamped(xi, yi);
        let b = self.at_clamped(xi + 1, yi);
        let c = self.at_clamped(xi, yi + 1);
        let d = self.at_clamped(xi + 1, yi + 1);
        bilerp(fx, fy, a, b, c, d)
    }

    /// True if `(x, y)` lies within the pixel-centre hull `[0, w−1] × [0, h−1]`.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= 0.0 && y >= 0.0 && x <= (self.width - 1) as f32 && y <= (self.height - 1) as f32
    }
}

/// Bilinear blend shared by every sampler, so they round identically.
#[inline]
fn bilerp(fx: f32, fy: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    let top = a + (b - a) * fx;
    let bottom = c + (d - c) * fx;
    top + (bottom - top) * fy
}

/// Converts an 8-bit frame into `dst` (resized as needed).
pub fn gray_to_f32_into(src: &GrayFrame, dst: &mut ImageF32) {
    dst.resize(src.width() as usize, src.height() as usize);
    for (d, &s) in dst.data.iter_mut().zip(src.as_bytes()) {
        *d = f32::from(s);
    }
}

/// Reflect-101 index (OpenCV's default border): `-1 → 1`, `n → n−2`.
#[inline]
fn reflect101(i: isize, n: usize) -> usize {
    let n = n as isize;
    if n == 1 {
        return 0;
    }
    let mut i = i;
    while i < 0 || i >= n {
        i = if i < 0 { -i } else { 2 * (n - 1) - i };
    }
    i as usize
}

const BINOMIAL5: [f32; 5] = [1.0 / 16.0, 4.0 / 16.0, 6.0 / 16.0, 4.0 / 16.0, 1.0 / 16.0];

/// Gaussian pyramid step: 5-tap binomial blur then keep even pixels. Output size is
/// `((w + 1) / 2, (h + 1) / 2)`, so `dst(i, j)` sits at `src(2i, 2j)` and a point `p` at
/// level `L` is `p / 2^L` at level `L + 1`. `tmp` is scratch (resized as needed).
pub fn pyr_down_into(src: &ImageF32, dst: &mut ImageF32, tmp: &mut ImageF32) {
    let (w, h) = (src.width, src.height);
    let (dw, dh) = (w.div_ceil(2), h.div_ceil(2));
    // Horizontal pass at decimated columns, all rows: tmp is dw × h.
    // Interior samples (all 5 taps in range) skip the border logic; the summation order is
    // the same either way, so results are bit-identical.
    tmp.resize(dw, h);
    for y in 0..h {
        let row = &src.data[y * w..(y + 1) * w];
        let out = &mut tmp.data[y * dw..(y + 1) * dw];
        for (x, o) in out.iter_mut().enumerate() {
            let cx = 2 * x;
            let mut acc = 0.0;
            if cx >= 2 && cx + 2 < w {
                for (wk, v) in BINOMIAL5.iter().zip(&row[cx - 2..cx + 3]) {
                    acc += wk * v;
                }
            } else {
                for (k, wk) in BINOMIAL5.iter().enumerate() {
                    acc += wk * row[reflect101(cx as isize + k as isize - 2, w)];
                }
            }
            *o = acc;
        }
    }
    // Vertical pass at decimated rows: five source rows, combined column-wise.
    dst.resize(dw, dh);
    for y in 0..dh {
        let cy = 2 * y as isize;
        let r: [usize; 5] = std::array::from_fn(|k| reflect101(cy + k as isize - 2, h) * dw);
        let out = &mut dst.data[y * dw..(y + 1) * dw];
        for (x, o) in out.iter_mut().enumerate() {
            let mut acc = 0.0;
            for (wk, rk) in BINOMIAL5.iter().zip(r) {
                acc += wk * tmp.data[rk + x];
            }
            *o = acc;
        }
    }
}

/// Scharr derivatives in intensity units per pixel:
/// `gx = (3·Δ(y−1) + 10·Δ(y) + 3·Δ(y+1)) / 32` with `Δ(r) = I(x+1, r) − I(x−1, r)`, and the
/// transpose for `gy`. Borders use reflect-101, so edge-column gradients are zero across x.
pub fn scharr_into(src: &ImageF32, gx: &mut ImageF32, gy: &mut ImageF32) {
    let (w, h) = (src.width, src.height);
    gx.resize(w, h);
    gy.resize(w, h);
    let p = |x: isize, y: isize| src.data[reflect101(y, h) * w + reflect101(x, w)];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let dx = |r: isize| p(x + 1, r) - p(x - 1, r);
            let dy = |c: isize| p(c, y + 1) - p(c, y - 1);
            let i = y as usize * w + x as usize;
            gx.data[i] = (3.0 * dx(y - 1) + 10.0 * dx(y) + 3.0 * dx(y + 1)) / 32.0;
            gy.data[i] = (3.0 * dy(x - 1) + 10.0 * dy(x) + 3.0 * dy(x + 1)) / 32.0;
        }
    }
}

/// Scharr derivatives at one pixel; exactly the value [`scharr_into`] writes there.
#[inline]
pub fn scharr_at(src: &ImageF32, x: usize, y: usize) -> (f32, f32) {
    let (w, h) = (src.width, src.height);
    let d = &src.data;
    if x >= 1 && y >= 1 && x + 1 < w && y + 1 < h {
        let p = |x: usize, y: usize| d[y * w + x];
        let dx = |r: usize| p(x + 1, r) - p(x - 1, r);
        let dy = |c: usize| p(c, y + 1) - p(c, y - 1);
        (
            (3.0 * dx(y - 1) + 10.0 * dx(y) + 3.0 * dx(y + 1)) / 32.0,
            (3.0 * dy(x - 1) + 10.0 * dy(x) + 3.0 * dy(x + 1)) / 32.0,
        )
    } else {
        let p = |x: isize, y: isize| d[reflect101(y, h) * w + reflect101(x, w)];
        let (x, y) = (x as isize, y as isize);
        let dx = |r: isize| p(x + 1, r) - p(x - 1, r);
        let dy = |c: isize| p(c, y + 1) - p(c, y - 1);
        (
            (3.0 * dx(y - 1) + 10.0 * dx(y) + 3.0 * dx(y + 1)) / 32.0,
            (3.0 * dy(x - 1) + 10.0 * dy(x) + 3.0 * dy(x + 1)) / 32.0,
        )
    }
}

/// Scharr gradients of a rectangular patch of an image, sampled like the full-image
/// gradient maps would be (bilinear, clamp-to-edge) — bit-identical to sampling
/// [`scharr_into`]'s output, for any point whose four bilinear neighbours (after clamping)
/// lie inside the patch. LK only reads gradients in its window, so this replaces full-image
/// gradient maps (Phase 4 E0). Buffers are reused across [`GradPatch::fill`] calls.
#[derive(Debug, Clone, Default)]
pub struct GradPatch {
    x0: usize,
    y0: usize,
    pw: usize,
    ph: usize,
    img_w: usize,
    img_h: usize,
    gx: Vec<f32>,
    gy: Vec<f32>,
}

impl GradPatch {
    /// Computes gradients for every pixel in `[x_lo, x_hi] × [y_lo, y_hi]` (inclusive, clamped
    /// to the image).
    pub fn fill(&mut self, src: &ImageF32, x_lo: isize, x_hi: isize, y_lo: isize, y_hi: isize) {
        let cx = |v: isize| v.clamp(0, src.width as isize - 1) as usize;
        let cy = |v: isize| v.clamp(0, src.height as isize - 1) as usize;
        let (x0, x1, y0, y1) = (cx(x_lo), cx(x_hi), cy(y_lo), cy(y_hi));
        self.x0 = x0;
        self.y0 = y0;
        self.pw = x1 + 1 - x0;
        self.ph = y1 + 1 - y0;
        self.img_w = src.width;
        self.img_h = src.height;
        let n = self.pw * self.ph;
        self.gx.resize(n, 0.0);
        self.gy.resize(n, 0.0);
        let mut i = 0;
        for y in y0..=y1 {
            for x in x0..=x1 {
                (self.gx[i], self.gy[i]) = scharr_at(src, x, y);
                i += 1;
            }
        }
    }

    #[inline]
    fn index(&self, x: isize, y: isize) -> usize {
        let x = x.clamp(0, self.img_w as isize - 1) as usize;
        let y = y.clamp(0, self.img_h as isize - 1) as usize;
        debug_assert!(x >= self.x0 && x < self.x0 + self.pw, "x {x} outside patch");
        debug_assert!(y >= self.y0 && y < self.y0 + self.ph, "y {y} outside patch");
        (y - self.y0) * self.pw + (x - self.x0)
    }

    /// Bilinear `(gx, gy)` at a continuous image point, clamp-to-edge like [`ImageF32::sample`].
    #[inline]
    pub fn sample(&self, x: f32, y: f32) -> (f32, f32) {
        let x0 = x.floor();
        let y0 = y.floor();
        let fx = x - x0;
        let fy = y - y0;
        let (xi, yi) = (x0 as isize, y0 as isize);
        let (ia, ib) = (self.index(xi, yi), self.index(xi + 1, yi));
        let (ic, id) = (self.index(xi, yi + 1), self.index(xi + 1, yi + 1));
        let s = |g: &[f32]| bilerp(fx, fy, g[ia], g[ib], g[ic], g[id]);
        (s(&self.gx), s(&self.gy))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(w: usize, h: usize, f: impl Fn(usize, usize) -> f32) -> ImageF32 {
        let mut m = ImageF32::new(w, h);
        for y in 0..h {
            for x in 0..w {
                m.data[y * w + x] = f(x, y);
            }
        }
        m
    }

    #[test]
    fn bilinear_hits_pixel_centres_and_interpolates() {
        let m = img(2, 2, |x, y| (x + 2 * y) as f32 * 10.0); // 0 10 / 20 30
        assert_eq!(m.sample(0.0, 0.0), 0.0);
        assert_eq!(m.sample(1.0, 1.0), 30.0);
        assert_eq!(m.sample(0.5, 0.0), 5.0);
        assert_eq!(m.sample(0.5, 0.5), 15.0);
        assert_eq!(m.sample(0.25, 0.75), 17.5);
    }

    #[test]
    fn bilinear_clamps_outside() {
        let m = img(2, 2, |x, y| (x + 2 * y) as f32 * 10.0);
        assert_eq!(m.sample(-5.0, -5.0), 0.0);
        assert_eq!(m.sample(9.0, 0.0), 10.0);
        assert_eq!(m.sample(9.0, 9.0), 30.0);
    }

    #[test]
    fn reflect101_matches_opencv() {
        assert_eq!(reflect101(-1, 5), 1);
        assert_eq!(reflect101(-2, 5), 2);
        assert_eq!(reflect101(5, 5), 3);
        assert_eq!(reflect101(6, 5), 2);
        assert_eq!(reflect101(-3, 1), 0);
    }

    #[test]
    fn pyr_down_sizes_round_up_and_preserve_constants() {
        let src = img(7, 4, |_, _| 42.0);
        let (mut dst, mut tmp) = (ImageF32::default(), ImageF32::default());
        pyr_down_into(&src, &mut dst, &mut tmp);
        assert_eq!((dst.width(), dst.height()), (4, 2));
        assert!(dst.as_slice().iter().all(|&v| (v - 42.0).abs() < 1e-5));
    }

    #[test]
    fn pyr_down_preserves_linear_ramp_in_interior() {
        // A linear ramp is reproduced exactly by a symmetric normalised kernel.
        let src = img(16, 4, |x, _| x as f32);
        let (mut dst, mut tmp) = (ImageF32::default(), ImageF32::default());
        pyr_down_into(&src, &mut dst, &mut tmp);
        for x in 1..7 {
            assert!((dst.at(x, 1) - (2 * x) as f32).abs() < 1e-5, "x={x}");
        }
    }

    #[test]
    fn scharr_on_ramps_gives_unit_slopes() {
        let src = img(6, 6, |x, y| 2.0 * x as f32 - 3.0 * y as f32);
        let (mut gx, mut gy) = (ImageF32::default(), ImageF32::default());
        scharr_into(&src, &mut gx, &mut gy);
        for y in 1..5 {
            for x in 1..5 {
                assert!((gx.at(x, y) - 2.0).abs() < 1e-5);
                assert!((gy.at(x, y) + 3.0).abs() < 1e-5);
            }
        }
        // Reflect-101 border: symmetric neighbourhood → zero derivative across the edge.
        assert_eq!(gx.at(0, 2), 0.0);
    }

    #[test]
    fn flat_image_has_zero_gradient_and_no_nan() {
        let src = img(3, 3, |_, _| 7.0);
        let (mut gx, mut gy) = (ImageF32::default(), ImageF32::default());
        scharr_into(&src, &mut gx, &mut gy);
        assert!(gx.as_slice().iter().chain(gy.as_slice()).all(|&v| v == 0.0));
    }

    /// Irregular image so border and interior paths both see non-trivial values.
    fn bumpy(w: usize, h: usize) -> ImageF32 {
        img(w, h, |x, y| {
            ((x * 7919 + y * 104_729) % 251) as f32 + 0.37 * (x as f32 * 0.7).sin()
        })
    }

    #[test]
    fn scharr_at_matches_full_image_everywhere() {
        for (w, h) in [(9, 7), (2, 3), (1, 1), (16, 5)] {
            let src = bumpy(w, h);
            let (mut gx, mut gy) = (ImageF32::default(), ImageF32::default());
            scharr_into(&src, &mut gx, &mut gy);
            for y in 0..h {
                for x in 0..w {
                    let (a, b) = scharr_at(&src, x, y);
                    assert_eq!(a.to_bits(), gx.at(x, y).to_bits(), "{w}x{h} gx@{x},{y}");
                    assert_eq!(b.to_bits(), gy.at(x, y).to_bits(), "{w}x{h} gy@{x},{y}");
                }
            }
        }
    }

    #[test]
    fn grad_patch_sampling_is_bit_identical_to_full_maps() {
        let src = bumpy(20, 14);
        let (mut gx, mut gy) = (ImageF32::default(), ImageF32::default());
        scharr_into(&src, &mut gx, &mut gy);
        let mut patch = GradPatch::default();
        // Includes a window hanging off the top-left corner (clamped sampling).
        for (cx, cy) in [(10.3f32, 6.8f32), (-2.6, -1.2), (18.9, 13.4), (0.0, 0.0)] {
            let hw = 3.0;
            patch.fill(
                &src,
                (cx - hw - 1.0).floor() as isize,
                (cx + hw + 1.0).floor() as isize + 1,
                (cy - hw - 1.0).floor() as isize,
                (cy + hw + 1.0).floor() as isize + 1,
            );
            for dy in -3..=3 {
                for dx in -3..=3 {
                    let (x, y) = (cx + dx as f32, cy + dy as f32);
                    let (a, b) = patch.sample(x, y);
                    assert_eq!(a.to_bits(), gx.sample(x, y).to_bits(), "gx@{x},{y}");
                    assert_eq!(b.to_bits(), gy.sample(x, y).to_bits(), "gy@{x},{y}");
                }
            }
        }
    }

    #[test]
    fn pyr_down_fast_path_matches_reflect_reference() {
        let src = bumpy(23, 17);
        let (mut dst, mut tmp) = (ImageF32::default(), ImageF32::default());
        pyr_down_into(&src, &mut dst, &mut tmp);
        // Straightforward reference: every tap through reflect101, same summation order.
        let (w, h) = (23usize, 17usize);
        let (dw, dh) = (w.div_ceil(2), h.div_ceil(2));
        let mut t = vec![0.0f32; dw * h];
        for y in 0..h {
            for x in 0..dw {
                let mut acc = 0.0;
                for (k, wk) in BINOMIAL5.iter().enumerate() {
                    acc += wk * src.at(reflect101((2 * x) as isize + k as isize - 2, w), y);
                }
                t[y * dw + x] = acc;
            }
        }
        for y in 0..dh {
            for x in 0..dw {
                let mut acc = 0.0;
                for (k, wk) in BINOMIAL5.iter().enumerate() {
                    acc += wk * t[reflect101((2 * y) as isize + k as isize - 2, h) * dw + x];
                }
                assert_eq!(acc.to_bits(), dst.at(x, y).to_bits(), "@{x},{y}");
            }
        }
    }

    #[test]
    fn gray_conversion_keeps_values() {
        let g = GrayFrame::from_vec(2, 1, vec![0, 255]).unwrap();
        let mut f = ImageF32::default();
        gray_to_f32_into(&g, &mut f);
        assert_eq!(f.as_slice(), &[0.0, 255.0]);
    }
}
