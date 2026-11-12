//! UI-less application controller (ADR-0003/ADR-0007).
//!
//! [`Controller`] is the whole R-A1 state machine: commands and inputs in, events, effects and
//! a render [`View`] out. Pure: no I/O, no threads, no clock (`now_us` is passed in). The
//! desktop app and the integration tests drive the same controller; `platform` adapters
//! implement the [`ports`].

pub mod controller;
pub mod ports;
pub mod settings;

pub use controller::{
    AppState, Command, Controller, Effect, Event, Input, Marker, MarkerKind, PointerAccess,
    PointerOwner, RECENTER_US, RecenterView, Step, View,
};
pub use pointer_core::DesktopRect;
pub use settings::{Settings, SettingsError, Theme};
