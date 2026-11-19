//! Camera buffer decode cost (task 0009 slice 2a). Runs on the capture thread, so it adds to
//! capture → pointer latency. Run: `cargo bench -p platform --bench decode` (add
//! `--profile dev` for the dev build). No camera needed.

use divan::{Bencher, black_box};
use platform::camera::{PixelFormat, decode};

const W: u32 = 640;
const H: u32 = 480;

fn main() {
    divan::main();
}

#[divan::bench(args = ["yuyv", "nv12"])]
fn decode_640x480(bencher: Bencher, name: &str) {
    let format = match name {
        "yuyv" => PixelFormat::Yuyv,
        _ => PixelFormat::Nv12,
    };
    let len = platform::camera::expected_len(format, W, H).unwrap();
    let bytes: Vec<u8> = (0..len).map(|i| (i * 31 % 251) as u8).collect();
    bencher.bench_local(|| decode(format, W, H, black_box(&bytes)));
}
