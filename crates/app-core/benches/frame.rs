//! Per-frame cost of the controller (task 0009 slice 2a: "where do the 19 ms `track` go").
//! Run: `cargo bench -p app-core --bench frame` (release) or add `--profile dev` to see what
//! the dev build pays. Compile-checked in CI by `cargo clippy --all-targets`.
//!
//! Input: a 640×480 textured frame with luma + RGB, the camera's native orientation, mirror
//! on (the default), so `handle(Frame)` does everything it does in the app.

use app_core::color::{downsample_rgb_to_width, ingest_rgb};
use app_core::ports::{CameraFrame, CaptureFormat, GrayFrame, Point2, RgbFrame};
use app_core::{AppState, Command, Controller, DesktopRect, Input, Settings};
use divan::{Bencher, black_box};

const W: u32 = 640;
const H: u32 = 480;
const FRAME_US: u64 = 33_333;
const DESK: DesktopRect = DesktopRect {
    x: 0.0,
    y: 0.0,
    width: 1920.0,
    height: 1080.0,
};

fn main() {
    divan::main();
}

/// Deterministic value noise with 4 px cells: enough texture for the tracker to lock on.
fn texture(x: i64, y: i64) -> u8 {
    let cell = |i: i64, j: i64| {
        let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (j as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z ^= z >> 29;
        z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
        (z >> 56) as u8
    };
    let (i, j) = (x.div_euclid(4), y.div_euclid(4));
    let v = (u32::from(cell(i, j)) + u32::from(cell(i + 1, j)) + u32::from(cell(i, j + 1))) / 3;
    30 + (v * 200 / 255) as u8
}

/// Frame `n` of a scene drifting one pixel right per frame (so the tracker really moves).
fn frame(n: u64) -> CameraFrame {
    let mut luma = Vec::with_capacity((W * H) as usize);
    let mut rgb = Vec::with_capacity((W * H * 3) as usize);
    for y in 0..H as i64 {
        for x in 0..W as i64 {
            let v = texture(x - n as i64, y);
            luma.push(v);
            rgb.extend([v, v / 2, 255 - v]);
        }
    }
    CameraFrame {
        luma: GrayFrame::from_vec(W, H, luma).unwrap(),
        rgb: Some(RgbFrame::from_vec(W, H, rgb).unwrap()),
        timestamp_us: n * FRAME_US,
    }
}

/// A controller in `Tracking` with the camera in control, after `n` warm-up frames.
fn tracking_controller() -> (Controller, u64) {
    let mut c = Controller::new(Settings::default(), DESK).unwrap();
    let mut now = 1_000_000;
    c.handle(Input::Command(Command::ToggleStart), now);
    c.handle(
        Input::CameraOpened(CaptureFormat {
            width: W,
            height: H,
            fps: 30.0,
        }),
        now,
    );
    now += FRAME_US;
    c.handle(Input::Frame(frame(0)), now);
    c.handle(
        Input::Command(Command::SelectTarget(Point2::new(320.0, 240.0))),
        now,
    );
    assert_eq!(c.state(), AppState::Tracking);
    (c, now)
}

/// Everything `handle(Frame)` does while tracking: ingest (downsample, mirror) of luma and
/// RGB, the tracker step, pointer mapping.
#[divan::bench]
fn handle_frame_tracking(bencher: Bencher) {
    let frames: Vec<CameraFrame> = (1..=8).map(frame).collect();
    let (mut c, mut now) = tracking_controller();
    let mut i = 0;
    bencher
        .with_inputs(|| {
            i = (i + 1) % frames.len();
            frames[i].clone()
        })
        .bench_local_values(|f| {
            now += FRAME_US;
            black_box(c.handle(Input::Frame(f), now))
        });
}

/// Ingest only (no target selected): the share of the frame cost that isn't tracking.
#[divan::bench]
fn handle_frame_no_target(bencher: Bencher) {
    let f = frame(1);
    let mut c = Controller::new(Settings::default(), DESK).unwrap();
    let mut now = 1_000_000;
    c.handle(Input::Command(Command::ToggleStart), now);
    bencher.with_inputs(|| f.clone()).bench_local_values(|f| {
        now += FRAME_US;
        black_box(c.handle(Input::Frame(f), now))
    });
}

/// The colour preview's ingest alone: downsample (a pass-through at 640) + mirror.
#[divan::bench]
fn rgb_ingest_mirror(bencher: Bencher) {
    let rgb = frame(1).rgb.unwrap();
    bencher
        .with_inputs(|| rgb.clone())
        .bench_values(|f| ingest_rgb(downsample_rgb_to_width(f, W), true));
}
