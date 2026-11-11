//! Synthetic fixture generator and metrics for tracker validation (`tracking-validation`
//! skill, BEHAVIOR_SPEC §8).
//!
//! Ground truth comes from the scene transform, never from a tracker. Scenes are defined in
//! *reference units* (1 unit = 1 px at 640 wide), so one fixture is the same motion, as a
//! fraction of frame width, at every resolution. Everything is seeded and deterministic.
#![allow(dead_code)]

use tracking_core::{GrayFrame, Point2, TrackStatus, Tracker, downsample_to_width};

// ---------------------------------------------------------------------------------------
// Deterministic randomness and texture.

/// SplitMix64. Small, seedable, platform-independent.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Standard normal (Box–Muller).
    pub fn normal(&mut self) -> f64 {
        let u1 = self.unit().max(1e-12);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

fn lattice(seed: u64, ix: i64, iy: i64) -> f64 {
    let mut r = Rng::new(
        seed ^ (ix as u64).wrapping_mul(0x632B_E59B_D9B4_E019)
            ^ (iy as u64).wrapping_mul(0x8CB9_2BA7_2F3D_8DD7),
    );
    r.unit()
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Smooth value noise in [0, 1], cell size `cell` (reference units).
fn value_noise(seed: u64, x: f64, y: f64, cell: f64) -> f64 {
    let (gx, gy) = (x / cell, y / cell);
    let (x0, y0) = (gx.floor(), gy.floor());
    let (fx, fy) = (fade(gx - x0), fade(gy - y0));
    let (ix, iy) = (x0 as i64, y0 as i64);
    let a = lattice(seed, ix, iy);
    let b = lattice(seed, ix + 1, iy);
    let c = lattice(seed, ix, iy + 1);
    let d = lattice(seed, ix + 1, iy + 1);
    let top = a + (b - a) * fx;
    let bot = c + (d - c) * fx;
    top + (bot - top) * fy
}

/// Fractal value noise, 4 octaves, normalised to [0, 1].
pub fn texture(seed: u64, x: f64, y: f64, cell: f64) -> f64 {
    let (mut sum, mut amp, mut norm, mut c) = (0.0, 1.0, 0.0, cell);
    for o in 0..4 {
        sum += amp * value_noise(seed.wrapping_add(o * 7919), x, y, c);
        norm += amp;
        amp *= 0.5;
        c *= 0.5;
    }
    sum / norm
}

// ---------------------------------------------------------------------------------------
// Scene description.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Surface {
    /// Fractal texture: `base + contrast · texture(seed, cell)`.
    Texture {
        seed: u64,
        cell: f64,
        base: f64,
        contrast: f64,
    },
    /// Sinusoidal grating along x with the given period (a repeated-texture distractor).
    Grating {
        period: f64,
        base: f64,
        contrast: f64,
    },
    Flat(f64),
}

impl Surface {
    fn eval(&self, x: f64, y: f64) -> f64 {
        match *self {
            Surface::Texture {
                seed,
                cell,
                base,
                contrast,
            } => base + contrast * texture(seed, x, y, cell),
            Surface::Grating {
                period,
                base,
                contrast,
            } => base + contrast * 0.5 * (1.0 + (std::f64::consts::TAU * x / period).sin()),
            Surface::Flat(v) => v,
        }
    }
}

/// Per-frame scene state. Positions/sizes in reference units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub center: (f64, f64),
    pub angle_rad: f64,
    pub scale: f64,
    pub gain: f64,
    pub bias: f64,
    /// Occluder rectangle `(x0, y0, x1, y1)`, drawn over the target.
    pub occluder: Option<(f64, f64, f64, f64)>,
}

impl Pose {
    pub fn at(center: (f64, f64)) -> Self {
        Self {
            center,
            angle_rad: 0.0,
            scale: 1.0,
            gain: 1.0,
            bias: 0.0,
            occluder: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub name: &'static str,
    pub description: &'static str,
    pub frames: usize,
    pub background: Surface,
    pub target: Surface,
    pub occluder_surface: Surface,
    /// Target disc radius.
    pub radius: f64,
    /// Selected point relative to the target centre, in target-local coordinates.
    pub select_offset: (f64, f64),
    pub noise_sigma: f64,
    /// Motion-blur sub-samples between consecutive poses (1 = none).
    pub blur_samples: usize,
    pub pose: fn(usize) -> Pose,
    /// Frames to repeat verbatim (a stalled camera): `(start, len)`.
    pub stall: Option<(usize, usize)>,
    pub seed: u64,
}

impl Scene {
    /// Ground-truth position of the selected point at frame `t` (reference units).
    pub fn truth_ref(&self, t: usize) -> (f64, f64) {
        let p = (self.pose)(self.source_frame(t));
        let (ox, oy) = self.select_offset;
        let (s, c) = p.angle_rad.sin_cos();
        (
            p.center.0 + p.scale * (c * ox - s * oy),
            p.center.1 + p.scale * (s * ox + c * oy),
        )
    }

    /// Whether the selected point is visible at frame `t`: not occluded and inside the frame.
    pub fn visible(&self, t: usize, ref_w: f64, ref_h: f64) -> bool {
        let (x, y) = self.truth_ref(t);
        let inside = x >= 0.0 && y >= 0.0 && x <= ref_w && y <= ref_h;
        let occluded = (self.pose)(self.source_frame(t))
            .occluder
            .is_some_and(|(x0, y0, x1, y1)| x >= x0 && x <= x1 && y >= y0 && y <= y1);
        inside && !occluded
    }

    fn source_frame(&self, t: usize) -> usize {
        match self.stall {
            Some((start, len)) if t >= start && t < start + len => start,
            _ => t,
        }
    }

    fn clean_value(&self, p: &Pose, x: f64, y: f64) -> f64 {
        if let Some((x0, y0, x1, y1)) = p.occluder
            && x >= x0
            && x <= x1
            && y >= y0
            && y <= y1
        {
            return self.occluder_surface.eval(x, y);
        }
        let (dx, dy) = (x - p.center.0, y - p.center.1);
        let (s, c) = p.angle_rad.sin_cos();
        let (lx, ly) = ((c * dx + s * dy) / p.scale, (-s * dx + c * dy) / p.scale);
        let r = (lx * lx + ly * ly).sqrt();
        let bg = self.background.eval(x, y);
        // Soft 2-unit edge so the disc boundary is band-limited.
        let w = ((self.radius - r) / 2.0 + 0.5).clamp(0.0, 1.0);
        if w == 0.0 {
            bg
        } else {
            bg + w * (self.target.eval(lx + 1000.0, ly + 1000.0) - bg)
        }
    }

    /// Renders the full sequence at `width × height`, then (if wider than 640) box-downsamples
    /// to ≤ 640 wide exactly as the app does (ADR-0008).
    pub fn render(&self, width: u32, height: u32) -> Sequence {
        let k = f64::from(width) / 640.0; // device px per reference unit
        let mut rng = Rng::new(self.seed);
        let mut frames = Vec::with_capacity(self.frames);
        let mut cache: Option<(usize, Vec<f64>)> = None;
        for t in 0..self.frames {
            let src = self.source_frame(t);
            let clean = match &cache {
                Some((s, buf)) if *s == src || self.is_static_between(*s, src) => buf.clone(),
                _ => {
                    let buf = self.render_clean(src, width, height, k);
                    cache = Some((src, buf.clone()));
                    buf
                }
            };
            let pose = (self.pose)(src);
            let data: Vec<u8> = clean
                .iter()
                .map(|&v| {
                    let n = if self.noise_sigma > 0.0 {
                        self.noise_sigma * rng.normal()
                    } else {
                        0.0
                    };
                    (pose.gain * v + pose.bias + n).round().clamp(0.0, 255.0) as u8
                })
                .collect();
            frames.push(downsample_to_width(
                GrayFrame::from_vec(width, height, data).unwrap(),
                640,
            ));
        }
        let (tw, th) = (frames[0].width(), frames[0].height());
        let f = f64::from(width / tw); // integer downsample factor (1 if none)
        // Reference → tracker px: device px p_d = ref·k; after box factor f, p = (p_d + 0.5)/f − 0.5.
        let to_px = |(x, y): (f64, f64)| {
            Point2::new(
                ((x * k + 0.5) / f - 0.5) as f32,
                ((y * k + 0.5) / f - 0.5) as f32,
            )
        };
        let (ref_w, ref_h) = (640.0, f64::from(height) / k);
        Sequence {
            name: self.name,
            device_size: (width, height),
            size: (tw, th),
            truth: (0..self.frames).map(|t| to_px(self.truth_ref(t))).collect(),
            visible: (0..self.frames)
                .map(|t| self.visible(t, ref_w, ref_h))
                .collect(),
            frames,
        }
    }

    fn is_static_between(&self, a: usize, b: usize) -> bool {
        let (pa, pb) = ((self.pose)(a), (self.pose)(b));
        self.blur_samples <= 1
            && pa.center == pb.center
            && pa.angle_rad == pb.angle_rad
            && pa.scale == pb.scale
            && pa.occluder == pb.occluder
    }

    fn render_clean(&self, t: usize, width: u32, height: u32, k: f64) -> Vec<f64> {
        let samples = self.blur_samples.max(1);
        let poses: Vec<Pose> = (0..samples)
            .map(|s| {
                let cur = (self.pose)(t);
                if samples == 1 || t == 0 {
                    return cur;
                }
                // Exposure centred on the frame time: samples span ±½ of the last frame's
                // motion around the current pose, so ground truth (the current pose) is the
                // blur's centre, not its trailing edge.
                let prev = (self.pose)(t - 1);
                let a = (s as f64 + 0.5) / samples as f64 - 0.5;
                Pose {
                    center: (
                        cur.center.0 + a * (cur.center.0 - prev.center.0),
                        cur.center.1 + a * (cur.center.1 - prev.center.1),
                    ),
                    ..cur
                }
            })
            .collect();
        let mut out = Vec::with_capacity((width * height) as usize);
        for py in 0..height {
            for px in 0..width {
                let (x, y) = (f64::from(px) / k, f64::from(py) / k);
                let v =
                    poses.iter().map(|p| self.clean_value(p, x, y)).sum::<f64>() / samples as f64;
                out.push(v);
            }
        }
        out
    }
}

/// A rendered sequence in tracker pixel coordinates.
#[derive(Debug, Clone)]
pub struct Sequence {
    pub name: &'static str,
    pub device_size: (u32, u32),
    pub size: (u32, u32),
    pub frames: Vec<GrayFrame>,
    pub truth: Vec<Point2>,
    pub visible: Vec<bool>,
}

// ---------------------------------------------------------------------------------------
// The corpus (BEHAVIOR_SPEC §8).

const BG: Surface = Surface::Texture {
    seed: 11,
    cell: 48.0,
    base: 40.0,
    contrast: 150.0,
};
const TARGET: Surface = Surface::Texture {
    seed: 23,
    cell: 14.0,
    base: 30.0,
    contrast: 200.0,
};
const OCCLUDER: Surface = Surface::Texture {
    seed: 37,
    cell: 30.0,
    base: 60.0,
    contrast: 120.0,
};
const C0: (f64, f64) = (320.0, 180.0);

fn base(
    name: &'static str,
    description: &'static str,
    frames: usize,
    pose: fn(usize) -> Pose,
) -> Scene {
    Scene {
        name,
        description,
        frames,
        background: BG,
        target: TARGET,
        occluder_surface: OCCLUDER,
        radius: 45.0,
        select_offset: (12.0, -8.0),
        noise_sigma: 2.0,
        blur_samples: 1,
        pose,
        stall: None,
        seed: 0xC0FFEE,
    }
}

fn sway(t: usize) -> (f64, f64) {
    let t = t as f64;
    (
        C0.0 + 110.0 * (t * 0.07).sin(),
        C0.1 + 50.0 * (t * 0.045).sin(),
    )
}

fn edge_path(t: usize, dir: (f64, f64), reach: f64) -> Pose {
    // Out over 40 frames, back over 40.
    let s = if t <= 40 {
        t as f64 / 40.0
    } else {
        (80 - t.min(80)) as f64 / 40.0
    };
    Pose::at((C0.0 + dir.0 * reach * s, C0.1 + dir.1 * reach * s))
}

pub fn corpus() -> Vec<Scene> {
    let mut v = vec![
        base(
            "translate_int",
            "integer translation +2/−1 ref px per frame",
            60,
            |t| Pose::at((C0.0 + 2.0 * t as f64, C0.1 - t as f64)),
        ),
        base(
            "translate_subpx",
            "sub-pixel translation 0.37/0.23 ref px per frame",
            60,
            |t| Pose::at((C0.0 + 0.37 * t as f64, C0.1 + 0.23 * t as f64)),
        ),
        base(
            "head_sway",
            "smooth sinusoidal head motion (±110, ±50 ref px)",
            90,
            |t| Pose::at(sway(t)),
        ),
        base(
            "fast_40px",
            "40 ref px/frame burst for 5 frames (crosses the 35 px guard)",
            30,
            |t| {
                let k = t.clamp(10, 15) - 10;
                Pose::at((C0.0 - 100.0 + 40.0 * k as f64, C0.1))
            },
        ),
        base(
            "brightness_step",
            "gain 1.0 → 0.7 at frame 30, slow sway",
            60,
            |t| Pose {
                gain: if t < 30 { 1.0 } else { 0.7 },
                ..Pose::at(sway(t / 3))
            },
        ),
        base(
            "brightness_ramp",
            "gain ramps 1.0 → 1.4 over 60 frames",
            60,
            |t| Pose {
                gain: 1.0 + 0.4 * t as f64 / 60.0,
                bias: -20.0 * t as f64 / 60.0,
                ..Pose::at(sway(t / 3))
            },
        ),
        base(
            "contrast_drop",
            "contrast halves around mid-grey at frame 30",
            60,
            |t| {
                let c = if t < 30 { 1.0 } else { 0.5 };
                Pose {
                    gain: c,
                    bias: 128.0 * (1.0 - c),
                    ..Pose::at(sway(t / 3))
                }
            },
        ),
        base(
            "occlusion_partial",
            "occluder covers the left half of the target, frames 30–49",
            70,
            |t| {
                let mut p = Pose::at((C0.0 + 0.5 * t as f64, C0.1));
                if (30..50).contains(&t) {
                    p.occluder = Some((
                        p.center.0 - 60.0,
                        p.center.1 - 60.0,
                        p.center.0,
                        p.center.1 + 60.0,
                    ));
                }
                p
            },
        ),
        base(
            "occlusion_full",
            "occluder covers the whole target, frames 30–49",
            70,
            |t| {
                let mut p = Pose::at((C0.0 + 0.5 * t as f64, C0.1));
                if (30..50).contains(&t) {
                    p.occluder = Some((
                        p.center.0 - 70.0,
                        p.center.1 - 70.0,
                        p.center.0 + 70.0,
                        p.center.1 + 70.0,
                    ));
                }
                p
            },
        ),
        base(
            "rotate_15",
            "in-plane rotation 0 → +15° → −15° (head roll)",
            60,
            |t| Pose {
                angle_rad: (15.0f64).to_radians() * (t as f64 * std::f64::consts::TAU / 60.0).sin(),
                ..Pose::at(C0)
            },
        ),
        base(
            "scale_10",
            "scale 1.0 → 1.1 → 0.9 (lean in/out)",
            60,
            |t| Pose {
                scale: 1.0 + 0.1 * (t as f64 * std::f64::consts::TAU / 60.0).sin(),
                ..Pose::at(C0)
            },
        ),
        base(
            "edge_left",
            "target leaves through the left edge and returns",
            81,
            |t| edge_path(t, (-1.0, 0.0), 360.0),
        ),
        base(
            "edge_right",
            "target leaves through the right edge and returns",
            81,
            |t| edge_path(t, (1.0, 0.0), 360.0),
        ),
        base(
            "edge_top",
            "target leaves through the top edge and returns",
            81,
            |t| edge_path(t, (0.0, -1.0), 200.0),
        ),
        base(
            "edge_bottom",
            "target leaves through the bottom edge and returns",
            81,
            |t| edge_path(t, (0.0, 1.0), 340.0),
        ),
        base(
            "stall",
            "camera stalls for 10 frames, then resumes mid-sway",
            60,
            |t| Pose::at(sway(t)),
        ),
        base(
            "static_60s",
            "static target for 60 s at 30 fps (drift)",
            1800,
            |_| Pose::at(C0),
        ),
    ];
    v.iter_mut().find(|s| s.name == "stall").unwrap().stall = Some((20, 10));

    let mut blur = base(
        "motion_blur",
        "head sway ×2 speed with 5-sample motion blur",
        45,
        |t| Pose::at(sway(2 * t)),
    );
    blur.blur_samples = 5;
    v.push(blur);

    let mut noise = base(
        "sensor_noise",
        "head sway with σ = 10 sensor noise",
        90,
        |t| Pose::at(sway(t)),
    );
    noise.noise_sigma = 10.0;
    v.push(noise);

    let mut flat = base(
        "textureless",
        "flat target on flat background, σ = 2 noise (expected failure)",
        30,
        |_| Pose::at(C0),
    );
    flat.background = Surface::Flat(120.0);
    flat.target = Surface::Flat(125.0);
    v.push(flat);

    let mut grating = base(
        "repeated_texture",
        "grating target on a same-period grating background (distractor)",
        60,
        |t| Pose::at((C0.0 + 1.5 * t as f64, C0.1)),
    );
    grating.background = Surface::Grating {
        period: 16.0,
        base: 60.0,
        contrast: 120.0,
    };
    grating.target = Surface::Grating {
        period: 16.0,
        base: 70.0,
        contrast: 120.0,
    };
    v.push(grating);
    v
}

pub fn scene(name: &str) -> Scene {
    corpus()
        .into_iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no scene {name}"))
}

pub const RESOLUTIONS: [(u32, u32); 3] = [(320, 240), (640, 480), (1280, 720)];

// ---------------------------------------------------------------------------------------
// Running and metrics.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub position: Point2,
    pub status: TrackStatus,
}

/// Selects at the ground-truth point on frame 0, then steps every later frame.
pub fn run<T: Tracker>(tracker: &mut T, seq: &Sequence) -> Vec<Step> {
    tracker
        .select(&seq.frames[0], seq.truth[0])
        .expect("selection inside frame");
    let mut out = vec![Step {
        position: seq.truth[0],
        status: TrackStatus::Tracking,
    }];
    for f in &seq.frames[1..] {
        let o = tracker.step(f).expect("same-size frames");
        out.push(Step {
            position: o.position,
            status: o.status,
        });
    }
    out
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub frames: usize,
    pub visible: usize,
    /// EPE over visible frames whose status is `Tracking` (px in tracker resolution).
    pub epe_mean: f64,
    pub epe_median: f64,
    pub epe_p95: f64,
    pub epe_max: f64,
    /// Fraction of visible frames that are `Tracking` with EPE ≤ 2 % of width.
    pub tracked_ok: f64,
    /// Visible frames reported `Tracking` but more than 2 % of width off (silent wrong lock).
    pub wrong_lock: usize,
    /// Invisible frames reported `Tracking` (false tracking during occlusion / out of frame).
    pub false_tracking: usize,
    /// Frames from the first invisible frame to the first non-`Tracking` status.
    pub frames_to_detect_loss: Option<usize>,
    /// Frames from visibility returning to `Tracking` within tolerance again.
    pub frames_to_recover: Option<usize>,
    /// EPE at the last frame if `Tracking`.
    pub final_epe: Option<f64>,
    pub width: u32,
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i]
}

pub fn metrics(seq: &Sequence, steps: &[Step]) -> Metrics {
    let width = seq.size.0;
    let tol = 0.02 * f64::from(width);
    let epe = |t: usize| {
        let (p, g) = (steps[t].position, seq.truth[t]);
        f64::from((p.x - g.x).hypot(p.y - g.y))
    };
    let mut errs = Vec::new();
    let (mut ok, mut wrong, mut false_t) = (0, 0, 0);
    for (t, step) in steps.iter().enumerate() {
        let tracking = step.status == TrackStatus::Tracking;
        if seq.visible[t] {
            if tracking {
                let e = epe(t);
                errs.push(e);
                if e <= tol { ok += 1 } else { wrong += 1 }
            }
        } else if tracking {
            false_t += 1;
        }
    }
    errs.sort_by(f64::total_cmp);
    let visible = seq.visible.iter().filter(|&&v| v).count();
    let first_invisible = seq.visible.iter().position(|&v| !v);
    let frames_to_detect_loss = first_invisible.and_then(|s| {
        (s..steps.len())
            .find(|&t| steps[t].status != TrackStatus::Tracking)
            .map(|t| t - s)
    });
    let frames_to_recover = first_invisible.and_then(|s| {
        let back = (s..steps.len()).find(|&t| seq.visible[t])?;
        (back..steps.len())
            .find(|&t| steps[t].status == TrackStatus::Tracking && epe(t) <= tol)
            .map(|t| t - back)
    });
    let last = steps.len() - 1;
    Metrics {
        frames: steps.len(),
        visible,
        epe_mean: if errs.is_empty() {
            f64::NAN
        } else {
            errs.iter().sum::<f64>() / errs.len() as f64
        },
        epe_median: percentile(&errs, 0.5),
        epe_p95: percentile(&errs, 0.95),
        epe_max: errs.last().copied().unwrap_or(f64::NAN),
        tracked_ok: if visible == 0 {
            0.0
        } else {
            ok as f64 / visible as f64
        },
        wrong_lock: wrong,
        false_tracking: false_t,
        frames_to_detect_loss,
        frames_to_recover,
        final_epe: (steps[last].status == TrackStatus::Tracking).then(|| epe(last)),
        width,
    }
}
