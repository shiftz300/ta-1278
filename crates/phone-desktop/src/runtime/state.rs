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
pub(super) struct State {
    pub(super) version: u32,
    pub(super) mailbox: Mailbox,
    pub(super) preferences: Preferences,
    pub(super) organizer: LocalData,
    pub(super) browser: BrowserData,
    pub(super) snake: SnakeData,
    pub(super) utilities: storage::utilities::Snapshot,
    pub(super) input: Data,
    pub(super) draft: Option<(Option<usize>, Message)>,
    #[serde(default)]
    pub(super) music: crate::phone::music::SavedMusic,
}
impl Runtime {
    /// Atomically persist the supplied bytes in the host's save transaction.
    /// Only Ok means durable success; outgoing requests and received-event acknowledgments
    /// remain blocked otherwise. A failed pending Send restores its editable draft.
    /// Unconfirmed IME candidates and unfinished organizer forms are not silently committed.
    pub fn save_to(&mut self, store: &mut dyn crate::persistence::Store) -> Result<(), String> {
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
        let (state, revision) = State::capture(self);
        let result = crate::persistence::write(store, crate::persistence::Record::Snapshot, &state);
        if let Err(error) = result {
            self.phone.storage_failed();
            return Err(error);
        }
        self.phone.storage_saved();
        self.predictor.saved(revision);
        self.phone.settings.dirty = false;
        self.phone.records_dirty = false;
        self.phone.browsing.dirty = false;
        self.phone.gaming.dirty = false;
        self.phone.tools.dirty = false;
        Ok(())
    }
    pub fn save(
        &mut self,
        persist: impl FnOnce(&[u8]) -> Result<(), String>,
    ) -> Result<(), String> {
        struct Callback<F>(Option<F>);
        impl<F: FnOnce(&[u8]) -> Result<(), String>> crate::persistence::Store for Callback<F> {
            fn read(&mut self, _: crate::persistence::Record) -> Result<Option<Vec<u8>>, String> {
                Ok(None)
            }
            fn write(&mut self, _: crate::persistence::Record, bytes: &[u8]) -> Result<(), String> {
                self.0.take().ok_or("Checkpoint already committed")?(bytes)
            }
        }
        self.save_to(&mut Callback(Some(persist)))
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
        Self::from_state(
            config,
            time,
            state,
            backend,
            crate::persistence::Resume::Paused,
        )
    }
    pub(super) fn from_state(
        config: RuntimeConfig,
        time: Time,
        state: State,
        backend: Box<dyn Backend>,
        resume: crate::persistence::Resume,
    ) -> Result<Self, String> {
        time.validate()?;
        if config.version != 1 {
            return Err("Unsupported runtime config version".into());
        }
        state.validate()?;
        let predictor = Arc::new(Firmware::from_data(state.input)?);
        let mut phone = Phone::new(state.mailbox);
        if let Some((index, message)) = state.draft {
            if message.to.encode_utf16().count() > 64 || message.body.encode_utf16().count() > 4096
            {
                return Err("Invalid saved draft".into());
            }
            if !message.to.is_empty() || !message.body.is_empty() {
                phone.messaging.mailbox.save(index, message);
            }
        }
        phone.features = config.effective_features();
        phone.apps = config.apps;
        phone.apply_preferences(state.preferences);
        phone.apply_records(state.organizer);
        phone.browsing.data = state.browser;
        phone.apply_snake_data(state.snake);
        state.utilities.apply(
            &mut phone.tools,
            match resume {
                crate::persistence::Resume::Paused => state.utilities.saved_at_ms,
                crate::persistence::Resume::WallClock => time.unix_ms,
            },
        );
        phone.playback.restore(state.music)?;
        phone.playback.model.tracks.clear();
        phone.playback.model.external_clock = true;
        let mut runtime = Self::from_phone(phone, predictor, time)?;
        runtime.connect(backend);
        Ok(runtime)
    }
}

impl State {
    pub(super) fn capture(runtime: &Runtime) -> (Self, u64) {
        let phone = &runtime.phone;
        let (input, revision) = runtime.predictor.snapshot();
        (
            Self {
                version: 1,
                mailbox: phone.messaging.mailbox.clone(),
                preferences: phone.settings.preferences.clone(),
                organizer: phone.records(),
                browser: phone.browsing.data.clone(),
                snake: phone.gaming.data.clone(),
                utilities: storage::utilities::Snapshot::capture(
                    &phone.tools,
                    runtime.time.unix_ms,
                ),
                input,
                draft: (matches!(phone.screen, Screen::Compose | Screen::Conversation)
                    && phone.messaging.ui.pending.is_none())
                .then(|| (phone.messaging.compose.draft, phone.message())),
                music: phone.playback.snapshot(),
            },
            revision,
        )
    }
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported phone state version".into());
        }
        self.preferences.validate()?;
        storage::validate_organizer(&self.organizer)?;
        self.utilities.validate()?;
        self.music.validate()?;
        if !self.snake.valid() {
            return Err("Invalid Snake records".into());
        }
        Ok(())
    }
}
