//! Application-side message/alert integration. The service bridge only sees mailbox state.
use super::*;
use crate::services::Services;
use phone_core::services::Apply;

impl Shell {
    pub(crate) fn poll_services(&mut self, services: &mut Services) -> Result<(), String> {
        let now = self.timestamp();
        let Some(capabilities) = services.prepare(&mut self.messaging, now)? else {
            return Ok(());
        };
        if services.needs_poll() {
            self.present_service_alert();
        }
        let selected = if self.screen == Screen::Messages && self.index > 0 {
            self.messaging
                .mailbox
                .conversations()
                .get(self.index - 1)
                .map(|c| c.peer.clone())
        } else {
            None
        };
        match services.receive(&mut self.messaging, capabilities)? {
            Some(Apply::Received(index)) => self.received_message(index, selected),
            Some(_) => {}
            None => self.present_service_alert(),
        }
        Ok(())
    }
}
impl Shell {
    pub fn present_service_alert(&mut self) {
        let now = self.timestamp();
        if let Some(Overlay::ServiceAlert {
            backend,
            id,
            title,
            body,
        }) = &mut self.overlay
        {
            let live = self.messaging.mailbox.services.alerts.iter().find(|a| {
                &a.backend == backend
                    && &a.alert.id == id
                    && !a.dismissed
                    && a.alert.expires_ms.is_none_or(|end| end > now)
            });
            if let Some(a) = live {
                if *title != a.alert.title || *body != a.alert.body {
                    *title = a.alert.title.clone();
                    *body = a.alert.body.clone();
                    self.notice_scroll = 0;
                }
            } else {
                self.overlay = None;
            }
        }
        // Preserve active editors/modals and native alarm priority. This is a
        // host policy, not an assertion about native ETWS interruption rules.
        if self.overlay.is_some() || self.editor().is_some() || self.active_reminder().is_some() {
            return;
        }
        if let Some(a) = self
            .messaging
            .mailbox
            .services
            .alerts
            .iter()
            .find(|a| !a.dismissed && a.alert.expires_ms.is_none_or(|end| end > now))
        {
            self.notice_scroll = 0;
            self.notice_lines = 0;
            self.overlay = Some(Overlay::ServiceAlert {
                backend: a.backend.clone(),
                id: a.alert.id.clone(),
                title: a.alert.title.clone(),
                body: a.alert.body.clone(),
            });
        }
    }
}
