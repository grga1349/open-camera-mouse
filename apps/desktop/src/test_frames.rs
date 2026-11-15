//! Synthetic camera frames for tests (as in app-core's controller tests): value noise that
//! a tracker can follow, shifted by a "head" offset, in raw (unmirrored) orientation.

use app_core::ports::{CameraFrame, GrayFrame, RgbFrame};

pub const W: u32 = 640;
pub const H: u32 = 480;
pub const FRAME_US: u64 = 33_333;
pub const SCENE: u32 = 7;
/// A flat, textureless frame (hand over the camera): a certain loss.
pub const FLAT: u32 = 99;

fn hash(i: i64, j: i64, seed: u32) -> f32 {
    let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (j as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ u64::from(seed).wrapping_mul(0xD6E8_FEB8_6659_FD93);
    z ^= z >> 29;
    z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 32;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

/// Value noise with 4 px cells: fine enough for re-detection to arm.
fn noise(x: f32, y: f32, seed: u32) -> f32 {
    let (gx, gy) = (x / 4.0, y / 4.0);
    let (x0, y0) = (gx.floor(), gy.floor());
    let (fx, fy) = (gx - x0, gy - y0);
    let (i, j) = (x0 as i64, y0 as i64);
    let a = hash(i, j, seed) * (1.0 - fx) + hash(i + 1, j, seed) * fx;
    let b = hash(i, j + 1, seed) * (1.0 - fx) + hash(i + 1, j + 1, seed) * fx;
    a * (1.0 - fy) + b * fy
}

/// A raw frame whose *display* (mirrored) content is the scene moved by (`dx`, `dy`): a
/// positive `dx` is the head moving right on screen.
pub fn shifted(w: u32, h: u32, dx: f32, dy: f32, seed: u32, ts: u64) -> CameraFrame {
    let mut data = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            // Raw is the mirror of display: raw x ↔ display w − 1 − x.
            let v = if seed == FLAT {
                0.5
            } else {
                noise((w - 1 - x) as f32 - dx, y as f32 - dy, seed)
            };
            data.push((30.0 + 200.0 * v) as u8);
        }
    }
    CameraFrame {
        luma: GrayFrame::from_vec(w, h, data).unwrap(),
        rgb: None,
        timestamp_us: ts,
    }
}

/// Like [`shifted`] with a matching grey colour image, as the real camera delivers.
pub fn with_color(mut f: CameraFrame) -> CameraFrame {
    let (w, h) = (f.luma.width(), f.luma.height());
    let rgb = f.luma.as_bytes().iter().flat_map(|&v| [v, v, v]).collect();
    f.rgb = Some(RgbFrame::from_vec(w, h, rgb).unwrap());
    f
}
