//! User-space shell interaction runtime. No window, event loop or device is opened on construction.
mod assets;
mod audio;
mod canvas;
pub mod config;
mod effects;
#[cfg(any(feature = "desktop", test))]
mod frames;
#[cfg(feature = "window")]
mod gpu;
#[cfg(feature = "window")]
mod host;
mod media;
mod menu;
mod packed;
mod services;
mod shell;
mod storage;
mod theme;
mod ui;
mod views;
mod web;

#[cfg(feature = "desktop")]
pub mod desktop;
pub mod persistence;
pub mod platform;
mod runtime;
pub use effects::Cue;
pub use phone_core::{input, services as messaging};
pub use runtime::{Frame, Request, Runtime, Time};

#[cfg(all(test, feature = "desktop"))]
mod accept;

#[cfg(all(target_os = "android", feature = "android"))]
mod android;
