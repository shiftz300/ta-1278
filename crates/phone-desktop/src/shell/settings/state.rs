//! State owned by this application.
use super::*;

pub struct State {
    pub preferences: Preferences,
    pub dirty: bool,
    pub page: SettingsPage,
    pub first: usize,
    pub preview: usize,
    pub accent: usize,
}
impl Default for State {
    fn default() -> Self {
        Self {
            preferences: Preferences::default(),
            dirty: false,
            page: SettingsPage::Root,
            first: 0,
            preview: 1,
            accent: crate::theme::Theme::load().startup_fallback_accent,
        }
    }
}
