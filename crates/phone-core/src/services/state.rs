use super::{Envelope, Event};
use crate::sms::Mailbox;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SubmissionState {
    Queued,
    Accepted,
    Sent,
    Delivered,
    Failed { reason: String },
}
impl SubmissionState {
    pub fn pending(&self) -> bool {
        matches!(self, Self::Queued | Self::Accepted)
    }
    pub fn sent(&self) -> bool {
        matches!(self, Self::Sent | Self::Delivered)
    }
    fn allows(&self, next: &Self) -> bool {
        self == next
            || match self {
                Self::Queued => !matches!(next, Self::Queued),
                Self::Accepted => {
                    matches!(next, Self::Sent | Self::Delivered | Self::Failed { .. })
                }
                Self::Sent => matches!(next, Self::Delivered),
                Self::Delivered | Self::Failed { .. } => false,
            }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Submission {
    pub backend: String,
    pub request_id: u64,
    pub state: SubmissionState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel {
    Information,
    Warning,
    Emergency,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub body: String,
    pub level: AlertLevel,
    pub issued_ms: u64,
    pub expires_ms: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredAlert {
    pub backend: String,
    pub alert: Alert,
    pub dismissed: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceState {
    pub next_request: u64,
    pub cursors: BTreeMap<String, u64>,
    pub alerts: Vec<StoredAlert>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Apply {
    Replay,
    Received(usize),
    Updated,
}
pub(super) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-:".contains(&b))
}
impl Mailbox {
    /// Validates before mutation. Caller persists mailbox+cursor in ONE atomic write.
    pub fn apply_service_event(
        &mut self,
        backend: &str,
        envelope: &Envelope,
    ) -> Result<Apply, &'static str> {
        if !valid_id(backend) {
            return Err("Invalid backend identity");
        }
        let cursor = self.services.cursors.get(backend).copied().unwrap_or(0);
        if envelope.sequence == 0 {
            return Err("Sequence starts at 1");
        }
        if envelope.sequence <= cursor {
            return Ok(Apply::Replay);
        }
        if cursor.checked_add(1) != Some(envelope.sequence) {
            return Err("Service event sequence gap");
        }
        if !self.services.cursors.contains_key(backend) && self.services.cursors.len() >= 8 {
            return Err("Too many service streams");
        }
        let applied = match &envelope.event {
            Event::MessageReceived {
                from,
                body,
                time_ms,
            } => {
                let i = self.receive(from.clone(), body.clone())?;
                self.inbox[i].time = *time_ms;
                Apply::Received(i)
            }
            Event::MessageStatus { request_id, state } => {
                if matches!(state, SubmissionState::Failed { reason } if reason.len() > 1024) {
                    return Err("Service error is too long");
                }
                if *request_id == 0 || *request_id >= self.services.next_request {
                    return Err("Unknown request id");
                }
                if let Some(m) = self.outbox.iter_mut().find(|m| {
                    m.submission
                        .as_ref()
                        .is_some_and(|s| s.backend == backend && s.request_id == *request_id)
                }) {
                    let submission = m.submission.as_mut().unwrap();
                    // Late progress must not turn a completed send back into a pending one.
                    if submission.state.allows(state) {
                        submission.state = state.clone();
                        m.sent = state.sent();
                    }
                }
                // Deleted messages stay deleted when a late receipt arrives.
                Apply::Updated
            }
            Event::AlertPublished { alert } => {
                if !valid_id(&alert.id)
                    || alert.title.trim().is_empty()
                    || alert.title.encode_utf16().count() > 80
                    || alert.body.is_empty()
                    || alert.body.encode_utf16().count() > 4096
                    || alert.expires_ms.is_some_and(|end| end <= alert.issued_ms)
                {
                    return Err("Invalid alert");
                }
                let stored = StoredAlert {
                    backend: backend.into(),
                    alert: alert.clone(),
                    dismissed: false,
                };
                if let Some(old) = self
                    .services
                    .alerts
                    .iter_mut()
                    .find(|a| a.backend == backend && a.alert.id == alert.id)
                {
                    *old = stored;
                } else {
                    // Dismissed entries can be discarded; stream cursor still prevents replay.
                    if self.services.alerts.iter().filter(|a| !a.dismissed).count() >= 32 {
                        return Err("Alert queue is full");
                    }
                    self.services.alerts.retain(|a| !a.dismissed);
                    self.services.alerts.push(stored);
                }
                Apply::Updated
            }
            Event::AlertWithdrawn { id } => {
                if !valid_id(id) {
                    return Err("Invalid alert id");
                }
                self.services
                    .alerts
                    .retain(|a| a.backend != backend || a.alert.id != *id);
                Apply::Updated
            }
            Event::Call(_) => return Err("Calls are reserved and not implemented"),
        };
        self.services
            .cursors
            .insert(backend.into(), envelope.sequence);
        Ok(applied)
    }
}
