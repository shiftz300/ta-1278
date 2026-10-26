mod atomic;
pub mod utilities;
use crate::phone::settings::Preferences;
pub use atomic::DataLock;
use phone_core::sms::Mailbox;
use std::path::{Path, PathBuf};
pub fn default_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("Library/Application Support/TA1278 Replica")
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
            })
            .join("ta1278-replica")
    }
}
pub fn load(dir: &Path) -> Result<Mailbox, String> {
    load_json(dir, "messages.json", 8 * 1024 * 1024)
}
pub fn save(dir: &Path, mailbox: &Mailbox) -> Result<(), String> {
    save_json(dir, "messages.json", mailbox, 8 * 1024 * 1024)
}
pub fn load_preferences(dir: &Path) -> Result<Preferences, String> {
    let prefs: Preferences = load_json(dir, "preferences.json", 65536)?;
    prefs.validate()?;
    Ok(prefs)
}
pub fn save_preferences(dir: &Path, prefs: &Preferences) -> Result<(), String> {
    prefs.validate()?;
    save_json(dir, "preferences.json", prefs, 65536)
}

pub fn load_browser(dir: &Path) -> Result<crate::phone::browser_data::BrowserData, String> {
    load_json(dir, "browser.json", 16 * 1024 * 1024)
}
pub fn save_browser(
    dir: &Path,
    data: &crate::phone::browser_data::BrowserData,
) -> Result<(), String> {
    save_json(dir, "browser.json", data, 16 * 1024 * 1024)
}
pub fn load_json<T: serde::de::DeserializeOwned + Default>(
    dir: &Path,
    name: &str,
    limit: u64,
) -> Result<T, String> {
    let Some(bytes) = atomic::read(&dir.join(name), limit)? else {
        return Ok(T::default());
    };
    serde_json::from_slice(&bytes).map_err(|e| format!("Cannot read {name} (retained): {e}"))
}
pub fn save_json<T: serde::Serialize>(
    dir: &Path,
    name: &str,
    value: &T,
    limit: usize,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    atomic::write(dir, name, &bytes, limit)
}

pub fn load_organizer(dir: &Path) -> Result<crate::phone::organizer::LocalData, String> {
    let data: crate::phone::organizer::LocalData =
        load_json(dir, "organizer.json", 8 * 1024 * 1024)?;
    if data.schema_version != 1
        || !data.reminders.valid()
        || data.groups.len() > 64
        || data
            .groups
            .iter()
            .any(|g| g.is_empty() || g.chars().count() > 128)
        || data.playlists.len() > 100
        || data.playlists.iter().any(|p| {
            p.name.is_empty()
                || p.name.chars().count() > 80
                || p.paths.len() > 1000
                || p.paths
                    .iter()
                    .any(|s| !s.starts_with('/') || s.len() > 4096)
        })
        || data.contacts.iter().any(|c| c.len() != 9)
        || data.alarms.iter().any(|a| a.fields.len() != 5)
        || data.events.iter().any(|e| {
            e.fields.len() != 7
                || phone_core::calendar::Date::new(e.date.year, e.date.month, e.date.day).is_none()
        })
    {
        return Err("Invalid organizer records; original retained".into());
    }
    Ok(data)
}
pub fn load_snake(dir: &Path) -> Result<crate::phone::snake::SnakeData, String> {
    let data: crate::phone::snake::SnakeData = load_json(dir, "snake.json", 4096)?;
    if !data.valid() {
        return Err("Invalid Snake records; original retained".into());
    }
    Ok(data)
}
#[cfg(test)]
mod tests;
