//! Nonblocking bridge. The default adapter is offline; no sockets/modem are opened.
use crate::shell::mail::State;
use phone_core::services::{
    Apply, Backend, Capabilities, Command, Envelope, Event, Offline, PortError, SendMessage,
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
    pub fn prepare(&mut self, mail: &mut State, now: u64) -> Result<Option<Capabilities>, String> {
        let capabilities = self.backend.capabilities();
        mail.backend = capabilities.send_messages.then(|| self.backend.id().into());
        if mail.dirty || mail.ui.pending.is_some() {
            return Ok(None);
        }
        if let Some(envelope) = &self.pending
            && self.applied
        {
            match self.backend.acknowledge(envelope.sequence) {
                Ok(()) => {
                    self.pending = None;
                    self.applied = false;
                }
                Err(PortError::Busy | PortError::Disconnected) => return Ok(None),
                Err(e) => return Err(format!("Service ack: {e:?}")),
            }
        }
        let before = mail.mailbox.services.alerts.len();
        mail.mailbox
            .services
            .alerts
            .retain(|a| a.alert.expires_ms.is_none_or(|end| end > now));
        if before != mail.mailbox.services.alerts.len() {
            mail.dirty = true;
            return Ok(None);
        }
        self.submitted.retain(|id| {
            mail.mailbox.outbox.iter().any(|m| {
                m.submission
                    .as_ref()
                    .is_some_and(|s| s.request_id == *id && s.state.pending())
            })
        });
        if capabilities.send_messages {
            for message in &mail.mailbox.outbox {
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
        Ok(Some(capabilities))
    }
    pub fn needs_poll(&self) -> bool {
        self.pending.is_none()
    }
    pub fn receive(
        &mut self,
        mail: &mut State,
        capabilities: Capabilities,
    ) -> Result<Option<Apply>, String> {
        if self.pending.is_none() {
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
            let result = mail
                .mailbox
                .apply_service_event(self.backend.id(), envelope)?;
            if result != Apply::Replay {
                mail.dirty = true;
            }
            self.applied = true;
            return Ok(Some(result)); // Ack only after a subsequent successful save.
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
