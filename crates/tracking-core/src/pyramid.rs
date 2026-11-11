//! Gaussian image pyramid. Gradients are not stored: LK computes them for its window only
//! ([`crate::image::GradPatch`], Phase 4 E0).

use crate::GrayFrame;
use crate::image::{ImageF32, gray_to_f32_into, pyr_down_into};

/// Smallest level side kept. Coarser levels than this carry too little signal for a window.
const MIN_LEVEL_SIDE: usize = 8;

#[derive(Debug, Clone, Default)]
pub struct Level {
    pub img: ImageF32,
}

/// Levels `0..=max_level` (OpenCV semantics: `max_level = 3` means four levels). Fewer levels
/// are built if the image gets smaller than [`MIN_LEVEL_SIDE`]. Buffers are reused across
/// [`Pyramid::build`] calls.
#[derive(Debug, Clone, Default)]
pub struct Pyramid {
    levels: Vec<Level>,
    len: usize,
    tmp: ImageF32,
}

impl Pyramid {
    pub fn build(&mut self, frame: &GrayFrame, max_level: usize) {
        if self.levels.len() < max_level + 1 {
            self.levels.resize_with(max_level + 1, Level::default);
        }
        gray_to_f32_into(frame, &mut self.levels[0].img);
        self.len = 1;
        for l in 1..=max_level {
            let (done, rest) = self.levels.split_at_mut(l);
            let src = &done[l - 1].img;
            if src.width().div_ceil(2) < MIN_LEVEL_SIDE || src.height().div_ceil(2) < MIN_LEVEL_SIDE
            {
                break;
            }
            pyr_down_into(src, &mut rest[0].img, &mut self.tmp);
            self.len += 1;
        }
    }

    /// Number of built levels (≥ 1 after a build).
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn level(&self, l: usize) -> &Level {
        assert!(l < self.len, "level {l} not built");
        &self.levels[l]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_requested_levels_with_halving_sizes() {
        let f = GrayFrame::filled(640, 480, 10).unwrap();
        let mut p = Pyramid::default();
        p.build(&f, 3);
        assert_eq!(p.len(), 4);
        let sizes: Vec<_> = (0..4)
            .map(|l| (p.level(l).img.width(), p.level(l).img.height()))
            .collect();
        assert_eq!(sizes, vec![(640, 480), (320, 240), (160, 120), (80, 60)]);
    }

    #[test]
    fn stops_before_levels_get_too_small() {
        let f = GrayFrame::filled(20, 20, 0).unwrap();
        let mut p = Pyramid::default();
        p.build(&f, 3);
        assert_eq!(p.len(), 2); // 20 → 10 → (5 < 8 stops)
    }
}
