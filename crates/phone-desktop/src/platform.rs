//! App-independent host ports. No adapter receives Shell or owns persistence.
pub(crate) mod visual;
pub use crate::persistence::{Directory, Record, Store};
pub use crate::storage::files::Folder;
pub use crate::web::parse;
use crate::{Request, Runtime, config::Features, web::Web};
pub use image::RgbaImage as Picture;
pub use phone_core::browser::Page;
use std::{path::Path, sync::Arc};
pub use visual::{Decoder, Video, VideoFrame, Visual};

/// Opaque binding to a runtime generation; restored instances get a new identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instance(pub(crate) u64);

pub trait Documents {
    fn submit(&mut self, request: Request, current: Option<Page>) -> Result<(), String>;
    fn poll(&mut self) -> Option<(Request, Result<Page, String>)>;
    fn cancel(&mut self);
}
/// Immutable playback intent. Adapters cannot edit application queues or navigation.
#[derive(Clone, Copy)]
pub struct Playback {
    pub playing: bool,
    pub position: f32,
    pub duration: f32,
    pub volume: u8,
    pub generation: u64,
    pub seek_revision: u64,
}
/// Device observations; the application decides how EOF affects repeat/queue state.
#[derive(Default)]
pub struct PlaybackStatus {
    pub duration: Option<f32>,
    pub position: Option<f32>,
    pub ended: bool,
    pub stopped: bool,
    pub error: Option<String>,
}
pub trait Sound {
    fn effect_volume(&mut self, volume: u8);
    fn feedback(&mut self, cue: crate::Cue, volume: u8) -> Result<(), String>;
    fn poll_effects(&mut self) -> Result<(), String>;
    fn preview(&mut self, files: &Folder, path: &str, volume: u8) -> Result<(), String>;
    fn alert(&mut self, files: &Folder, tone: Option<&str>, volume: u8) -> Result<(), String>;
    fn stop(&mut self);
    fn sync(&mut self, files: &Folder, path: Option<&str>, playback: Playback) -> PlaybackStatus;
}
/// A selected real folder, output device and decoder, supplied explicitly once.
/// This is direct filesystem access, not a virtual disk or a kernel driver API.
pub struct Devices {
    pub folder: Arc<Folder>,
    pub sound: Box<dyn Sound>,
    pub decoder: Arc<dyn Decoder>,
}
impl Devices {
    pub fn native(folder: Folder) -> Self {
        Self {
            folder: Arc::new(folder),
            sound: Box::<crate::audio::Audio>::default(),
            decoder: Arc::new(visual::SystemDecoder),
        }
    }
}
/// Custom hosts may provide documents, media, both or neither. Methods must not
/// wait for network/decoding; use a worker and return completions through poll.
pub trait Platform {
    fn instance(&self) -> Instance;
    fn documents(&mut self) -> Option<&mut dyn Documents> {
        None
    }
    fn take_devices(&mut self) -> Option<Devices> {
        None
    }
}

pub struct Native {
    instance: Instance,
    web: Web,
    request: Option<Request>,
    devices: Option<Devices>,
}
impl Native {
    /// No window, picker or profile I/O. Runtime/Store owns application data.
    pub fn new(
        runtime: &Runtime,
        directory: Option<&Path>,
        read_only: bool,
    ) -> Result<Self, String> {
        let folder = directory
            .map(|p| Folder::new(p, read_only).map_err(|e| e.to_string()))
            .transpose()?;
        Ok(Self::from_folder(runtime, folder))
    }
    pub fn from_folder(runtime: &Runtime, folder: Option<Folder>) -> Self {
        let Features {
            network,
            javascript,
            ..
        } = runtime.capabilities();
        Self {
            instance: runtime.instance(),
            web: Web::configured(network, javascript),
            request: None,
            devices: folder.map(Devices::native),
        }
    }
    /// Supply a host decoder while retaining the native document and audio services.
    pub fn with_decoder(mut self, decoder: Arc<dyn Decoder>) -> Self {
        if let Some(devices) = &mut self.devices {
            devices.decoder = decoder;
        }
        self
    }
    /// Compatibility convenience; all app integration lives in Runtime now.
    pub fn poll(&mut self, runtime: &mut Runtime) -> Result<(), String> {
        runtime.poll_platform(self)
    }
}
impl Platform for Native {
    fn instance(&self) -> Instance {
        self.instance
    }
    fn documents(&mut self) -> Option<&mut dyn Documents> {
        Some(self)
    }
    fn take_devices(&mut self) -> Option<Devices> {
        self.devices.take()
    }
}
impl Documents for Native {
    fn submit(&mut self, request: Request, current: Option<Page>) -> Result<(), String> {
        self.web
            .load_page(request.id, request.url.clone(), current)?;
        self.request = Some(request);
        Ok(())
    }
    fn poll(&mut self) -> Option<(Request, Result<Page, String>)> {
        while let Ok((id, result)) = self.web.results.try_recv() {
            if self.request.as_ref().is_some_and(|r| r.id == id) {
                return Some((self.request.take().unwrap(), result));
            }
        }
        None
    }
    fn cancel(&mut self) {
        self.web.cancel();
        self.request = None;
    }
}
