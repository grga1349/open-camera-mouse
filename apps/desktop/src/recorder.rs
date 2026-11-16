//! Frame recorder for diagnostics and the benchmark recordings (task 0008 D7, task 0009 2b):
//! writes the raw camera frames, the controller's events, the desktop geometry and the
//! settings, so tracking and cursor behaviour can be replayed and measured offline
//! (`docs/qa/RECORDING_PROTOCOL.md`).
//!
//! Off unless the `OCM_RECORD=<folder>` environment variable is set. The folder gets:
//! - `frame_000001.pgm`, … — raw luma (binary PGM, native camera orientation, not mirrored);
//! - `frames.csv` — `seq,timestamp_us,width,height`;
//! - `events.csv` — `now_us,event` (state changes, selections in display-space frame
//!   coordinates, pointer owner changes), for replaying a session's selections;
//! - `desktop.csv` — `now_us,kind,x,y,width,height`, one `bounds` row and one `display` row
//!   per display, written whenever the desktop changes;
//! - `settings.toml` — the settings in effect (rewritten when they change).
//!
//! Files are written on a separate thread so a slow disk can't slow the engine. The queue is
//! bounded: a frame that doesn't fit is dropped, counted and reported on stderr; its `seq`
//! is then missing from `frames.csv` (gaps are visible to the replay).
//!
//! Recordings show the user's face: they stay on this machine and are never committed.
//! On the first write error the recorder stops and says so on stderr.

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::thread::JoinHandle;

use app_core::ports::{CameraFrame, Desktop, GrayFrame};
use app_core::{Event, Settings};
use platform::settings_store::SettingsStore;

/// Frames that may wait for the disk (≈ 2 s at 30 fps, ≈ 18 MB at 640×480).
const QUEUE_FRAMES: usize = 60;

enum Msg {
    Frame {
        seq: u64,
        timestamp_us: u64,
        luma: GrayFrame,
    },
    Events(String),
    Desktop(String),
    Settings(Box<Settings>),
}

#[derive(Debug)]
pub struct Recorder {
    dir: PathBuf,
    seq: u64,
    dropped: u64,
    tx: Option<SyncSender<Msg>>,
    writer: Option<JoinHandle<()>>,
}

impl Recorder {
    /// From `OCM_RECORD`, if set. A folder that can't be created is reported, not fatal.
    pub fn from_env() -> Option<Self> {
        let dir = std::env::var_os("OCM_RECORD")?;
        match Self::create(Path::new(&dir)) {
            Ok(r) => {
                eprintln!("Recording camera frames to {}", r.dir.display());
                Some(r)
            }
            Err(e) => {
                eprintln!("Can't record to {}: {e}", Path::new(&dir).display());
                None
            }
        }
    }

    pub fn create(dir: &Path) -> io::Result<Self> {
        Self::with_queue(dir, QUEUE_FRAMES)
    }

    fn with_queue(dir: &Path, queue: usize) -> io::Result<Self> {
        let files = Files::create(dir)?;
        let (tx, rx) = sync_channel(queue);
        let writer = std::thread::Builder::new()
            .name("ocm-recorder".into())
            .spawn(move || files.run(&rx))?;
        Ok(Self {
            dir: dir.to_path_buf(),
            seq: 0,
            dropped: 0,
            tx: Some(tx),
            writer: Some(writer),
        })
    }

    pub fn frame(&mut self, f: &CameraFrame) {
        self.seq += 1;
        let msg = Msg::Frame {
            seq: self.seq,
            timestamp_us: f.timestamp_us,
            luma: f.luma.clone(),
        };
        if let Some(tx) = &self.tx {
            match tx.try_send(msg) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => {
                    self.dropped += 1;
                    if self.dropped.is_power_of_two() {
                        eprintln!(
                            "Recorder: disk too slow, {} frame(s) dropped so far (last seq {})",
                            self.dropped, self.seq
                        );
                    }
                }
                // The writer stopped after an error (already reported).
                Err(TrySendError::Disconnected(_)) => self.tx = None,
            }
        }
    }

    pub fn events(&mut self, now_us: u64, events: &[Event]) {
        if events.is_empty() {
            return;
        }
        let mut text = String::new();
        for e in events {
            // Debug text has commas inside braces; quote it for CSV.
            let e = format!("{e:?}").replace('"', "'");
            text.push_str(&format!("{now_us},\"{e}\"\n"));
        }
        self.send(Msg::Events(text));
    }

    pub fn desktop(&mut self, now_us: u64, d: &Desktop) {
        let row = |kind: &str, r: &app_core::DesktopRect| {
            format!("{now_us},{kind},{},{},{},{}\n", r.x, r.y, r.width, r.height)
        };
        let mut text = row("bounds", &d.bounds);
        for r in &d.displays {
            text.push_str(&row("display", r));
        }
        self.send(Msg::Desktop(text));
    }

    pub fn settings(&mut self, s: &Settings) {
        self.send(Msg::Settings(Box::new(s.clone())));
    }

    /// Frames that didn't fit in the queue.
    #[cfg(test)]
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Small messages wait for room: they are rare and must not be lost.
    fn send(&mut self, msg: Msg) {
        if let Some(tx) = &self.tx
            && tx.send(msg).is_err()
        {
            self.tx = None;
        }
    }
}

impl Drop for Recorder {
    /// Flushes everything queued before returning.
    fn drop(&mut self) {
        self.tx = None;
        if let Some(w) = self.writer.take() {
            let _ = w.join();
        }
        if self.dropped > 0 {
            eprintln!(
                "Recorder: {} of {} frames dropped ({})",
                self.dropped,
                self.seq,
                self.dir.display()
            );
        }
    }
}

/// The writer thread's side.
struct Files {
    dir: PathBuf,
    frames: BufWriter<File>,
    events: BufWriter<File>,
    desktop: BufWriter<File>,
}

impl Files {
    fn create(dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let mut frames = BufWriter::new(File::create(dir.join("frames.csv"))?);
        writeln!(frames, "seq,timestamp_us,width,height")?;
        let mut events = BufWriter::new(File::create(dir.join("events.csv"))?);
        writeln!(events, "now_us,event")?;
        let mut desktop = BufWriter::new(File::create(dir.join("desktop.csv"))?);
        writeln!(desktop, "now_us,kind,x,y,width,height")?;
        for f in [&mut frames, &mut events, &mut desktop] {
            f.flush()?;
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            frames,
            events,
            desktop,
        })
    }

    fn run(mut self, rx: &Receiver<Msg>) {
        while let Ok(msg) = rx.recv() {
            if let Err(e) = self.write(msg) {
                // Dropping `rx` on return tells the recorder to stop sending.
                eprintln!("Recording stopped ({}): {e}", self.dir.display());
                return;
            }
        }
    }

    fn write(&mut self, msg: Msg) -> io::Result<()> {
        match msg {
            Msg::Frame {
                seq,
                timestamp_us,
                luma,
            } => {
                let (w, h) = (luma.width(), luma.height());
                let path = self.dir.join(format!("frame_{seq:06}.pgm"));
                let mut out = BufWriter::new(File::create(path)?);
                write!(out, "P5\n{w} {h}\n255\n")?;
                out.write_all(luma.as_bytes())?;
                out.flush()?;
                writeln!(self.frames, "{seq},{timestamp_us},{w},{h}")?;
                self.frames.flush()
            }
            Msg::Events(text) => {
                self.events.write_all(text.as_bytes())?;
                self.events.flush()
            }
            Msg::Desktop(text) => {
                self.desktop.write_all(text.as_bytes())?;
                self.desktop.flush()
            }
            Msg::Settings(s) => SettingsStore::at(self.dir.join("settings.toml"))
                .save(&s)
                .map(|_| ())
                .map_err(|e| io::Error::other(format!("{e:?}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::DesktopRect;
    use app_core::ports::Point2;

    fn frame(ts: u64) -> CameraFrame {
        CameraFrame {
            luma: GrayFrame::from_vec(4, 2, vec![0, 1, 2, 3, 4, 5, 6, 7]).unwrap(),
            rgb: None,
            timestamp_us: ts,
        }
    }

    #[test]
    fn writes_pgm_frames_csv_logs_desktop_and_settings() {
        let dir = tempfile::tempdir().unwrap();
        let mut r = Recorder::create(dir.path()).unwrap();
        r.frame(&frame(42));
        r.events(
            7,
            &[Event::TargetSelected {
                at: Point2::new(1.0, 2.0),
            }],
        );
        let rect = |x: f32| DesktopRect {
            x,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        r.desktop(
            9,
            &Desktop {
                bounds: DesktopRect {
                    width: 200.0,
                    ..rect(0.0)
                },
                displays: vec![rect(0.0), rect(100.0)],
                primary: Some(0),
            },
        );
        let s = Settings {
            gain_x: 7.0,
            gain_y: 7.0,
            smoothing: 0.3,
            ..Settings::default()
        };
        r.settings(&s);
        drop(r); // joins the writer: everything is on disk

        let pgm = fs::read(dir.path().join("frame_000001.pgm")).unwrap();
        assert_eq!(&pgm[..11], b"P5\n4 2\n255\n");
        assert_eq!(&pgm[11..], &[0, 1, 2, 3, 4, 5, 6, 7]);
        let frames = fs::read_to_string(dir.path().join("frames.csv")).unwrap();
        assert_eq!(frames, "seq,timestamp_us,width,height\n1,42,4,2\n");
        let events = fs::read_to_string(dir.path().join("events.csv")).unwrap();
        assert!(
            events.starts_with("now_us,event\n7,\"TargetSelected"),
            "{events}"
        );
        let desktop = fs::read_to_string(dir.path().join("desktop.csv")).unwrap();
        assert_eq!(
            desktop,
            "now_us,kind,x,y,width,height\n9,bounds,0,0,200,50\n\
             9,display,0,0,100,50\n9,display,100,0,100,50\n"
        );
        let mut store = SettingsStore::at(dir.path().join("settings.toml"));
        assert_eq!(store.load().settings, s);
    }

    #[test]
    fn a_full_queue_drops_frames_without_blocking_and_keeps_seq_gaps() {
        let dir = tempfile::tempdir().unwrap();
        // Queue of 0: a frame is only taken while the writer is waiting, so a burst drops.
        let mut r = Recorder::with_queue(dir.path(), 0).unwrap();
        for ts in 0..200 {
            r.frame(&frame(ts));
        }
        let (sent, dropped) = (r.seq, r.dropped());
        assert_eq!(sent, 200);
        assert!(dropped > 0, "a burst into a zero queue must drop");
        drop(r);
        let frames = fs::read_to_string(dir.path().join("frames.csv")).unwrap();
        let written: Vec<u64> = frames
            .lines()
            .skip(1)
            .map(|l| l.split(',').next().unwrap().parse().unwrap())
            .collect();
        assert_eq!(written.len() as u64, sent - dropped);
        assert!(written.windows(2).all(|w| w[0] < w[1]), "seq stays ordered");
        for seq in &written {
            assert!(dir.path().join(format!("frame_{seq:06}.pgm")).exists());
        }
    }
}
