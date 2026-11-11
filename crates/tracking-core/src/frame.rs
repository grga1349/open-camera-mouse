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
    fn ingest_without_mirror_is_identity() {
        let f = GrayFrame::from_vec(2, 1, vec![7, 9]).unwrap();
        assert_eq!(ingest(f.clone(), false), f);
    }
}
