//! Colour preview frames (ADR-0008 amendment, user decision 2026-11-12: colour preview).
//!
//! The tracker runs on luma only. The camera adapter also delivers an RGB image for the
//! preview, and ingest applies the same mirror and integer box downsample to both, so the
//! preview, selection, overlay and tracking share one canonical display space (R-C1).

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RgbFrameError {
    #[error("frame dimensions must be non-zero, got {width}x{height}")]
    ZeroSize { width: u32, height: u32 },
    #[error("buffer length {actual} does not match {width}x{height}x3 = {expected}")]
    LengthMismatch {
        width: u32,
        height: u32,
        expected: usize,
        actual: usize,
    },
}

/// Row-major, tightly packed 8-bit RGB (3 bytes per pixel, no padding).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbFrame {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl RgbFrame {
    pub fn from_vec(width: u32, height: u32, data: Vec<u8>) -> Result<Self, RgbFrameError> {
        if width == 0 || height == 0 {
            return Err(RgbFrameError::ZeroSize { width, height });
        }
        let expected = width as usize * height as usize * 3;
        if data.len() != expected {
            return Err(RgbFrameError::LengthMismatch {
                width,
                height,
                expected,
                actual: data.len(),
            });
        }
        Ok(Self {
            width,
            height,
            data,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// `[r, g, b]` at `(x, y)`, or `None` outside the frame.
    pub fn get(&self, x: u32, y: u32) -> Option<[u8; 3]> {
        (x < self.width && y < self.height).then(|| {
            let i = (y as usize * self.width as usize + x as usize) * 3;
            [self.data[i], self.data[i + 1], self.data[i + 2]]
        })
    }

    /// Reverses every row in place (horizontal mirror), keeping channel order.
    pub fn mirror_horizontal(&mut self) {
        let w = self.width as usize;
        for row in self.data.chunks_exact_mut(w * 3) {
            row.as_chunks_mut::<3>().0.reverse();
        }
    }
}

/// The colour twin of `tracking_core::downsample_to_width`: same factor
/// `k = ceil(width / max_width)`, same `(width / k, height / k)` output and rounding, per
/// channel. Keeping the two identical is what keeps preview and tracking aligned.
pub fn downsample_rgb_to_width(frame: RgbFrame, max_width: u32) -> RgbFrame {
    let max_width = max_width.max(1);
    if frame.width <= max_width {
        return frame;
    }
    let k = frame.width.div_ceil(max_width);
    let (w, h) = (frame.width / k, (frame.height / k).max(1));
    let (k, sw) = (k as usize, frame.width as usize);
    let area = (k * k) as u32;
    let mut data = Vec::with_capacity(w as usize * h as usize * 3);
    for y in 0..h as usize {
        for x in 0..w as usize {
            let mut acc = [0u32; 3];
            for dy in 0..k {
                let row = ((y * k + dy) * sw + x * k) * 3;
                for px in frame.data[row..row + k * 3].as_chunks::<3>().0 {
                    for (a, &p) in acc.iter_mut().zip(px) {
                        *a += u32::from(p);
                    }
                }
            }
            data.extend(acc.map(|a| ((a + area / 2) / area) as u8));
        }
    }
    RgbFrame {
        width: w,
        height: h,
        data,
    }
}

/// Colour counterpart of `tracking_core::ingest`: mirror into canonical display space.
pub fn ingest_rgb(mut frame: RgbFrame, mirror: bool) -> RgbFrame {
    if mirror {
        frame.mirror_horizontal();
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracking_core::{GrayFrame, downsample_to_width};

    fn ramp(w: u32, h: u32) -> RgbFrame {
        let mut d = Vec::new();
        for y in 0..h {
            for x in 0..w {
                d.extend([(x * 7 + y) as u8, (x * 3) as u8, (y * 5) as u8]);
            }
        }
        RgbFrame::from_vec(w, h, d).unwrap()
    }

    #[test]
    fn rejects_bad_sizes() {
        assert!(matches!(
            RgbFrame::from_vec(0, 1, vec![]),
            Err(RgbFrameError::ZeroSize { .. })
        ));
        assert!(matches!(
            RgbFrame::from_vec(2, 1, vec![0; 5]),
            Err(RgbFrameError::LengthMismatch { expected: 6, .. })
        ));
    }

    #[test]
    fn mirror_keeps_channel_order() {
        let f = ramp(5, 2);
        let mut m = f.clone();
        m.mirror_horizontal();
        for y in 0..2 {
            for x in 0..5 {
                assert_eq!(m.get(x, y), f.get(4 - x, y));
            }
        }
        let mut back = m;
        back.mirror_horizontal();
        assert_eq!(back, f);
    }

    #[test]
    fn downsample_matches_the_luma_path_per_channel() {
        // Same factor, size and rounding as tracking-core, so preview and tracker stay aligned.
        let f = ramp(1283, 721);
        let d = downsample_rgb_to_width(f.clone(), 640);
        for c in 0..3 {
            let plane: Vec<u8> = f.as_bytes().iter().skip(c).step_by(3).copied().collect();
            let g = downsample_to_width(GrayFrame::from_vec(1283, 721, plane).unwrap(), 640);
            assert_eq!((d.width(), d.height()), (g.width(), g.height()));
            let got: Vec<u8> = d.as_bytes().iter().skip(c).step_by(3).copied().collect();
            assert_eq!(got, g.as_bytes(), "channel {c}");
        }
    }

    #[test]
    fn narrow_frames_pass_through() {
        let f = ramp(640, 480);
        assert_eq!(downsample_rgb_to_width(f.clone(), 640), f);
    }
}
