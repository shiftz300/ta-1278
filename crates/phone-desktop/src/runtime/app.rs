//! Application-side effects and durability policy, below the public Runtime API.
//! Runtime retains one coordinator; platform adapters only provide I/O mechanisms.
mod bridge;
mod persist;
mod state;
use super::*;
use crate::{services::Services, storage};

#[derive(Default)]
pub(super) struct App {
    services: Services,
    attached: bool,
    media: Option<crate::media::Media>,
    request: Option<Request>,
    last_periodic: u64,
    utilities: Option<storage::utilities::Data>,
    music: Option<crate::shell::music::SavedMusic>,
}
