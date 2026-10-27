//! State owned by this application.
use super::*;

#[derive(Default)]
pub struct State {
    pub mailbox: Mailbox,
    pub backend: Option<String>,
    pub compose: Compose,
    pub ui: mail::MailUi,
    pub chat: conversation::ChatUi,
    pub dirty: bool,
    pub pending_close: bool,
}
