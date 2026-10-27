//! Application-side completion handling. Adapters never receive Shell.
use super::*;
use crate::platform::Platform;

impl Runtime {
    pub(in crate::runtime) fn connect(&mut self, backend: Box<dyn Backend>) {
        self.shell.messaging.backend = backend
            .capabilities()
            .send_messages
            .then(|| backend.id().to_owned());
        self.app.services = Services::new(backend);
    }
    pub fn poll_services(&mut self) -> Result<(), String> {
        self.shell.poll_services(&mut self.app.services)
    }
    pub fn poll_platform(&mut self, platform: &mut dyn Platform) -> Result<(), String> {
        if platform.instance() != self.instance() {
            return Err("Native adapter belongs to another runtime".into());
        }
        if !self.app.attached {
            self.app.media = platform
                .take_devices()
                .map(|devices| crate::media::Media::attach(devices, &mut self.shell));
            self.app.attached = true;
        }
        if let Some(media) = &mut self.app.media {
            media.tick(&mut self.shell);
        }
        if let Some(request) = self.take_request() {
            let current = request
                .url
                .starts_with("about:script/")
                .then(|| self.shell.browsing.model.page.clone());
            let result = platform
                .documents()
                .ok_or("此宿主未提供网页服务。".to_owned())
                .and_then(|documents| documents.submit(request.clone(), current));
            match result {
                Ok(()) => self.app.request = Some(request),
                Err(error) => {
                    self.complete(&request, Err(error));
                    self.app.request = None;
                }
            }
        }
        if self.app.request.is_some() && !self.shell.browsing.model.loading {
            if let Some(documents) = platform.documents() {
                documents.cancel();
            }
            self.app.request = None;
        }
        for _ in 0..16 {
            let Some((request, result)) = platform.documents().and_then(|d| d.poll()) else {
                break;
            };
            if !self
                .app
                .request
                .as_ref()
                .is_some_and(|r| r.owner == request.owner && r.id == request.id)
            {
                continue;
            }
            self.app.request = None;
            let keep_position = request.url.starts_with("about:script/")
                && result
                    .as_ref()
                    .is_ok_and(|p| p.url == self.shell.browsing.model.page.url);
            let position = (
                self.shell.browsing.model.focus,
                self.shell.browsing.model.scroll,
            );
            if self.complete(&request, result) && keep_position {
                self.shell.browsing.model.focus = position.0.min(
                    self.shell
                        .browsing
                        .model
                        .page
                        .elements
                        .len()
                        .saturating_sub(1),
                );
                self.shell.browsing.model.scroll = position.1;
            }
        }
        Ok(())
    }
    /// Request a document through the same worker boundary used by browser navigation.
    pub fn navigate(&mut self, url: &str) -> Result<(), String> {
        if !self.shell.apps.browser {
            return Err("Browser is disabled".into());
        }
        if url.is_empty() || url.len() > 8192 {
            return Err("Invalid document URL".into());
        }
        self.shell.go(Screen::Browser);
        self.shell.browsing.view = crate::shell::BrowserView::Page;
        self.shell.browsing.pending = Some(url.into());
        Ok(())
    }
    pub fn take_request(&mut self) -> Option<Request> {
        let url = self.shell.browsing.pending.take()?;
        self.shell.browsing.requested = url.clone();
        let id = self.shell.browsing.model.start();
        if !self.shell.settings.preferences.mobile_data {
            self.shell
                .browsing
                .model
                .complete(id, Err("移动数据已关闭。".into()));
            return None;
        }
        Some(Request {
            url,
            id,
            owner: self.owner,
        })
    }
    pub fn complete(
        &mut self,
        request: &Request,
        result: Result<phone_core::browser::Page, String>,
    ) -> bool {
        if request.owner != self.owner {
            return false;
        }
        let updated = self.shell.browsing.model.complete(request.id, result);
        if updated {
            self.shell
                .browsing
                .data
                .visit(&self.shell.browsing.model.page);
            self.shell.browsing.dirty = true;
            if matches!(
                self.shell.overlay,
                Some(
                    Overlay::Text {
                        target: TextTarget::Field(_),
                        ..
                    } | Overlay::Select { .. }
                )
            ) {
                self.shell.overlay = None;
            }
        }
        updated
    }
}
