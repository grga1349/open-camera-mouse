//! Frame recorder for diagnostics (task 0008 D7): writes the raw camera frames and the
//! controller's events, so the Phase 4 tracker thresholds can be re-measured on real footage
//! (open risk: all tracker evidence so far is synthetic).
//!
//! Off unless the `OCM_RECORD=<folder>` environment variable is set. The folder gets:
//! - `frame_000001.pgm`, … — raw luma (binary PGM, native camera orientation, not mirrored);
//! - `frames.csv` — `seq,timestamp_us,width,height`;
//! - `events.csv` — `now_us,event` (state changes, selections in display-space frame
//!   coordinates, pointer owner changes), for replaying a session's selections.
//!
//! Recordings show the user's face: they stay on this machine and are never committed.
//! On the first write error the recorder stops and says so on stderr.

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use app_core::Event;
use app_core::ports::CameraFrame;

#[derive(Debug)]
pub struct Recorder {
    dir: PathBuf,
    seq: u64,
    frames: BufWriter<File>,
    events: BufWriter<File>,
    failed: bool,
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
        fs::create_dir_all(dir)?;
        let mut frames = BufWriter::new(File::create(dir.join("frames.csv"))?);
        writeln!(frames, "seq,timestamp_us,width,height")?;
        let mut events = BufWriter::new(File::create(dir.join("events.csv"))?);
        writeln!(events, "now_us,event")?;
        Ok(Self {
            dir: dir.to_path_buf(),
            seq: 0,
            frames,
            events,
            failed: false,
        })
    }

    pub fn frame(&mut self, f: &CameraFrame) {
        if self.failed {
            return;
        }
        self.seq += 1;
        let r = self.write_frame(f);
        self.check(r);
    }

    pub fn events(&mut self, now_us: u64, events: &[Event]) {
        if self.failed || events.is_empty() {
            return;
        }
        let r = events.iter().try_for_each(|e| {
            // Debug text has commas inside braces; quote it for CSV.
            let text = format!("{e:?}").replace('"', "'");
            writeln!(self.events, "{now_us},\"{text}\"")
        });
        let r = r.and_then(|()| self.events.flush());
        self.check(r);
    }

    fn write_frame(&mut self, f: &CameraFrame) -> io::Result<()> {
        let (w, h) = (f.luma.width(), f.luma.height());
        let path = self.dir.join(format!("frame_{:06}.pgm", self.seq));
        let mut out = BufWriter::new(File::create(path)?);
        write!(out, "P5\n{w} {h}\n255\n")?;
        out.write_all(f.luma.as_bytes())?;
        out.flush()?;
        writeln!(self.frames, "{},{},{w},{h}", self.seq, f.timestamp_us)?;
        self.frames.flush()
    }

    fn check(&mut self, r: io::Result<()>) {
        if let Err(e) = r {
            self.failed = true;
            eprintln!("Recording stopped ({}): {e}", self.dir.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::ports::{GrayFrame, Point2};

    #[test]
    fn writes_pgm_frames_and_csv_logs() {
        let dir = tempfile::tempdir().unwrap();
        let mut r = Recorder::create(dir.path()).unwrap();
        let luma = GrayFrame::from_vec(4, 2, vec![0, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        r.frame(&CameraFrame {
            luma,
            rgb: None,
            timestamp_us: 42,
        });
        r.events(
            7,
            &[Event::TargetSelected {
                at: Point2::new(1.0, 2.0),
            }],
        );
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
    }
}
