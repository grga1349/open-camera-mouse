//! On-device camera checks (task 0008 D3, PLATFORM_RISKS P1/P4/P5). Ignored by default: CI
//! has no webcam. Run on a machine with a camera:
//!
//! ```text
//! cargo test -p platform --test camera_device -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Set `OCM_CAMERA_SNAPSHOT=/path/frame.ppm` to also save one colour frame for a visual check.

use std::time::{Duration, Instant};

use app_core::ports::{CameraSource, CaptureRequest};
use platform::{NokhwaCamera, list_devices};

const FRAMES: usize = 150;
const WARMUP: usize = 30;

fn measure(cam: &mut NokhwaCamera) -> f64 {
    let mut ts = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        let f = cam.next_frame().expect("frame");
        let (g, c) = (&f.luma, f.rgb.as_ref().expect("colour"));
        assert_eq!((g.width(), g.height()), (c.width(), c.height()));
        ts.push(f.timestamp_us);
    }
    let w = &ts[WARMUP..];
    assert!(
        w.windows(2).all(|p| p[1] > p[0]),
        "timestamps must increase"
    );
    (w.len() - 1) as f64 / ((w[w.len() - 1] - w[0]) as f64 / 1e6)
}

#[test]
#[ignore = "needs a webcam"]
fn opens_640x480_delivers_30fps_and_reopens_after_close() {
    let devices = list_devices();
    println!("devices: {devices:?}");
    assert!(!devices.is_empty(), "no camera");

    let epoch = Instant::now();
    let mut cam = NokhwaCamera::new(None, epoch);
    for round in 0..2 {
        let t = Instant::now();
        let fmt = cam.open(CaptureRequest::default()).expect("open");
        let opened = t.elapsed();
        let first = cam.next_frame().expect("first frame");
        let first_at = t.elapsed();
        if round == 0
            && let Ok(path) = std::env::var("OCM_CAMERA_SNAPSHOT")
        {
            let c = first.rgb.as_ref().unwrap();
            let mut ppm = format!("P6\n{} {}\n255\n", c.width(), c.height()).into_bytes();
            ppm.extend_from_slice(c.as_bytes());
            std::fs::write(&path, ppm).unwrap();
            println!("snapshot: {path}");
        }
        let luma = first.luma.as_bytes();
        let (lo, hi) = (luma.iter().min().unwrap(), luma.iter().max().unwrap());
        assert!(
            hi - lo > 20,
            "luma looks flat ({lo}..{hi}): wrong plane or covered lens"
        );

        let fps = measure(&mut cam);
        let t = Instant::now();
        cam.close();
        let closed = t.elapsed();
        println!(
            "round {round}: negotiated {}x{} @ {} fps (reported), open {opened:?}, first frame \
             {first_at:?}, measured {fps:.2} fps over {} frames, close {closed:?}",
            fmt.width,
            fmt.height,
            fmt.fps,
            FRAMES - WARMUP,
        );
        assert_eq!((fmt.width, fmt.height), (640, 480), "P4: not 640x480");
        assert!(fps > 20.0, "measured {fps:.1} fps");
        assert!(closed < Duration::from_secs(2), "close took {closed:?}");
    }
}
