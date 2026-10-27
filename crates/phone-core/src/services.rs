//! Host service port, not a recovered modem ABI. No network/device implementation.
//! See docs/services.md for persistence, replay and capability contracts.
mod state;
use serde::{Deserialize, Serialize};
pub const API_VERSION: u32 = 1;
pub use state::{Alert, AlertLevel, Apply, ServiceState, Submission, SubmissionState};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub send_messages: bool,
    pub receive_messages: bool,
    pub alerts: bool,
    // Call transport is reserved. The desktop dialer remains offline.
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendMessage {
    pub request_id: u64,
    pub to: String,
    pub body: String,
}
/// Reserved contract only. No app action submits these commands yet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum CallCommand {
    Dial { request_id: u64, to: String },
    Answer { call_id: String },
    HangUp { call_id: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CallEvent {
    Incoming { call_id: String, from: String },
    Dialing { call_id: String, request_id: u64 },
    Connected { call_id: String },
    Ended { call_id: String, reason: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Command {
    Send(SendMessage),
    Call(CallCommand),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    MessageReceived {
        from: String,
        body: String,
        time_ms: u64,
    },
    MessageStatus {
        request_id: u64,
        state: SubmissionState,
    },
    AlertPublished {
        alert: Alert,
    },
    AlertWithdrawn {
        id: String,
    },
    /// Reserved; the current consumer rejects this without acknowledging it.
    Call(CallEvent),
}
/// One ordered durable stream per backend identity and local data directory.
/// Starts at 1. Replay is allowed; gaps are not. Ack only after an atomic save.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub sequence: u64,
    pub event: Event,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortError {
    Busy,
    Disconnected,
    Unsupported,
}
/// All methods must return promptly. IO belongs in the adapter's worker.
/// `submit` accepts work; it does NOT mean sent or delivered. The adapter must
/// deduplicate (profile identity, backend id, request_id), including after restart.
/// An unacknowledged envelope must be replayable after process failure.
pub trait Backend {
    /// Stable ASCII identity, 1..=64 bytes; changing it starts a separate stream.
    fn id(&self) -> &str;
    fn capabilities(&self) -> Capabilities;
    fn submit(&mut self, command: Command) -> Result<(), PortError>;
    fn poll(&mut self) -> Result<Option<Envelope>, PortError>;
    fn acknowledge(&mut self, sequence: u64) -> Result<(), PortError>;
}
pub struct Offline;
impl Backend for Offline {
    fn id(&self) -> &str {
        "offline"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    fn submit(&mut self, _: Command) -> Result<(), PortError> {
        Err(PortError::Unsupported)
    }
    fn poll(&mut self) -> Result<Option<Envelope>, PortError> {
        Ok(None)
    }
    fn acknowledge(&mut self, _: u64) -> Result<(), PortError> {
        Err(PortError::Unsupported)
    }
}

#[cfg(test)]
mod tests;
