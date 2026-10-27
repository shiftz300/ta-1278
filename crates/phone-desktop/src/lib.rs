//! User-space phone interaction runtime. No window, event loop or device is opened on construction.
mod assets;
mod audio;
mod canvas;
pub mod config;
mod effects;
#[cfg(any(feature = "desktop", test))]
mod frames;
#[cfg(feature = "desktop")]
mod gpu;
#[cfg(feature = "desktop")]
mod host;
mod media;
mod menu;
mod packed;
mod phone;
mod services;
mod storage;
mod theme;
mod ui;
mod views;
mod web;

#[cfg(feature = "desktop")]
pub mod desktop;
pub mod platform;
mod runtime;
pub use effects::Cue;
pub use phone_core::{input, services as messaging};
pub use runtime::{Frame, Request, Runtime, Time};

#[cfg(all(test, feature = "desktop"))]
mod accept;
