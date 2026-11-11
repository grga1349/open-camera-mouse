//! Phase 3 timing + allocation harness (`benchmarking` skill, BEHAVIOR_SPEC §8).
//! Run: `cargo bench -p tracking-core`. Inputs are `head_sway` fixture frames; 1280×720 is
//! box-downsampled to 640×360 first (ADR-0008), and that downsample is benched separately.

#[path = "../tests/support/mod.rs"]
mod support;

use divan::{AllocProfiler, Bencher, black_box};
use tracking_core::image::{ImageF32, gray_to_f32_into};
use tracking_core::lk::{FlowScratch, LkParams, LkTracker, track_point};
use tracking_core::ncc::NccTracker;
use tracking_core::pyramid::Pyramid;
use tracking_core::{GrayFrame, Tracker, downsample_to_width};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

const RES: [&str; 3] = ["320x240", "640x480", "1280x720"];

fn parse(res: &str) -> (u32, u32) {
    let (w, h) = res.split_once('x').unwrap();
    (w.parse().unwrap(), h.parse().unwrap())
}

/// Two consecutive fixture frames (already at tracker resolution) and the selected point.
fn input(res: &str) -> (Vec<GrayFrame>, tracking_core::Point2) {
    let (w, h) = parse(res);
    let mut s = support::scene("head_sway");
    s.frames = 3;
    let seq = s.render(w, h);
    (seq.frames, seq.truth[0])
}

fn main() {
    divan::main();
}

#[divan::bench(args = ["1280x720"])]
fn downsample_1280_to_640(bencher: Bencher, res: &str) {
    let (w, h) = parse(res);
    let raw = GrayFrame::filled(w, h, 100).unwrap();
    bencher
        .with_inputs(|| raw.clone())
        .bench_values(|f| downsample_to_width(f, 640));
}

#[divan::bench(args = RES)]
fn gray_to_f32(bencher: Bencher, res: &str) {
    let (frames, _) = input(res);
    let mut dst = ImageF32::default();
    gray_to_f32_into(&frames[0], &mut dst);
    bencher.bench_local(|| gray_to_f32_into(black_box(&frames[0]), &mut dst));
}

#[divan::bench(args = RES)]
fn pyramid_build(bencher: Bencher, res: &str) {
    let (frames, _) = input(res);
    let mut p = Pyramid::default();
    p.build(&frames[0], 3);
    bencher.bench_local(|| p.build(black_box(&frames[0]), 3));
}

#[divan::bench(args = RES)]
fn lk_track_point(bencher: Bencher, res: &str) {
    let (frames, pt) = input(res);
    let (mut a, mut b) = (Pyramid::default(), Pyramid::default());
    a.build(&frames[0], 3);
    b.build(&frames[1], 3);
    let params = LkParams::default();
    let mut scratch = FlowScratch::default();
    track_point(&a, &b, pt, &params, &mut scratch);
    bencher.bench_local(|| track_point(&a, &b, black_box(pt), &params, &mut scratch));
}

/// End-to-end `Tracker::step` (pyramid + LK + policy), alternating two frames.
#[divan::bench(args = RES)]
fn lk_step(bencher: Bencher, res: &str) {
    let (frames, pt) = input(res);
    let mut t = LkTracker::default();
    t.select(&frames[0], pt).unwrap();
    let mut i = 0usize;
    t.step(&frames[1]).unwrap();
    bencher.bench_local(|| {
        i += 1;
        t.step(black_box(&frames[i % 2])).unwrap()
    });
}

/// End-to-end NCC step (±90 px search, 45 px template), alternating two frames.
#[divan::bench(args = RES)]
fn ncc_step(bencher: Bencher, res: &str) {
    let (frames, pt) = input(res);
    let mut t = NccTracker::default();
    t.select(&frames[0], pt).unwrap();
    let mut i = 0usize;
    t.step(&frames[1]).unwrap();
    bencher.bench_local(|| {
        i += 1;
        t.step(black_box(&frames[i % 2])).unwrap()
    });
}
