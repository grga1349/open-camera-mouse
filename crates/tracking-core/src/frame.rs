//! Owned 8-bit grayscale frames and the canonical-space ingest step (ADR-0008).

use thiserror::Error;

/// Errors constructing a frame from raw parts.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FrameError {
    #[error("frame dimensions must be non-zero, got {width}x{height}")]
    ZeroSize { width: u32, height: u32 },
    #[error("buffer length {actual} does not match {width}x{height} = {expected}")]
    LengthMismatch {
        width: u32,
        height: u32,
        expected: usize,
        actual: usize,
    },
}

/// Row-major, tightly packed 8-bit luma frame. Index `(x, y)` is `data[y * width + x]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrayFrame {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl GrayFrame {
    /// Wraps an existing buffer. Fails if the size is zero or the length is wrong.
    pub fn from_vec(width: u32, height: u32, data: Vec<u8>) -> Result<Self, FrameError> {
        if width == 0 || height == 0 {
            return Err(FrameError::ZeroSize { width, height });
        }
        let expected = width as usize * height as usize;
        if data.len() != expected {
            return Err(FrameError::LengthMismatch {
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

    /// A frame filled with one value.
    pub fn filled(width: u32, height: u32, value: u8) -> Result<Self, FrameError> {
        Self::from_vec(width, height, vec![value; width as usize * height as usize])
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

    /// Pixel at `(x, y)`, or `None` outside the frame.
    pub fn get(&self, x: u32, y: u32) -> Option<u8> {
        (x < self.width && y < self.height)
            .then(|| self.data[y as usize * self.width as usize + x as usize])
    }

    /// Reverses every row in place (horizontal mirror). No allocation.
    pub fn mirror_horizontal(&mut self) {
        for row in self.data.chunks_exact_mut(self.width as usize) {
            row.reverse();
        }
    }
}

/// Converts a camera frame into the one canonical display space used by selection, overlay,
/// tracking and pointer mapping (BEHAVIOR_SPEC R-C1, ADR-0008: mirror at ingest).
pub fn ingest(mut frame: GrayFrame, mirror: bool) -> GrayFrame {
    if mirror {
        frame.mirror_horizontal();
    }
    frame
}

/// Integer box downsample so the width is at most `max_width` (ADR-0008). Returns the frame
/// unchanged if it already fits. Factor `k = ceil(width / max_width)`; output is
/// `(width / k, height / k)` and pixel `(i, j)` averages source `[k·i, k·i + k) × [k·j, k·j + k)`
/// with rounding, so a source point `p` maps to `(p + 0.5) / k − 0.5`.
pub fn downsample_to_width(frame: GrayFrame, max_width: u32) -> GrayFrame {
    let max_width = max_width.max(1);
    if frame.width <= max_width {
        return frame;
    }
    let k = frame.width.div_ceil(max_width);
    let (w, h) = (frame.width / k, (frame.height / k).max(1));
    let (k, sw) = (k as usize, frame.width as usize);
    let area = (k * k) as u32;
    let mut data = Vec::with_capacity(w as usize * h as usize);
    for y in 0..h as usize {
        for x in 0..w as usize {
            let mut acc = 0u32;
            for dy in 0..k {
                let row = (y * k + dy) * sw + x * k;
                acc += frame.data[row..row + k]
                    .iter()
                    .map(|&b| u32::from(b))
                    .sum::<u32>();
            }
            data.push(((acc + area / 2) / area) as u8);
        }
    }
    GrayFrame {
        width: w,
        height: h,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_size_and_wrong_length() {
        assert_eq!(
            GrayFrame::from_vec(0, 2, vec![]),
            Err(FrameError::ZeroSize {
                width: 0,
                height: 2
            })
        );
        assert!(matches!(
            GrayFrame::from_vec(2, 2, vec![0; 3]),
            Err(FrameError::LengthMismatch {
                expected: 4,
                actual: 3,
                ..
            })
        ));
    }

    #[test]
    fn get_is_row_major_and_bounds_checked() {
        let f = GrayFrame::from_vec(3, 2, vec![0, 1, 2, 3, 4, 5]).unwrap();
        assert_eq!(f.get(2, 0), Some(2));
        assert_eq!(f.get(0, 1), Some(3));
        assert_eq!(f.get(3, 0), None);
        assert_eq!(f.get(0, 2), None);
    }

    #[test]
    fn mirror_reverses_rows_and_is_an_involution() {
        let original = GrayFrame::from_vec(3, 2, vec![0, 1, 2, 3, 4, 5]).unwrap();
        let mirrored = ingest(original.clone(), true);
        assert_eq!(mirrored.as_bytes(), &[2, 1, 0, 5, 4, 3]);
        assert_eq!(ingest(mirrored, true), original);
    }

    #[test]
    fn downsample_halves_1280_and_averages_blocks() {
        let f = GrayFrame::from_vec(4, 2, vec![0, 2, 10, 20, 4, 6, 30, 40]).unwrap();
        let d = downsample_to_width(f, 2);
        assert_eq!((d.width(), d.height()), (2, 1));
        assert_eq!(d.as_bytes(), &[3, 25]);
        let big = GrayFrame::filled(1280, 720, 9).unwrap();
        let d = downsample_to_width(big, 640);
        assert_eq!((d.width(), d.height()), (640, 360));
        let same = GrayFrame::filled(640, 480, 1).unwrap();
        assert_eq!(downsample_to_width(same.clone(), 640), same);
    }

    #[test]
    fn ingest_without_mirror_is_identity() {
        let f = GrayFrame::from_vec(2, 1, vec![7, 9]).unwrap();
        assert_eq!(ingest(f.clone(), false), f);
    }
}
