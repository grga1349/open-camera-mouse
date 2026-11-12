//! Settings file on disk (BEHAVIOR_SPEC R-A5, ADR-0012). The pure `app_core::Settings`
//! defines the fields and validation; this module owns the file: where it lives, the TOML
//! format, the schema version, atomic writes, and what happens when the file is bad.
//!
//! Loading never fails. Every problem falls back to something usable plus a
//! [`SettingsNotice`] the UI shows:
//!
//! | File | Result | Store |
//! |---|---|---|
//! | missing | defaults, no notice (first run) | writable |
//! | not TOML / wrong types / no or old `schema_version` | defaults, `Corrupt` (file moved aside to `settings.toml.corrupt`) | writable |
//! | `schema_version` newer than this build | defaults, `NewerSchema` | **read-only**, so an older build never overwrites a newer file |
//! | some values out of range | those fields reset, `Repaired` | writable |
//! | can't be read (permissions, I/O) | defaults, `Unreadable` | read-only |

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use app_core::Settings;

/// File name inside the config directory.
pub const FILE_NAME: &str = "settings.toml";
const SCHEMA_KEY: &str = "schema_version";

/// Why the loaded settings aren't exactly what the file said. Shown to the user once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsNotice {
    /// The file couldn't be parsed. `moved_to` is where the old file was kept, if moving it
    /// worked.
    Corrupt {
        reason: String,
        moved_to: Option<PathBuf>,
    },
    /// Written by a newer version of the app. Not overwritten while this build runs.
    NewerSchema { found: i64, supported: u32 },
    /// These fields were out of range and were reset to their defaults.
    Repaired { fields: Vec<&'static str> },
    /// The file exists but couldn't be read. Not overwritten while this build runs.
    Unreadable { reason: String },
}

impl fmt::Display for SettingsNotice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corrupt { moved_to, .. } => {
                write!(f, "Your settings file was damaged, so defaults are in use.")?;
                if let Some(p) = moved_to {
                    write!(f, " The old file was kept as {}.", p.display())?;
                }
                Ok(())
            }
            Self::NewerSchema { .. } => write!(
                f,
                "Your settings were saved by a newer version of Open Camera Mouse, so \
                 defaults are in use. Changes made now won't be saved."
            ),
            Self::Repaired { fields } => write!(
                f,
                "Some settings were out of range and were reset: {}.",
                fields.join(", ")
            ),
            Self::Unreadable { .. } => write!(
                f,
                "Your settings file couldn't be read, so defaults are in use. Changes made \
                 now won't be saved."
            ),
        }
    }
}

/// Result of [`SettingsStore::load`]: always valid settings, plus a notice if anything was
/// off.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    pub settings: Settings,
    pub notice: Option<SettingsNotice>,
}

/// What [`SettingsStore::save`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saved {
    Written,
    /// The store is read-only (newer-schema or unreadable file); nothing was written.
    Skipped,
}

/// Writing the settings file failed.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("settings could not be encoded: {0}")]
    Encode(#[from] toml::ser::Error),
    #[error("settings could not be written to {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
}

/// One settings file. Create with [`at`](Self::at) (tests, explicit path) or
/// [`in_user_config_dir`](Self::in_user_config_dir).
#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: PathBuf,
    read_only: bool,
}

impl SettingsStore {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            read_only: false,
        }
    }

    /// `settings.toml` in the per-user config directory (see [`config_dir`]). `None` if the
    /// OS gives no usable home or config directory.
    pub fn in_user_config_dir() -> Option<Self> {
        let os = if cfg!(target_os = "macos") {
            Os::Mac
        } else if cfg!(target_os = "windows") {
            Os::Windows
        } else {
            Os::Unix
        };
        let dir = config_dir(os, |k| std::env::var_os(k), std::env::home_dir())?;
        Some(Self::at(dir.join(FILE_NAME)))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// True after loading a file this build must not overwrite.
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Reads the file. Never fails; see the module table for every fallback.
    pub fn load(&mut self) -> Loaded {
        self.read_only = false;
        let text = match fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return defaults(None),
            Err(e) => {
                self.read_only = true;
                return defaults(Some(SettingsNotice::Unreadable {
                    reason: e.to_string(),
                }));
            }
        };
        match decode(&text) {
            Ok((settings, fields)) => Loaded {
                settings,
                notice: (!fields.is_empty()).then_some(SettingsNotice::Repaired { fields }),
            },
            Err(Decode::Newer(found)) => {
                self.read_only = true;
                defaults(Some(SettingsNotice::NewerSchema {
                    found,
                    supported: Settings::SCHEMA_VERSION,
                }))
            }
            Err(Decode::Corrupt(reason)) => {
                let aside = self.path.with_extension("toml.corrupt");
                let moved_to = fs::rename(&self.path, &aside).ok().map(|()| aside);
                defaults(Some(SettingsNotice::Corrupt { reason, moved_to }))
            }
        }
    }

    /// Writes `settings` (validated by the caller, as `Controller` does) atomically: a
    /// temporary file next to the target, then a rename, so a crash mid-write leaves the
    /// old file intact. Creates the directory if needed.
    pub fn save(&self, settings: &Settings) -> Result<Saved, SaveError> {
        if self.read_only {
            return Ok(Saved::Skipped);
        }
        let text = encode(settings)?;
        let io_err = |source| SaveError::Io {
            path: self.path.clone(),
            source,
        };
        if let Some(dir) = self.path.parent().filter(|d| !d.as_os_str().is_empty()) {
            fs::create_dir_all(dir).map_err(io_err)?;
        }
        let tmp = self.path.with_extension("toml.tmp");
        fs::write(&tmp, text).map_err(io_err)?;
        // `rename` replaces an existing target on macOS, Linux and Windows.
        fs::rename(&tmp, &self.path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            io_err(e)
        })?;
        Ok(Saved::Written)
    }
}

/// Which per-user config convention applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    Mac,
    Windows,
    /// Linux and other Unix: XDG Base Directory.
    Unix,
}

/// The app's config directory, from the environment and home directory passed in (so every
/// OS's rule is testable on any OS). Hand-written instead of the `directories` crate, whose
/// `option-ext` dependency is MPL-2.0 (ADR-0012).
///
/// - macOS: `$HOME/Library/Application Support/Open Camera Mouse`
/// - Windows: `%APPDATA%\Open Camera Mouse` (roaming, like `directories`' `config_dir`)
/// - Unix: `$XDG_CONFIG_HOME/open-camera-mouse`, else `$HOME/.config/open-camera-mouse`
///
/// Relative values are ignored (the XDG spec requires that; it's also the safe choice for
/// the others), so a relative path never puts settings in the working directory.
fn config_dir(
    os: Os,
    env: impl Fn(&str) -> Option<std::ffi::OsString>,
    home: Option<PathBuf>,
) -> Option<PathBuf> {
    let abs = |p: PathBuf| p.is_absolute().then_some(p);
    let home = home.and_then(abs);
    match os {
        Os::Mac => Some(
            home?
                .join("Library")
                .join("Application Support")
                .join("Open Camera Mouse"),
        ),
        Os::Windows => Some(
            env("APPDATA")
                .map(PathBuf::from)
                .and_then(abs)?
                .join("Open Camera Mouse"),
        ),
        Os::Unix => {
            let base = env("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .and_then(abs)
                .or_else(|| Some(home?.join(".config")))?;
            Some(base.join("open-camera-mouse"))
        }
    }
}

fn defaults(notice: Option<SettingsNotice>) -> Loaded {
    Loaded {
        settings: Settings::default(),
        notice,
    }
}

enum Decode {
    Corrupt(String),
    Newer(i64),
}

/// `schema_version` first, then the settings fields at the top level.
fn encode(settings: &Settings) -> Result<String, toml::ser::Error> {
    let body = toml::to_string(settings)?;
    Ok(format!(
        "{SCHEMA_KEY} = {}\n{body}",
        Settings::SCHEMA_VERSION
    ))
}

fn decode(text: &str) -> Result<(Settings, Vec<&'static str>), Decode> {
    let mut table: toml::Table = text
        .parse()
        .map_err(|e: toml::de::Error| Decode::Corrupt(e.message().to_owned()))?;
    let version = match table.remove(SCHEMA_KEY) {
        Some(toml::Value::Integer(v)) => v,
        Some(_) => return Err(Decode::Corrupt(format!("{SCHEMA_KEY} is not an integer"))),
        None => return Err(Decode::Corrupt(format!("{SCHEMA_KEY} is missing"))),
    };
    let current = i64::from(Settings::SCHEMA_VERSION);
    if version > current {
        return Err(Decode::Newer(version));
    }
    if version < current {
        // No older schema has ever shipped; a migration goes here when one does.
        return Err(Decode::Corrupt(format!(
            "{SCHEMA_KEY} {version} is not supported"
        )));
    }
    let settings: Settings = table
        .try_into()
        .map_err(|e: toml::de::Error| Decode::Corrupt(e.message().to_owned()))?;
    Ok(settings.repaired())
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::Theme;

    fn store(dir: &tempfile::TempDir) -> SettingsStore {
        SettingsStore::at(dir.path().join("cfg").join(FILE_NAME))
    }

    fn write(store: &SettingsStore, text: &str) {
        fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        fs::write(store.path(), text).unwrap();
    }

    fn changed() -> Settings {
        Settings {
            gain_x: 8.5,
            link_gain: false,
            gain_y: 4.0,
            smoothing: 0.3,
            camera: Some("FaceTime HD Camera".into()),
            loss_recovery: true,
            theme: Theme::Paper,
            text_scale: 1.2,
            ..Settings::default()
        }
    }

    #[test]
    fn missing_file_gives_defaults_without_a_notice() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        assert_eq!(
            s.load(),
            Loaded {
                settings: Settings::default(),
                notice: None
            }
        );
        assert!(!s.is_read_only());
    }

    #[test]
    fn save_then_load_round_trips_every_field() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        let mut expected = changed();
        expected.dwell.enabled = true;
        expected.dwell.dwell_s = 1.5;
        expected.limits.right = 40;
        assert_eq!(s.save(&expected).unwrap(), Saved::Written);
        let text = fs::read_to_string(s.path()).unwrap();
        assert!(text.starts_with("schema_version = 1\n"), "{text}");
        assert!(text.contains("theme = \"paper\""), "{text}");
        assert!(!s.path().with_extension("toml.tmp").exists());
        let loaded = s.load();
        assert_eq!(loaded.notice, None);
        assert_eq!(loaded.settings, expected);
    }

    #[test]
    fn default_camera_is_omitted_and_round_trips_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        s.save(&Settings::default()).unwrap();
        assert!(!fs::read_to_string(s.path()).unwrap().contains("camera"));
        assert_eq!(s.load().settings, Settings::default());
    }

    #[test]
    fn missing_fields_take_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        write(
            &s,
            "schema_version = 1\ntheme = \"paper\"\n[dwell]\nenabled = true\n",
        );
        let loaded = s.load();
        assert_eq!(loaded.notice, None);
        assert_eq!(loaded.settings.theme, Theme::Paper);
        assert!(loaded.settings.dwell.enabled);
        assert_eq!(loaded.settings.dwell.dwell_s, 1.0);
        assert_eq!(loaded.settings.gain_x, 6.0);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        write(&s, "schema_version = 1\nsomething_old = 3\ngain_x = 7.0\n");
        let loaded = s.load();
        assert_eq!(loaded.notice, None);
        assert_eq!(loaded.settings.gain_x, 7.0);
    }

    #[test]
    fn corrupt_file_gives_defaults_and_is_moved_aside() {
        for text in [
            "this is { not toml",
            "schema_version = 1\ngain_x = \"fast\"\n",
            "schema_version = 1\ntheme = \"neon\"\n",
            "gain_x = 7.0\n",
            "schema_version = \"1\"\n",
            "schema_version = 0\n",
            "",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let mut s = store(&dir);
            write(&s, text);
            let loaded = s.load();
            assert_eq!(loaded.settings, Settings::default(), "{text:?}");
            let aside = s.path().with_extension("toml.corrupt");
            match &loaded.notice {
                Some(SettingsNotice::Corrupt { moved_to, .. }) => {
                    assert_eq!(moved_to.as_deref(), Some(aside.as_path()));
                }
                other => panic!("{text:?}: {other:?}"),
            }
            assert_eq!(fs::read_to_string(&aside).unwrap(), text, "old file kept");
            assert!(!s.path().exists());
            assert!(!s.is_read_only(), "a corrupt file may be replaced");
            assert_eq!(s.save(&changed()).unwrap(), Saved::Written);
            assert_eq!(s.load().settings, changed());
        }
    }

    #[test]
    fn newer_schema_gives_defaults_and_is_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        let newer = "schema_version = 2\ngain_x = 9.0\nnew_thing = true\n";
        write(&s, newer);
        let loaded = s.load();
        assert_eq!(loaded.settings, Settings::default());
        assert_eq!(
            loaded.notice,
            Some(SettingsNotice::NewerSchema {
                found: 2,
                supported: 1
            })
        );
        assert!(s.is_read_only());
        assert_eq!(s.save(&changed()).unwrap(), Saved::Skipped);
        assert_eq!(fs::read_to_string(s.path()).unwrap(), newer);
    }

    #[test]
    fn out_of_range_values_are_repaired_field_by_field() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        write(
            &s,
            "schema_version = 1\ngain_x = 50.0\nsmoothing = 0.4\ntheme = \"paper\"\n\
             [dwell]\nradius_frac = 0.9\n",
        );
        let loaded = s.load();
        assert_eq!(
            loaded.notice,
            Some(SettingsNotice::Repaired {
                fields: vec!["gain_x", "dwell.radius_frac"]
            })
        );
        assert_eq!(loaded.settings.gain_x, 6.0);
        assert_eq!(loaded.settings.smoothing, 0.4);
        assert_eq!(loaded.settings.theme, Theme::Paper);
        assert_eq!(loaded.settings.dwell.radius_frac, 0.05);
        assert!(!s.is_read_only());
    }

    #[test]
    fn unreadable_file_gives_defaults_and_read_only_store() {
        // A directory where the file should be: reading fails with an error that isn't
        // NotFound on every OS.
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        fs::create_dir_all(s.path()).unwrap();
        let loaded = s.load();
        assert_eq!(loaded.settings, Settings::default());
        assert!(
            matches!(loaded.notice, Some(SettingsNotice::Unreadable { .. })),
            "{:?}",
            loaded.notice
        );
        assert!(s.is_read_only());
        assert_eq!(s.save(&changed()).unwrap(), Saved::Skipped);
        assert!(s.path().is_dir());
    }

    #[test]
    fn save_creates_the_directory_and_replaces_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        s.save(&Settings::default()).unwrap();
        s.save(&changed()).unwrap();
        assert_eq!(s.load().settings, changed());
    }

    #[test]
    fn save_error_is_reported_not_panicked() {
        // The parent "directory" is a file, so creating the directory fails.
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("cfg");
        fs::write(&blocker, "").unwrap();
        let s = SettingsStore::at(blocker.join(FILE_NAME));
        assert!(matches!(
            s.save(&Settings::default()),
            Err(SaveError::Io { .. })
        ));
    }

    #[test]
    fn every_notice_has_user_facing_text() {
        let notices = [
            SettingsNotice::Corrupt {
                reason: "x".into(),
                moved_to: Some(PathBuf::from("a.corrupt")),
            },
            SettingsNotice::NewerSchema {
                found: 2,
                supported: 1,
            },
            SettingsNotice::Repaired {
                fields: vec!["gain_x"],
            },
            SettingsNotice::Unreadable { reason: "x".into() },
        ];
        for n in notices {
            assert!(n.to_string().len() > 20, "{n:?}");
        }
    }

    #[test]
    fn config_dir_follows_each_os_convention() {
        use std::ffi::OsString;
        // Absolute on the host OS (a POSIX "/home/u" isn't absolute on Windows).
        let root = std::env::temp_dir();
        let home = root.join("home");
        let xdg = root.join("xdg");
        let appdata = root.join("appdata");
        let env = |pairs: Vec<(&'static str, PathBuf)>| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(n, _)| *n == k)
                    .map(|(_, v)| OsString::from(v))
            }
        };
        let none = || env(vec![]);

        assert_eq!(
            config_dir(Os::Mac, none(), Some(home.clone())),
            Some(home.join("Library/Application Support/Open Camera Mouse"))
        );
        assert_eq!(
            config_dir(Os::Windows, env(vec![("APPDATA", appdata.clone())]), None),
            Some(appdata.join("Open Camera Mouse"))
        );
        assert_eq!(config_dir(Os::Windows, none(), Some(home.clone())), None);
        assert_eq!(
            config_dir(
                Os::Unix,
                env(vec![("XDG_CONFIG_HOME", xdg.clone())]),
                Some(home.clone())
            ),
            Some(xdg.join("open-camera-mouse"))
        );
        assert_eq!(
            config_dir(Os::Unix, none(), Some(home.clone())),
            Some(home.join(".config/open-camera-mouse"))
        );
        // Relative values are ignored, never resolved against the working directory.
        let rel = PathBuf::from("relative");
        assert_eq!(
            config_dir(
                Os::Unix,
                env(vec![("XDG_CONFIG_HOME", rel.clone())]),
                Some(home.clone())
            ),
            Some(home.join(".config/open-camera-mouse"))
        );
        assert_eq!(config_dir(Os::Unix, none(), Some(rel.clone())), None);
        assert_eq!(config_dir(Os::Mac, none(), None), None);
        assert_eq!(
            config_dir(Os::Windows, env(vec![("APPDATA", rel)]), None),
            None
        );
    }

    #[test]
    fn user_config_dir_ends_in_the_settings_file() {
        // CI runners have a home directory; this only checks the shape, not the OS path.
        if let Some(s) = SettingsStore::in_user_config_dir() {
            assert_eq!(s.path().file_name().unwrap(), FILE_NAME);
            assert!(s.path().is_absolute());
        }
    }
}
