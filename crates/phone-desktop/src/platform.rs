//! Optional native user-space adapters. Creating a Runtime alone does not activate them.
pub use crate::web::parse;
use crate::{Runtime, media::Media, runtime::Request, storage::files::Folder, web::Web};
pub use phone_core::browser::Page;
use std::path::Path;

/// Existing folder/audio/HTTP adapter, usable with or without a desktop window.
/// Bound to one runtime so background completions cannot leak into a restored instance.
pub struct Native {
    owner: u64,
    web: Web,
    media: Option<Media>,
    request: Option<Request>,
}
impl Native {
    /// `directory` maps the phone's media storage directly; `profile` stores music preferences.
    /// No picker is opened. None means no file/audio adapter. HTTP is gated by config.
    pub fn new(
        runtime: &mut Runtime,
        directory: Option<&Path>,
        profile: &Path,
        read_only: bool,
    ) -> Result<Self, String> {
        let folder = directory
            .map(|p| Folder::new(p, read_only).map_err(|error| error.to_string()))
            .transpose()?;
        Self::from_folder(runtime, folder, profile)
    }
    pub(crate) fn from_folder(
        runtime: &mut Runtime,
        folder: Option<Folder>,
        profile: &Path,
    ) -> Result<Self, String> {
        let media = folder
            .map(|folder| Media::new(folder, profile, &mut runtime.phone))
            .transpose()?;
        Ok(Self {
            owner: runtime.owner,
            web: Web::new(runtime.phone.features.network),
            media,
            request: None,
        })
    }
    fn check(&self, runtime: &Runtime) -> Result<(), String> {
        if runtime.owner != self.owner {
            return Err("Native adapter belongs to another runtime".into());
        }
        Ok(())
    }
    pub fn poll(&mut self, runtime: &mut Runtime) -> Result<(), String> {
        self.check(runtime)?;
        if let Some(media) = &mut self.media {
            media.tick(&mut runtime.phone);
        }
        if let Some(request) = runtime.take_request() {
            let result = self.web.load(request.id, request.url.clone());
            match result {
                Ok(()) => self.request = Some(request),
                Err(error) => {
                    runtime.complete(&request, Err(error));
                    self.request = None;
                }
            }
        }
        while let Ok((id, result)) = self.web.results.try_recv() {
            if self
                .request
                .as_ref()
                .is_some_and(|request| request.id == id)
            {
                runtime.complete(&self.request.take().unwrap(), result);
            }
        }
        Ok(())
    }
    pub fn save(&mut self, runtime: &Runtime, profile: &Path, force: bool) -> Result<(), String> {
        self.check(runtime)?;
        if let Some(media) = &mut self.media {
            media.save(profile, &runtime.phone, force)?;
        }
        Ok(())
    }
}
