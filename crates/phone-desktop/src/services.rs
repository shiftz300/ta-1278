//! Nonblocking bridge. The default adapter is offline; no sockets/modem are opened.
use crate::phone::{Overlay, Phone};
use phone_core::services::{
    Apply, Backend, Command, Envelope, Event, Offline, PortError, SendMessage,
};
use std::collections::HashSet;

pub struct Services {
    backend: Box<dyn Backend>,
    pending: Option<Envelope>,
    applied: bool,
    submitted: HashSet<u64>,
}
impl Default for Services {
    fn default() -> Self {
        Self::new(Box::new(Offline))
    }
}
impl Services {
    pub fn new(backend: Box<dyn Backend>) -> Self {
        Self {
            backend,
            pending: None,
            applied: false,
            submitted: HashSet::new(),
        }
    }
    /// Called after the app's normal flush. A dirty mailbox is never submitted
    /// or acknowledged; failed saves leave the event available for retry.
    pub fn tick(&mut self, phone: &mut Phone) -> Result<(), String> {
        let capabilities = self.backend.capabilities();
        phone.messaging.backend = capabilities.send_messages.then(|| self.backend.id().into());
        if phone.messaging.dirty || phone.messaging.ui.pending.is_some() {
            return Ok(());
        }
        if let Some(envelope) = &self.pending
            && self.applied
        {
            match self.backend.acknowledge(envelope.sequence) {
                Ok(()) => {
                    self.pending = None;
                    self.applied = false;
                }
                Err(PortError::Busy | PortError::Disconnected) => return Ok(()),
                Err(e) => return Err(format!("Service ack: {e:?}")),
            }
        }
        let now = phone.timestamp();
        let before = phone.messaging.mailbox.services.alerts.len();
        phone
            .messaging
            .mailbox
            .services
            .alerts
            .retain(|a| a.alert.expires_ms.is_none_or(|end| end > now));
        if before != phone.messaging.mailbox.services.alerts.len() {
            phone.messaging.dirty = true;
            return Ok(());
        }
        self.submitted.retain(|id| {
            phone.messaging.mailbox.outbox.iter().any(|m| {
                m.submission
                    .as_ref()
                    .is_some_and(|s| s.request_id == *id && s.state.pending())
            })
        });
        if capabilities.send_messages {
            for message in &phone.messaging.mailbox.outbox {
                let Some(s) = &message.submission else {
                    continue;
                };
                if s.backend != self.backend.id()
                    || !s.state.pending()
                    || self.submitted.contains(&s.request_id)
                {
                    continue;
                }
                let command = Command::Send(SendMessage {
                    request_id: s.request_id,
                    to: message.to.clone(),
                    body: message.body.clone(),
                });
                match self.backend.submit(command) {
                    Ok(()) => {
                        self.submitted.insert(s.request_id);
                    }
                    Err(PortError::Busy | PortError::Disconnected) => {}
                    Err(e) => return Err(format!("Service submit: {e:?}")),
                }
                break; // At most one dispatch per frame; never block input.
            }
        }
        if self.pending.is_none() {
            phone.present_service_alert();
            self.pending = match self.backend.poll() {
                Ok(event) => event,
                Err(PortError::Busy | PortError::Disconnected) => None,
                Err(e) => return Err(format!("Service poll: {e:?}")),
            };
        }
        if let Some(envelope) = &self.pending {
            let enabled = match &envelope.event {
                Event::MessageReceived { .. } => capabilities.receive_messages,
                Event::MessageStatus { .. } => capabilities.send_messages,
                Event::AlertPublished { .. } | Event::AlertWithdrawn { .. } => capabilities.alerts,
                Event::Call(_) => false,
            };
            if !enabled {
                return Err("Service event exceeds enabled capabilities".into());
            }
            let selected = if phone.screen == crate::phone::Screen::Messages && phone.index > 0 {
                phone
                    .messaging
                    .mailbox
                    .conversations()
                    .get(phone.index - 1)
                    .map(|c| c.peer.clone())
            } else {
                None
            };
            let result = phone
                .messaging
                .mailbox
                .apply_service_event(self.backend.id(), envelope)?;
            if result != Apply::Replay {
                phone.messaging.dirty = true;
                if let Apply::Received(index) = result {
                    phone.received_message(index, selected);
                }
            }
            self.applied = true;
            return Ok(()); // Ack on a subsequent tick, after successful flush.
        }
        phone.present_service_alert();
        Ok(())
    }
}
impl Phone {
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

#[cfg(test)]
mod tests;
