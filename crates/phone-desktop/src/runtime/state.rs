//! Versioned user data, independent of the host's file format and save location.
//! Restore starts at Home. This is not a process snapshot or a rewind of external IO.
use super::*;
use crate::{
    phone::{
        browser::data::BrowserData, organizer::LocalData, settings::Preferences, snake::SnakeData,
    },
    storage,
};
use phone_core::{
    predict::firmware::Data,
    sms::{Mailbox, Message},
};
use serde::{Deserialize, Serialize};
const LIMIT: usize = 32 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    version: u32,
    mailbox: Mailbox,
    preferences: Preferences,
    organizer: LocalData,
    browser: BrowserData,
    snake: SnakeData,
    utilities: storage::utilities::Snapshot,
    input: Data,
    draft: Option<(Option<usize>, Message)>,
}
impl Runtime {
    /// Atomically persist the supplied bytes in the host's save transaction.
    /// Only Ok means durable success; outgoing requests and received-event acknowledgments
    /// remain blocked otherwise. A failed pending Send restores its editable draft.
    /// Unconfirmed IME candidates and unfinished organizer forms are not silently committed.
    pub fn save(
        &mut self,
        persist: impl FnOnce(&[u8]) -> Result<(), String>,
    ) -> Result<(), String> {
        if self.phone.files.busy
            || self
                .phone
                .files
                .pending
                .as_ref()
                .is_some_and(crate::phone::files::FileRequest::mutating)
        {
            return Err("Wait for the file operation before saving".into());
        }
        if self
            .phone
            .editor()
            .is_some_and(|editor| editor.prediction.active())
        {
            return Err("Confirm or cancel the input candidate before saving".into());
        }
        let (input, revision) = self.predictor.snapshot();
        let draft = (matches!(self.phone.screen, Screen::Compose | Screen::Conversation)
            && self.phone.mail_ui.pending.is_none())
        .then(|| (self.phone.compose.draft, self.phone.message()));
        let state = State {
            version: 1,
            mailbox: self.phone.mailbox.clone(),
            preferences: self.phone.preferences.clone(),
            organizer: self.phone.local_data.clone(),
            browser: self.phone.browser_data.clone(),
            snake: self.phone.snake_data.clone(),
            utilities: storage::utilities::Snapshot::capture(&self.phone.tools, self.time.unix_ms),
            input,
            draft,
        };
        let result = serde_json::to_vec(&state)
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                if bytes.len() > LIMIT {
                    return Err("Phone state exceeds 32 MiB".into());
                }
                persist(&bytes)
            });
        if let Err(error) = result {
            self.phone.storage_failed();
            return Err(error);
        }
        self.phone.storage_saved();
        self.predictor.saved(revision);
        self.phone.preferences_dirty = false;
        self.phone.local_dirty = false;
        self.phone.browser_dirty = false;
        self.phone.snake_dirty = false;
        self.phone.tools.dirty = false;
        Ok(())
    }
    /// Restore into a fresh instance: stale browser responses/adapters cannot affect it.
    /// Timers resume from their saved remaining time, without offline catch-up. The host
    /// must separately reconcile external backend state before polling after a rollback.
    pub fn restore(
        config: RuntimeConfig,
        time: Time,
        bytes: &[u8],
        backend: Box<dyn Backend>,
    ) -> Result<Self, String> {
        time.validate()?;
        if config.version != 1 {
            return Err("Unsupported runtime config version".into());
        }
        if bytes.len() > LIMIT {
            return Err("Phone state exceeds 32 MiB".into());
        }
        let state: State = serde_json::from_slice(bytes)
            .map_err(|error| format!("Invalid phone state: {error}"))?;
        if state.version != 1 {
            return Err("Unsupported phone state version".into());
        }
        state.preferences.validate()?;
        storage::validate_organizer(&state.organizer)?;
        state.utilities.validate()?;
        if !state.snake.valid() {
            return Err("Invalid Snake records".into());
        }
        let predictor = Arc::new(Firmware::from_data(state.input)?);
        let mut phone = Phone::new(state.mailbox);
        if let Some((index, message)) = state.draft {
            if message.to.encode_utf16().count() > 64 || message.body.encode_utf16().count() > 4096
            {
                return Err("Invalid saved draft".into());
            }
            if !message.to.is_empty() || !message.body.is_empty() {
                phone.mailbox.save(index, message);
            }
        }
        phone.features = config.effective_features();
        phone.apps = config.apps;
        phone.apply_preferences(state.preferences);
        phone.local_data = state.organizer;
        phone.browser_data = state.browser;
        phone.apply_snake_data(state.snake);
        state
            .utilities
            .apply(&mut phone.tools, state.utilities.saved_at_ms);
        phone.music.tracks.clear();
        phone.music.external_clock = true;
        let mut runtime = Self::from_phone(phone, predictor, time)?;
        runtime.connect(backend);
        Ok(runtime)
    }
}
