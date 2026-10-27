//! One app-state persistence driver for desktop records and embedded checkpoints.
use super::{state::State, *};
use crate::persistence::{self, Record, Resume, Store};
use crate::storage;

#[derive(Default)]
pub(super) struct Session {
    last_periodic: u64,
    utilities: Option<storage::utilities::Data>,
    music: Option<crate::phone::music::SavedMusic>,
}
impl Runtime {
    pub fn load_profile(
        config: RuntimeConfig,
        time: Time,
        store: &mut dyn Store,
        backend: Box<dyn Backend>,
        resume: Resume,
    ) -> Result<Self, String> {
        let mut preferences: crate::phone::settings::Preferences =
            persistence::read(store, Record::Preferences)?;
        for shortcut in [
            &mut preferences.left_shortcut,
            &mut preferences.right_shortcut,
        ] {
            if *shortcut == 17 {
                *shortcut = 0;
            }
        }
        let state = State {
            version: 1,
            mailbox: persistence::read(store, Record::Messages)?,
            preferences,
            organizer: persistence::read(store, Record::Organizer)?,
            browser: persistence::read(store, Record::Browser)?,
            snake: persistence::read(store, Record::Snake)?,
            utilities: persistence::read(store, Record::Utilities)?,
            input: persistence::read(store, Record::Input)?,
            music: persistence::read(store, Record::Music)?,
            draft: None,
        };
        let saved_utilities = state.utilities.data.clone();
        let saved_music = state.music.clone();
        let mut runtime = Self::from_state(config, time, state, backend, resume)?;
        runtime.persistence.utilities = Some(saved_utilities);
        runtime.persistence.music = Some(saved_music);
        Ok(runtime)
    }
    pub fn restore_from(
        config: RuntimeConfig,
        time: Time,
        store: &mut dyn Store,
        backend: Box<dyn Backend>,
    ) -> Result<Self, String> {
        match store.read(Record::Snapshot)? {
            Some(bytes) => Self::restore(config, time, &bytes, backend),
            None => Self::with_backend(config, time, backend),
        }
    }
    /// Background persistence preserves uncommitted editors. Only successfully
    /// written sections are acknowledged; a failed mailbox write rolls back Send.
    /// Periodic clocks/media remain five-second checkpoints, as before.
    pub fn flush(&mut self, store: &mut dyn Store, force: bool) -> Result<(), String> {
        if let Some(error) = self.predictor.take_error() {
            self.phone.status = format!("输入法错误：{error}");
        }
        let periodic = force
            || self
                .time
                .elapsed_ms
                .saturating_sub(self.persistence.last_periodic)
                >= 5000;
        if !periodic && !self.unsaved() {
            return Ok(());
        }
        let music = self.phone.playback.snapshot();
        let utilities = storage::utilities::Snapshot::capture(&self.phone.tools, self.time.unix_ms);
        macro_rules! commit {
            ($key:ident, $condition:expr, $value:expr, $success:block, $label:literal) => {
                if $condition {
                    match persistence::write(store, Record::$key, $value) {
                        Ok(()) => $success,
                        Err(error) => {
                            let message = format!(concat!($label, "：{}"), error);
                            if Record::$key == Record::Music {
                                self.phone.notice(message.clone());
                            }
                            return Err(message);
                        }
                    }
                }
            };
        }
        commit!(
            Music,
            periodic && self.persistence.music.as_ref() != Some(&music),
            &music,
            {
                self.persistence.music = Some(music.clone());
            },
            "音乐设置保存失败"
        );
        if let Some((input, revision)) = self.predictor.pending_save() {
            commit!(
                Input,
                true,
                &input,
                {
                    self.predictor.saved(revision);
                },
                "输入记录保存失败"
            );
        }
        commit!(
            Utilities,
            (periodic || self.phone.tools.dirty)
                && self.persistence.utilities.as_ref() != Some(&utilities.data),
            &utilities,
            {
                self.persistence.utilities = Some(utilities.data.clone());
                self.phone.tools.dirty = false;
            },
            "计时数据保存失败"
        );
        self.phone.tools.dirty = false;
        commit!(
            Snake,
            self.phone.gaming.dirty,
            &self.phone.gaming.data,
            {
                self.phone.gaming.dirty = false;
            },
            "游戏保存失败"
        );
        commit!(
            Organizer,
            self.phone.records_dirty,
            &self.phone.records(),
            {
                self.phone.records_dirty = false;
            },
            "保存失败"
        );
        commit!(
            Browser,
            self.phone.browsing.dirty,
            &self.phone.browsing.data,
            {
                self.phone.browsing.dirty = false;
            },
            "浏览记录保存失败"
        );
        commit!(
            Preferences,
            self.phone.settings.dirty,
            &self.phone.settings.preferences,
            {
                self.phone.settings.dirty = false;
            },
            "设置保存失败"
        );
        if self.phone.messaging.dirty {
            match persistence::write(store, Record::Messages, &self.phone.messaging.mailbox) {
                Ok(()) => self.phone.storage_saved(),
                Err(error) => {
                    self.phone.storage_failed();
                    self.phone
                        .notice(format!("Save failed; data remains in memory. {error}"));
                    return Err(format!("Save failed: {error}"));
                }
            }
        }
        if periodic {
            self.persistence.last_periodic = self.time.elapsed_ms;
        }
        self.phone.tools.dirty = false;
        Ok(())
    }
    pub fn unsaved(&self) -> bool {
        self.pending_commit()
            || self.phone.settings.dirty
            || self.phone.browsing.dirty
            || self.phone.records_dirty
            || self.phone.gaming.dirty
            || self.phone.tools.dirty
            || self.predictor.pending_save().is_some()
    }
}
