//! One app-state persistence driver for desktop records and embedded checkpoints.
use super::{state::State, *};
use crate::persistence::{self, Resume};
use crate::platform::{Record, Store};
use crate::{shell, storage};

impl Runtime {
    pub fn load_profile(
        config: RuntimeConfig,
        time: Time,
        store: &mut dyn Store,
        backend: Box<dyn Backend>,
        resume: Resume,
    ) -> Result<Self, String> {
        let mut preferences: crate::shell::settings::Preferences =
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
        runtime.app.utilities = Some(saved_utilities);
        runtime.app.music = Some(saved_music);
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
            self.shell.status = format!("输入法错误：{error}");
        }
        let periodic = force || self.time.elapsed_ms.saturating_sub(self.app.last_periodic) >= 5000;
        if !periodic && !self.unsaved() {
            return Ok(());
        }
        let music = self.shell.playback.snapshot();
        let utilities = storage::utilities::Snapshot::capture(&self.shell.tools, self.time.unix_ms);
        macro_rules! commit {
            ($key:ident, $condition:expr, $value:expr, $success:block, $label:literal) => {
                if $condition {
                    match persistence::write(store, Record::$key, $value) {
                        Ok(()) => $success,
                        Err(error) => {
                            let message = format!(concat!($label, "：{}"), error);
                            if Record::$key == Record::Music {
                                self.shell.notice(message.clone());
                            }
                            return Err(message);
                        }
                    }
                }
            };
        }
        commit!(
            Music,
            periodic && self.app.music.as_ref() != Some(&music),
            &music,
            {
                self.app.music = Some(music.clone());
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
            (periodic || self.shell.tools.dirty)
                && self.app.utilities.as_ref() != Some(&utilities.data),
            &utilities,
            {
                self.app.utilities = Some(utilities.data.clone());
                self.shell.tools.dirty = false;
            },
            "计时数据保存失败"
        );
        self.shell.tools.dirty = false;
        commit!(
            Snake,
            self.shell.gaming.dirty,
            &self.shell.gaming.data,
            {
                self.shell.gaming.dirty = false;
            },
            "游戏保存失败"
        );
        commit!(
            Organizer,
            self.shell.records_dirty,
            &self.shell.records(),
            {
                self.shell.records_dirty = false;
            },
            "保存失败"
        );
        commit!(
            Browser,
            self.shell.browsing.dirty,
            &self.shell.browsing.data,
            {
                self.shell.browsing.dirty = false;
            },
            "浏览记录保存失败"
        );
        commit!(
            Preferences,
            self.shell.settings.dirty,
            &self.shell.settings.preferences,
            {
                self.shell.settings.dirty = false;
            },
            "设置保存失败"
        );
        if self.shell.messaging.dirty {
            match persistence::write(store, Record::Messages, &self.shell.messaging.mailbox) {
                Ok(()) => self.shell.storage_saved(),
                Err(error) => {
                    self.shell.storage_failed();
                    self.shell
                        .notice(format!("Save failed; data remains in memory. {error}"));
                    return Err(format!("Save failed: {error}"));
                }
            }
        }
        if periodic {
            self.app.last_periodic = self.time.elapsed_ms;
        }
        self.shell.tools.dirty = false;
        Ok(())
    }
    pub fn unsaved(&self) -> bool {
        self.pending_commit()
            || self.shell.settings.dirty
            || self.shell.browsing.dirty
            || self.shell.records_dirty
            || self.shell.gaming.dirty
            || self.shell.tools.dirty
            || self.predictor.pending_save().is_some()
    }
    /// Shared window/host close policy: commit drafts, retain the editor after
    /// failures and do not quit during mutating file operations.
    pub fn close(&mut self, store: &mut dyn crate::platform::Store) -> bool {
        if self.shell.files.busy
            || self
                .shell
                .files
                .pending
                .as_ref()
                .is_some_and(shell::files::FileRequest::mutating)
        {
            self.shell.notice("文件操作尚未结束，请稍候。");
            return false;
        }
        if self.shell.messaging.ui.pending.is_some() {
            if let Err(error) = self.flush(store, false) {
                self.report(error);
            }
            // Failed queue writes roll back and leave the original editor open.
            if matches!(
                self.shell.screen,
                shell::Screen::Compose | shell::Screen::Conversation
            ) {
                return false;
            }
        }
        if matches!(
            self.shell.screen,
            shell::Screen::Compose | shell::Screen::Conversation
        ) {
            if !self.shell.messaging.compose.body.accept_prediction()
                || !self.shell.messaging.compose.to.accept_prediction()
            {
                self.shell
                    .notice("Text limit reached. Confirm or cancel the candidate before closing.");
                return false;
            }
            self.shell.save_draft();
        }
        if let Err(error) = self.flush(store, false) {
            self.report(error);
        }
        if self.unsaved() {
            self.shell
                .notice("Save failed. Window stays open so you can keep your text.");
            return false;
        }
        if let Err(error) = self.flush(store, true) {
            self.shell.notice(format!("{error}，窗口保持打开。"));
            return false;
        }
        true
    }
}
