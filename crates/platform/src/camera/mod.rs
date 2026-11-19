//! Camera capture through `nokhwa` (`input-native`: AVFoundation, Media Foundation, V4L2),
//! chosen in task 0008 D3 by measurement on the dev Mac (ADR-0012).
//!
//! # Threading
//!
//! The `nokhwa` camera is created, read and dropped on one capture thread, so it never has
//! to be `Send`. The thread decodes each frame into luma + RGB ([`pixels`]) and stores it in
//! a [`LatestSlot`]: the newest frame replaces an unread one. [`NokhwaCamera::next_frame`]
//! waits on the slot with a timeout.
//!
//! # Failure mapping (ARCHITECTURE.md P1, P2, P3, P5)
//!
//! | What happens | Result |
//! |---|---|
//! | macOS camera access not granted / refused at the prompt | `PermissionDenied` |
//! | no devices, or the configured device is gone | `NoDevice` |
//! | a panic inside `nokhwa` (P1) | caught on the capture thread → `Backend` |
//! | the device doesn't open within [`OPEN_TIMEOUT`] (P3) | `Backend` (thread left to finish) |
//! | only MJPEG offered (no JPEG decoder, ADR-0012) | `Backend` |
//! | a read error, or no frame for [`FRAME_TIMEOUT`] (P5) | `Disconnected` |
//!
//! `nokhwa`'s `frame()` blocks without a timeout, so a vanished device can leave the capture
//! thread stuck in it; the reader still sees the timeout. [`NokhwaCamera::close`] waits up to
//! [`CLOSE_WAIT`] for the thread to stop the stream, then detaches it.

pub mod latest;
pub mod pixels;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use app_core::ports::{
    CameraError, CameraFrame, CameraSource, CaptureFormat, CaptureRequest, GrayFrame, RgbFrame,
};
use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, CameraInfo, FrameFormat, RequestedFormat,
    RequestedFormatType, Resolution,
};

pub use latest::{LatestSlot, Take};
pub use pixels::{DecodeError, PixelFormat, decode, expected_len};

/// Longest wait for the device to start streaming (P3: some devices hang on init).
pub const OPEN_TIMEOUT: Duration = Duration::from_secs(10);
/// No frame for this long while streaming means the camera is gone (P5). Generous, because
/// webcams lower their frame rate in the dark.
pub const FRAME_TIMEOUT: Duration = Duration::from_secs(3);
/// Longest wait for the user to answer the macOS camera prompt.
pub const PERMISSION_TIMEOUT: Duration = Duration::from_secs(120);
/// How long `close` waits for the capture thread to stop the stream.
pub const CLOSE_WAIT: Duration = Duration::from_secs(2);

/// A camera the user can pick in Options → Camera. `id` is what `Settings::camera` stores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraDevice {
    pub id: String,
    pub name: String,
}

type Slot = LatestSlot<Result<CameraFrame, CameraError>>;

#[derive(Debug)]
struct Running {
    slot: Arc<Slot>,
    stop: Arc<AtomicBool>,
    done: mpsc::Receiver<()>,
}

/// [`CameraSource`] over `nokhwa`. Frame timestamps are microseconds since `epoch`, which
/// the runtime shares with the controller's `now_us` clock.
#[derive(Debug)]
pub struct NokhwaCamera {
    device: Option<String>,
    epoch: Instant,
    running: Option<Running>,
}

impl NokhwaCamera {
    /// `device`: a [`CameraDevice::id`], or `None` for the first camera.
    pub fn new(device: Option<String>, epoch: Instant) -> Self {
        Self {
            device,
            epoch,
            running: None,
        }
    }

    /// Takes effect on the next `open`.
    pub fn set_device(&mut self, device: Option<String>) {
        self.device = device;
    }
}

impl CameraSource for NokhwaCamera {
    fn open(&mut self, request: CaptureRequest) -> Result<CaptureFormat, CameraError> {
        self.close();
        ensure_permission(PERMISSION_TIMEOUT)?;
        let index = resolve_device(self.device.as_deref())?;

        let slot = Arc::new(Slot::default());
        let stop = Arc::new(AtomicBool::new(false));
        let (opened_tx, opened_rx) = mpsc::channel();
        let (done_tx, done) = mpsc::channel();
        let (thread_slot, thread_stop, epoch) = (slot.clone(), stop.clone(), self.epoch);
        thread::Builder::new()
            .name("camera-capture".into())
            .spawn(move || {
                capture_thread(index, request, epoch, &thread_slot, &thread_stop, opened_tx);
                let _ = done_tx.send(());
            })
            .map_err(|e| CameraError::Backend(format!("capture thread: {e}")))?;

        let running = Running { slot, stop, done };
        match opened_rx.recv_timeout(OPEN_TIMEOUT) {
            Ok(Ok(format)) => {
                self.running = Some(running);
                Ok(format)
            }
            Ok(Err(e)) => Err(e),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                running.stop.store(true, Ordering::Relaxed);
                running.slot.close();
                Err(CameraError::Backend(format!(
                    "the camera did not start within {} s",
                    OPEN_TIMEOUT.as_secs()
                )))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(CameraError::Backend(
                "the capture thread ended while opening".into(),
            )),
        }
    }

    fn next_frame(&mut self) -> Result<CameraFrame, CameraError> {
        let Some(r) = &self.running else {
            return Err(CameraError::Backend("camera is not open".into()));
        };
        match r.slot.take_timeout(FRAME_TIMEOUT) {
            Take::Item(f) => f,
            Take::Timeout | Take::Closed => Err(CameraError::Disconnected),
        }
    }

    fn close(&mut self) {
        if let Some(r) = self.running.take() {
            r.stop.store(true, Ordering::Relaxed);
            r.slot.close();
            // The thread checks `stop` after each frame; a stuck read never returns (see the
            // module doc), so don't wait forever.
            let _ = r.done.recv_timeout(CLOSE_WAIT);
        }
    }
}

impl Drop for NokhwaCamera {
    fn drop(&mut self) {
        self.close();
    }
}

/// Cameras present now. Empty if there are none or the query failed.
pub fn list_devices() -> Vec<CameraDevice> {
    query_devices()
        .unwrap_or_default()
        .iter()
        .map(|i| CameraDevice {
            id: device_id(i),
            name: i.human_name(),
        })
        .collect()
}

fn query_devices() -> Result<Vec<CameraInfo>, CameraError> {
    catch_unwind(|| nokhwa::query(ApiBackend::Auto))
        .map_err(|p| CameraError::Backend(format!("camera query panicked: {}", panic_text(&p))))?
        .map_err(|e| CameraError::Backend(e.to_string()))
}

/// Stable id: the backend's unique id (`misc`) when it has one, else the name.
fn device_id(info: &CameraInfo) -> String {
    let misc = info.misc();
    if misc.trim().is_empty() {
        info.human_name()
    } else {
        misc
    }
}

fn resolve_device(wanted: Option<&str>) -> Result<CameraIndex, CameraError> {
    let devices = query_devices()?;
    let found = match wanted {
        None => devices.first(),
        Some(id) => devices
            .iter()
            .find(|i| device_id(i) == id || i.human_name() == id),
    };
    found
        .map(|i| i.index().clone())
        .ok_or(CameraError::NoDevice)
}

/// macOS asks the user once; other backends always report granted.
fn ensure_permission(timeout: Duration) -> Result<(), CameraError> {
    if nokhwa::nokhwa_check() {
        return Ok(());
    }
    let (tx, rx) = mpsc::sync_channel(1);
    nokhwa::nokhwa_initialize(move |granted| {
        let _ = tx.try_send(granted);
    });
    match rx.recv_timeout(timeout) {
        Ok(true) => Ok(()),
        Ok(false) | Err(_) => Err(CameraError::PermissionDenied),
    }
}

/// Requests tried in order. YUYV first: it is uncompressed everywhere and the format the
/// macOS backend maps correctly (`nokhwa` 0.10 maps an NV12 request to a 10-bit format on
/// macOS, which the FaceTime camera refuses, so NV12 is only asked for elsewhere).
fn requests(req: CaptureRequest) -> Vec<RequestedFormatType> {
    let res = Resolution::new(req.width, req.height);
    let mut v = vec![RequestedFormatType::Closest(CameraFormat::new(
        res,
        FrameFormat::YUYV,
        req.fps,
    ))];
    if !cfg!(target_os = "macos") {
        v.push(RequestedFormatType::Closest(CameraFormat::new(
            res,
            FrameFormat::NV12,
            req.fps,
        )));
    }
    v.push(RequestedFormatType::HighestResolution(res));
    v.push(RequestedFormatType::None);
    v
}

fn pixel_format(f: FrameFormat) -> PixelFormat {
    match f {
        FrameFormat::YUYV => PixelFormat::Yuyv,
        FrameFormat::NV12 => PixelFormat::Nv12,
        FrameFormat::GRAY => PixelFormat::Gray,
        FrameFormat::RAWRGB => PixelFormat::Rgb,
        FrameFormat::RAWBGR => PixelFormat::Bgr,
        FrameFormat::MJPEG => PixelFormat::Mjpeg,
    }
}

fn panic_text(p: &Box<dyn std::any::Any + Send>) -> String {
    p.downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| p.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".into())
}

/// Calls into `nokhwa` with panics turned into `Backend` errors (P1).
fn guarded<T>(what: &str, f: impl FnOnce() -> T) -> Result<T, CameraError> {
    catch_unwind(AssertUnwindSafe(f))
        .map_err(|p| CameraError::Backend(format!("{what} panicked: {}", panic_text(&p))))
}

fn open_camera(index: &CameraIndex, req: CaptureRequest) -> Result<Camera, CameraError> {
    let mut last = String::from("no format request succeeded");
    for kind in requests(req) {
        let attempt = guarded("camera open", || {
            let mut cam = Camera::new(index.clone(), RequestedFormat::new::<RgbFormat>(kind))?;
            if cam.camera_format().format() == FrameFormat::MJPEG {
                return Ok(None);
            }
            cam.open_stream()?;
            Ok::<_, nokhwa::NokhwaError>(Some(cam))
        });
        match attempt {
            Ok(Ok(Some(cam))) => return Ok(cam),
            Ok(Ok(None)) => last = "the camera only offers MJPEG, which isn't supported yet".into(),
            Ok(Err(e)) => last = e.to_string(),
            Err(e) => last = e.to_string(),
        }
    }
    Err(CameraError::Backend(last))
}

fn capture_thread(
    index: CameraIndex,
    req: CaptureRequest,
    epoch: Instant,
    slot: &Slot,
    stop: &AtomicBool,
    opened: mpsc::Sender<Result<CaptureFormat, CameraError>>,
) {
    let mut cam = match open_camera(&index, req) {
        Ok(c) => c,
        Err(e) => {
            let _ = opened.send(Err(e));
            return;
        }
    };
    let f = cam.camera_format();
    let _ = opened.send(Ok(CaptureFormat {
        width: f.resolution().width(),
        height: f.resolution().height(),
        fps: f.frame_rate() as f32,
    }));

    while !stop.load(Ordering::Relaxed) {
        let item = match guarded("camera read", || cam.frame()) {
            Ok(Ok(buf)) => {
                let timestamp_us = epoch.elapsed().as_micros() as u64;
                let res = buf.resolution();
                let format = pixel_format(buf.source_frame_format());
                match decode(format, res.width(), res.height(), buf.buffer()) {
                    Ok((luma, rgb)) => Ok(frame(luma, rgb, timestamp_us)),
                    // A malformed frame is skipped, not fatal; a stream of them times out.
                    Err(DecodeError::Length { .. }) => continue,
                    Err(e) => Err(CameraError::Backend(e.to_string())),
                }
            }
            Ok(Err(_)) => Err(CameraError::Disconnected),
            Err(e) => Err(e),
        };
        let fatal = item.is_err();
        if !slot.put(item) || fatal {
            break;
        }
    }
    let _ = guarded("camera close", || cam.stop_stream());
    let _ = guarded("camera drop", || drop(cam));
}

fn frame(luma: GrayFrame, rgb: RgbFrame, timestamp_us: u64) -> CameraFrame {
    CameraFrame {
        luma,
        rgb: Some(rgb),
        timestamp_us,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuyv_is_requested_first_and_driver_default_last() {
        let r = requests(CaptureRequest::default());
        assert!(
            matches!(r[0], RequestedFormatType::Closest(f) if f.format() == FrameFormat::YUYV
            && f.resolution() == Resolution::new(640, 480) && f.frame_rate() == 30)
        );
        assert!(matches!(r.last(), Some(RequestedFormatType::None)));
        let nv12 = r.iter().any(
            |k| matches!(k, RequestedFormatType::Closest(f) if f.format() == FrameFormat::NV12),
        );
        assert_eq!(nv12, !cfg!(target_os = "macos"));
    }

    #[test]
    fn every_backend_format_maps_and_only_mjpeg_is_refused() {
        for f in [
            FrameFormat::YUYV,
            FrameFormat::NV12,
            FrameFormat::GRAY,
            FrameFormat::RAWRGB,
            FrameFormat::RAWBGR,
            FrameFormat::MJPEG,
        ] {
            let p = pixel_format(f);
            assert_eq!(
                pixels::expected_len(p, 2, 2).is_none(),
                f == FrameFormat::MJPEG
            );
        }
    }

    #[test]
    fn panics_become_backend_errors() {
        let r: Result<(), _> = guarded("camera open", || panic!("format negotiation"));
        assert_eq!(
            r,
            Err(CameraError::Backend(
                "camera open panicked: format negotiation".into()
            ))
        );
    }

    #[test]
    fn next_frame_before_open_is_an_error_not_a_hang() {
        let mut c = NokhwaCamera::new(None, Instant::now());
        assert!(matches!(c.next_frame(), Err(CameraError::Backend(_))));
        c.close();
    }
}
