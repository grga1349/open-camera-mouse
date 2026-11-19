//! Raw camera buffers → luma (tracking) + RGB (preview). Pure, no camera crate types, so it
//! is tested on every OS.
//!
//! YUV formats keep the sensor's Y plane as the luma unchanged (no RGB round trip) and convert
//! to RGB with BT.601 limited-range integer maths, the usual webcam encoding. Packed RGB/BGR
//! derive luma with BT.601 weights.

use app_core::ports::{GrayFrame, RgbFrame};
use thiserror::Error;

/// Pixel layouts the adapter can decode. MJPEG is listed so it can be refused by name: the
/// adapter asks for uncompressed formats and decoding JPEG is not in scope (ADR-0012).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// Packed 4:2:2, byte order Y0 U Y1 V.
    Yuyv,
    /// Planar Y, then interleaved U V at half resolution in both axes.
    Nv12,
    Gray,
    Rgb,
    Bgr,
    Mjpeg,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecodeError {
    #[error("unsupported pixel format {0:?}")]
    Unsupported(PixelFormat),
    #[error("{format:?} needs even dimensions, got {width}x{height}")]
    OddSize {
        format: PixelFormat,
        width: u32,
        height: u32,
    },
    #[error("{format:?} {width}x{height} needs {expected} bytes, got {actual}")]
    Length {
        format: PixelFormat,
        width: u32,
        height: u32,
        expected: usize,
        actual: usize,
    },
    #[error("empty frame {width}x{height}")]
    Empty { width: u32, height: u32 },
}

/// Bytes a frame of this format and size occupies, if the format is decodable.
pub fn expected_len(format: PixelFormat, width: u32, height: u32) -> Option<usize> {
    let n = width as usize * height as usize;
    match format {
        PixelFormat::Yuyv => Some(n * 2),
        PixelFormat::Nv12 => Some(n + n / 2),
        PixelFormat::Gray => Some(n),
        PixelFormat::Rgb | PixelFormat::Bgr => Some(n * 3),
        PixelFormat::Mjpeg => None,
    }
}

/// Decodes one frame. Trailing bytes beyond the expected length (row padding some drivers
/// leave at the end) are ignored; a short buffer is an error.
pub fn decode(
    format: PixelFormat,
    width: u32,
    height: u32,
    bytes: &[u8],
) -> Result<(GrayFrame, RgbFrame), DecodeError> {
    if width == 0 || height == 0 {
        return Err(DecodeError::Empty { width, height });
    }
    let expected = expected_len(format, width, height).ok_or(DecodeError::Unsupported(format))?;
    if matches!(format, PixelFormat::Yuyv | PixelFormat::Nv12)
        && (!width.is_multiple_of(2) || !height.is_multiple_of(2))
    {
        return Err(DecodeError::OddSize {
            format,
            width,
            height,
        });
    }
    if bytes.len() < expected {
        return Err(DecodeError::Length {
            format,
            width,
            height,
            expected,
            actual: bytes.len(),
        });
    }
    let bytes = &bytes[..expected];
    let n = width as usize * height as usize;
    let (luma, rgb) = match format {
        PixelFormat::Yuyv => yuyv(bytes, n),
        PixelFormat::Nv12 => nv12(bytes, width as usize, height as usize),
        PixelFormat::Gray => (
            bytes.to_vec(),
            bytes.iter().flat_map(|&g| [g, g, g]).collect(),
        ),
        PixelFormat::Rgb => (
            bytes
                .as_chunks::<3>()
                .0
                .iter()
                .map(|p| luma_of(p[0], p[1], p[2]))
                .collect(),
            bytes.to_vec(),
        ),
        PixelFormat::Bgr => {
            let rgb: Vec<u8> = bytes
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|p| [p[2], p[1], p[0]])
                .collect();
            (
                rgb.as_chunks::<3>()
                    .0
                    .iter()
                    .map(|p| luma_of(p[0], p[1], p[2]))
                    .collect(),
                rgb,
            )
        }
        PixelFormat::Mjpeg => unreachable!("rejected by expected_len"),
    };
    // Sizes were checked above, so construction can't fail.
    let luma = GrayFrame::from_vec(width, height, luma).expect("luma size checked");
    let rgb = RgbFrame::from_vec(width, height, rgb).expect("rgb size checked");
    Ok((luma, rgb))
}

/// BT.601 luma from full-range RGB.
fn luma_of(r: u8, g: u8, b: u8) -> u8 {
    ((77 * u32::from(r) + 150 * u32::from(g) + 29 * u32::from(b) + 128) >> 8) as u8
}

/// BT.601 limited-range YCbCr → RGB.
fn ycbcr_to_rgb(y: u8, u: u8, v: u8) -> [u8; 3] {
    let c = 298 * (i32::from(y) - 16);
    let d = i32::from(u) - 128;
    let e = i32::from(v) - 128;
    let clamp = |x: i32| ((x + 128) >> 8).clamp(0, 255) as u8;
    [
        clamp(c + 409 * e),
        clamp(c - 100 * d - 208 * e),
        clamp(c + 516 * d),
    ]
}

fn yuyv(bytes: &[u8], n: usize) -> (Vec<u8>, Vec<u8>) {
    let mut luma = Vec::with_capacity(n);
    let mut rgb = Vec::with_capacity(n * 3);
    for q in bytes.as_chunks::<4>().0 {
        let (y0, u, y1, v) = (q[0], q[1], q[2], q[3]);
        luma.extend([y0, y1]);
        rgb.extend(ycbcr_to_rgb(y0, u, v));
        rgb.extend(ycbcr_to_rgb(y1, u, v));
    }
    (luma, rgb)
}

fn nv12(bytes: &[u8], w: usize, h: usize) -> (Vec<u8>, Vec<u8>) {
    let (y_plane, uv) = bytes.split_at(w * h);
    let mut rgb = Vec::with_capacity(w * h * 3);
    for (row, ys) in y_plane.chunks_exact(w).enumerate() {
        let uv_row = &uv[(row / 2) * w..(row / 2) * w + w];
        for (x, &y) in ys.iter().enumerate() {
            let i = (x / 2) * 2;
            rgb.extend(ycbcr_to_rgb(y, uv_row[i], uv_row[i + 1]));
        }
    }
    (y_plane.to_vec(), rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuyv_keeps_y_as_luma_and_converts_primaries() {
        // Two pixels: white (Y 235) and black (Y 16), neutral chroma.
        let (l, c) = decode(
            PixelFormat::Yuyv,
            2,
            2,
            &[235, 128, 16, 128, 81, 90, 81, 240],
        )
        .unwrap();
        assert_eq!(l.as_bytes(), &[235, 16, 81, 81]);
        assert_eq!(c.get(0, 0), Some([255, 255, 255]));
        assert_eq!(c.get(1, 0), Some([0, 0, 0]));
        // BT.601 limited red is Y 81, Cb 90, Cr 240.
        let [r, g, b] = c.get(0, 1).unwrap();
        assert!(r >= 250 && g <= 5 && b <= 5, "{r} {g} {b}");
    }

    #[test]
    fn nv12_shares_chroma_over_two_by_two_blocks() {
        // 2x2: Y plane, then one U V pair (blue: Y 41, Cb 240, Cr 110).
        let (l, c) = decode(PixelFormat::Nv12, 2, 2, &[41, 41, 41, 41, 240, 110]).unwrap();
        assert_eq!(l.as_bytes(), &[41; 4]);
        for (x, y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let [r, g, b] = c.get(x, y).unwrap();
            assert!(r <= 5 && g <= 5 && b >= 250, "{r} {g} {b}");
        }
    }

    #[test]
    fn nv12_picks_the_right_chroma_column_and_row() {
        // 4x4 with four distinct chroma blocks; pixel (3, 3) uses the last pair.
        let mut b = vec![128u8; 16];
        b.extend([128, 128, 128, 128, 128, 128, 240, 110]);
        let (_, c) = decode(PixelFormat::Nv12, 4, 4, &b).unwrap();
        assert_eq!(c.get(0, 0), c.get(1, 1));
        assert_ne!(c.get(3, 3), c.get(0, 0));
        assert_eq!(c.get(2, 2), c.get(3, 3));
    }

    #[test]
    fn packed_rgb_bgr_and_gray() {
        let (l, c) = decode(PixelFormat::Rgb, 1, 1, &[255, 0, 0]).unwrap();
        assert_eq!(
            (l.as_bytes(), c.get(0, 0)),
            (&[77u8][..], Some([255, 0, 0]))
        );
        let (_, c) = decode(PixelFormat::Bgr, 1, 1, &[255, 0, 0]).unwrap();
        assert_eq!(c.get(0, 0), Some([0, 0, 255]));
        let (l, c) = decode(PixelFormat::Gray, 1, 1, &[9]).unwrap();
        assert_eq!((l.as_bytes(), c.get(0, 0)), (&[9u8][..], Some([9, 9, 9])));
        let (l, _) = decode(PixelFormat::Rgb, 1, 1, &[255, 255, 255]).unwrap();
        assert_eq!(l.as_bytes(), &[255]);
    }

    #[test]
    fn rejects_short_odd_empty_and_mjpeg() {
        assert!(matches!(
            decode(PixelFormat::Yuyv, 2, 2, &[0; 7]),
            Err(DecodeError::Length {
                expected: 8,
                actual: 7,
                ..
            })
        ));
        assert!(matches!(
            decode(PixelFormat::Nv12, 3, 2, &[0; 9]),
            Err(DecodeError::OddSize { .. })
        ));
        assert!(matches!(
            decode(PixelFormat::Gray, 0, 2, &[]),
            Err(DecodeError::Empty { .. })
        ));
        assert_eq!(
            decode(PixelFormat::Mjpeg, 2, 2, &[0; 64]),
            Err(DecodeError::Unsupported(PixelFormat::Mjpeg))
        );
    }

    #[test]
    fn trailing_padding_is_ignored() {
        let (l, _) = decode(PixelFormat::Gray, 2, 1, &[1, 2, 99, 99]).unwrap();
        assert_eq!(l.as_bytes(), &[1, 2]);
    }
}
