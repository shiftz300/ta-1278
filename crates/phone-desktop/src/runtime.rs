//! Host-driven execution. All window/event-loop ownership stays outside this type.
mod app;
use crate::{
    canvas::Canvas,
    config::RuntimeConfig,
    shell::{Overlay, Screen, Shell, TextTarget},
    views,
};
use phone_core::{
    input::{Event, Input, Key},
    predict::firmware::Firmware,
    services::{Backend, Offline},
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

/// The host supplies both clocks. Do not advance either while virtual time is paused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Time {
    pub elapsed_ms: u64,
    pub unix_ms: u64,
}
impl Time {
    fn validate(self) -> Result<(), String> {
        if self.unix_ms > 253402300799999 {
            return Err("Wall time exceeds year 9999".into());
        }
        Ok(())
    }
}
/// Tightly packed, top-to-bottom RGBA8. Borrowed until the next mutable runtime call.
pub struct Frame<'a> {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub pixels: &'a [u8],
}
/// An opaque instance-bound browser operation. Old runtime responses are rejected.
#[derive(Clone, Debug)]
pub struct Request {
    pub url: String,
    pub(crate) id: u64,
    owner: u64,
}

pub struct Runtime {
    pub(crate) shell: Shell,
    pub(crate) canvas: Canvas,
    pub(crate) predictor: Arc<Firmware>,
    pub(crate) owner: u64,
    input: Input,
    time: Time,
    app: app::App,
}
impl Runtime {
    /// Constructs an offline shell. No window, filesystem, audio device or worker is opened.
    pub fn new(config: RuntimeConfig, time: Time) -> Result<Self, String> {
        Self::with_backend(config, time, Box::new(Offline))
    }
    pub fn with_backend(
        config: RuntimeConfig,
        time: Time,
        backend: Box<dyn Backend>,
    ) -> Result<Self, String> {
        if config.version != 1 {
            return Err("Unsupported runtime config version".into());
        }
        time.validate()?;
        let predictor = Arc::new(Firmware::from_data(Default::default())?);
        let mut shell = Shell::new(Default::default());
        shell.apps = config.apps;
        shell.features = config.effective_features();
        // Real tracks are supplied by the media adapter, never simulated here.
        shell.playback.model.tracks.clear();
        shell.playback.model.external_clock = true;
        let mut runtime = Self::from_shell(shell, predictor, time)?;
        runtime.connect(backend);
        Ok(runtime)
    }
    pub(crate) fn from_shell(
        mut shell: Shell,
        predictor: Arc<Firmware>,
        time: Time,
    ) -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        time.validate()?;
        shell.wall_ms = Some(time.unix_ms);
        shell.last_tick = Some(time.elapsed_ms);
        shell.prediction_engine = predictor.clone();
        shell.messaging.compose.time = time.unix_ms;
        shell.organizer.date = shell.local_clock().0;
        Ok(Self {
            shell,
            predictor,
            time,
            owner: NEXT.fetch_add(1, Ordering::Relaxed),
            canvas: Canvas::new()?,
            input: Input::default(),
            app: Default::default(),
        })
    }
    fn bound_cues(&mut self) {
        let excess = self.shell.cues.len().saturating_sub(64);
        self.shell.cues.drain(..excess);
    }
    /// Short sound effects for a custom host. Native consumes these itself when attached.
    /// Retains at most the latest 64 cues if a host does not consume them.
    pub fn take_cues(&mut self) -> Vec<crate::Cue> {
        let cues = std::mem::take(&mut self.shell.cues);
        if self.shell.features.sounds && !self.shell.settings.preferences.silent {
            cues
        } else {
            Vec::new()
        }
    }
    pub fn time(&self) -> Time {
        self.time
    }
    /// Advance timers and held keys. Equal elapsed times are allowed; backwards time is not.
    pub fn advance(&mut self, time: Time) -> Result<(), String> {
        time.validate()?;
        let elapsed = time
            .elapsed_ms
            .checked_sub(self.time.elapsed_ms)
            .ok_or("Elapsed time moved backwards; restore into a new runtime instead")?;
        self.time = time;
        self.shell.wall_ms = Some(time.unix_ms);
        for event in self.input.tick(time.elapsed_ms) {
            self.dispatch(event);
        }
        self.shell
            .tick(time.elapsed_ms, (elapsed as f32 / 1000.).min(0.25));
        self.bound_cues();
        Ok(())
    }
    pub(crate) fn dispatch(&mut self, event: Event) {
        self.shell.status.clear();
        if matches!(
            event.phase,
            phone_core::input::Phase::Down | phone_core::input::Phase::Repeat
        ) {
            views::prepare(&mut self.canvas, &mut self.shell);
        }
        self.shell.handle(event, self.time.elapsed_ms);
        self.bound_cues();
    }
    pub fn press(&mut self, source: u32, key: Key) {
        if matches!(key, Key::Digit(n) if n > 9) {
            return;
        }
        if let Some(event) = self.input.press(source, key, self.time.elapsed_ms) {
            self.dispatch(event);
        }
    }
    pub fn release(&mut self, source: u32) {
        for event in self.input.release(source) {
            self.dispatch(event);
        }
    }
    /// Call on focus loss or before suspending the host. Cancellation never confirms a key.
    pub fn cancel_input(&mut self) {
        for event in self.input.cancel() {
            self.dispatch(event);
        }
    }
    /// Optional ASCII keyboard convenience. Composed host IME text is rejected.
    pub fn text(&mut self, text: &str) {
        if text
            .chars()
            .all(|c| c.is_ascii() && (!c.is_control() || c == '\n'))
        {
            self.shell.text(text);
        }
    }
    pub fn tab(&mut self) {
        self.shell.tab();
    }
    pub fn select_all(&mut self) {
        if let Some(editor) = self.shell.editor_mut() {
            editor.select_all();
        }
    }
    pub fn render(&mut self) -> Frame<'_> {
        views::paint(&mut self.canvas, &self.shell);
        Frame {
            width: crate::canvas::W,
            height: crate::canvas::H,
            stride: crate::canvas::W as usize * 4,
            pixels: &self.canvas.pixels,
        }
    }
    /// Does not mark dirty state saved. Submit/ack remain gated on successful persistence.
    /// A mailbox transaction must be persisted before the backend may submit or ack it.
    /// Other user data can be saved at host-selected checkpoints through `save`.
    pub fn pending_commit(&self) -> bool {
        self.shell.messaging.dirty || self.shell.messaging.ui.pending.is_some()
    }
    pub fn instance(&self) -> crate::platform::Instance {
        crate::platform::Instance(self.owner)
    }
    pub fn capabilities(&self) -> crate::config::Features {
        self.shell.features
    }
    pub fn report(&mut self, message: impl Into<String>) {
        self.shell.status = message.into();
    }
    pub fn status(&self) -> &str {
        &self.shell.status
    }
    pub fn mailbox(&self) -> &phone_core::sms::Mailbox {
        &self.shell.messaging.mailbox
    }
}
#[cfg(test)]
mod tests;
