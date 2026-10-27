//! App-side effects bridge. Platform ports exchange requests/data, not Phone.
use super::*;
use crate::phone;
use crate::platform::{Instance, Platform};

#[derive(Default)]
pub(super) struct Host {
    attached: bool,
    media: Option<crate::media::Media>,
    request: Option<Request>,
}
impl Runtime {
    pub fn instance(&self) -> Instance {
        Instance(self.owner)
    }
    pub fn capabilities(&self) -> crate::config::Features {
        self.phone.features
    }

    pub fn poll_platform(&mut self, platform: &mut dyn Platform) -> Result<(), String> {
        if platform.instance() != self.instance() {
            return Err("Native adapter belongs to another runtime".into());
        }
        if !self.host.attached {
            self.host.media = platform
                .take_devices()
                .map(|devices| crate::media::Media::attach(devices, &mut self.phone));
            self.host.attached = true;
        }
        if let Some(media) = &mut self.host.media {
            media.tick(&mut self.phone);
        }
        if let Some(request) = self.take_request() {
            let current = request
                .url
                .starts_with("about:script/")
                .then(|| self.phone.browsing.model.page.clone());
            let result = platform
                .documents()
                .ok_or("此宿主未提供网页服务。".to_owned())
                .and_then(|documents| documents.submit(request.clone(), current));
            match result {
                Ok(()) => self.host.request = Some(request),
                Err(error) => {
                    self.complete(&request, Err(error));
                    self.host.request = None;
                }
            }
        }
        if self.host.request.is_some() && !self.phone.browsing.model.loading {
            if let Some(documents) = platform.documents() {
                documents.cancel();
            }
            self.host.request = None;
        }
        for _ in 0..16 {
            let Some((request, result)) = platform.documents().and_then(|d| d.poll()) else {
                break;
            };
            if !self
                .host
                .request
                .as_ref()
                .is_some_and(|r| r.owner == request.owner && r.id == request.id)
            {
                continue;
            }
            self.host.request = None;
            let keep_position = request.url.starts_with("about:script/")
                && result
                    .as_ref()
                    .is_ok_and(|p| p.url == self.phone.browsing.model.page.url);
            let position = (
                self.phone.browsing.model.focus,
                self.phone.browsing.model.scroll,
            );
            if self.complete(&request, result) && keep_position {
                self.phone.browsing.model.focus = position.0.min(
                    self.phone
                        .browsing
                        .model
                        .page
                        .elements
                        .len()
                        .saturating_sub(1),
                );
                self.phone.browsing.model.scroll = position.1;
            }
        }
        Ok(())
    }
}

impl Runtime {
    pub fn report(&mut self, message: impl Into<String>) {
        self.phone.status = message.into();
    }
    /// Shared window/host close policy: commit drafts, retain the editor after
    /// failures and do not quit during mutating file operations.
    pub fn close(&mut self, store: &mut dyn crate::persistence::Store) -> bool {
        if self.phone.files.busy
            || self
                .phone
                .files
                .pending
                .as_ref()
                .is_some_and(phone::files::FileRequest::mutating)
        {
            self.phone.notice("文件操作尚未结束，请稍候。");
            return false;
        }
        if self.phone.messaging.ui.pending.is_some() {
            if let Err(error) = self.flush(store, false) {
                self.report(error);
            }
            // Failed queue writes roll back and leave the original editor open.
            if matches!(
                self.phone.screen,
                phone::Screen::Compose | phone::Screen::Conversation
            ) {
                return false;
            }
        }
        if matches!(
            self.phone.screen,
            phone::Screen::Compose | phone::Screen::Conversation
        ) {
            if !self.phone.messaging.compose.body.accept_prediction()
                || !self.phone.messaging.compose.to.accept_prediction()
            {
                self.phone
                    .notice("Text limit reached. Confirm or cancel the candidate before closing.");
                return false;
            }
            self.phone.save_draft();
        }
        if let Err(error) = self.flush(store, false) {
            self.report(error);
        }
        if self.unsaved() {
            self.phone
                .notice("Save failed. Window stays open so you can keep your text.");
            return false;
        }
        if let Err(error) = self.flush(store, true) {
            self.phone.notice(format!("{error}，窗口保持打开。"));
            return false;
        }
        true
    }
}
