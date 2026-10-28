//! Host input and native application lifecycle.
pub mod input;
#[cfg(target_os = "macos")]
pub mod lifecycle;

mod app;
pub(crate) mod keypad;
pub(crate) use app::App;
#[cfg(feature = "desktop")]
pub(crate) fn theme(runtime: &mut crate::Runtime) {
    runtime.shell.settings.accent = 4;
}
