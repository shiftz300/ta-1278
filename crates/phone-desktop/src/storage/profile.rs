//! Default profile location and historical-format test fixtures.
//! Production record loading and commits are owned by Runtime/Store.
#[cfg(test)]
use super::{load_json, save_json, validate_organizer};
#[cfg(test)]
use crate::shell::settings::Preferences;
#[cfg(test)]
use phone_core::sms::Mailbox;
#[cfg(test)]
use std::path::Path;
#[cfg(feature = "desktop")]
use std::path::PathBuf;
#[cfg(feature = "desktop")]
pub fn default_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("Library/Application Support/TA1278 Replica")
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .or_else(|| std::env::var_os("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("USERPROFILE").unwrap_or_default())
                    .join("AppData/Local")
            })
            .join("TA1278 Replica")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
            })
            .join("ta1278-replica")
    }
}
#[cfg(test)]
pub fn load(dir: &Path) -> Result<Mailbox, String> {
    load_json(dir, "messages.json", 8 * 1024 * 1024)
}
#[cfg(test)]
pub fn save(dir: &Path, mailbox: &Mailbox) -> Result<(), String> {
    save_json(dir, "messages.json", mailbox, 8 * 1024 * 1024)
}
#[cfg(test)]
pub fn load_preferences(dir: &Path) -> Result<Preferences, String> {
    let mut prefs: Preferences = load_json(dir, "preferences.json", 65536)?;
    // Retired shortcut ID from older profiles. Keep all surviving IDs stable.
    for shortcut in [&mut prefs.left_shortcut, &mut prefs.right_shortcut] {
        if *shortcut == 17 {
            *shortcut = 0;
        }
    }
    prefs.validate()?;
    Ok(prefs)
}
#[cfg(test)]
pub fn save_preferences(dir: &Path, prefs: &Preferences) -> Result<(), String> {
    prefs.validate()?;
    save_json(dir, "preferences.json", prefs, 65536)
}

#[cfg(test)]
pub fn load_browser(dir: &Path) -> Result<crate::shell::browser::data::BrowserData, String> {
    load_json(dir, "browser.json", 16 * 1024 * 1024)
}
#[cfg(test)]
pub fn save_browser(
    dir: &Path,
    data: &crate::shell::browser::data::BrowserData,
) -> Result<(), String> {
    save_json(dir, "browser.json", data, 16 * 1024 * 1024)
}
#[cfg(test)]
pub fn load_organizer(dir: &Path) -> Result<crate::shell::organizer::LocalData, String> {
    let data: crate::shell::organizer::LocalData =
        load_json(dir, "organizer.json", 8 * 1024 * 1024)?;
    validate_organizer(&data)?;
    Ok(data)
}
#[cfg(test)]
pub fn load_snake(dir: &Path) -> Result<crate::shell::snake::SnakeData, String> {
    let data: crate::shell::snake::SnakeData = load_json(dir, "snake.json", 4096)?;
    if !data.valid() {
        return Err("Invalid Snake records; original retained".into());
    }
    Ok(data)
}
