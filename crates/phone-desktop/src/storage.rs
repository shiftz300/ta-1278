use crate::phone::settings::Preferences;
use phone_core::sms::Mailbox;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
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
    let path = dir.join("messages.json");
    if !path.exists() {
        return Ok(Mailbox::default());
    }
    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() > 8 * 1024 * 1024 {
        return Err("Local store exceeds 8 MiB; original retained".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Cannot read local store (retained): {e}"))
}
pub fn save(dir: &Path, mailbox: &Mailbox) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(mailbox).map_err(|e| e.to_string())?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("Local store is full; delete some entries".into());
    }
    let tmp = dir.join("messages.json.tmp");
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut f = options.open(&tmp).map_err(|e| e.to_string())?;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    std::fs::rename(tmp, dir.join("messages.json")).map_err(|e| e.to_string())
}

pub fn load_preferences(dir: &Path) -> Result<Preferences, String> {
    let path = dir.join("preferences.json");
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Preferences::default()),
        Err(e) => return Err(e.to_string()),
    };
    if bytes.len() > 65536 {
        return Err("Preferences file exceeds limit; original retained".into());
    }
    let prefs: Preferences = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Cannot read preferences (retained): {e}"))?;
    prefs.validate()?;
    Ok(prefs)
}
pub fn save_preferences(dir: &Path, prefs: &Preferences) -> Result<(), String> {
    prefs.validate()?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(prefs).map_err(|e| e.to_string())?;
    let path = dir.join("preferences.json.tmp");
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut f = options.open(&path).map_err(|e| e.to_string())?;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    std::fs::rename(path, dir.join("preferences.json")).map_err(|e| e.to_string())
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
    let path = dir.join(name);
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(T::default()),
        Err(e) => return Err(e.to_string()),
    };
    if meta.len() > limit {
        return Err(format!("{name} exceeds limit; original retained"));
    }
    serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Cannot read {name} (retained): {e}"))
}
pub fn save_json<T: serde::Serialize>(
    dir: &Path,
    name: &str,
    value: &T,
    limit: usize,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!("{name} exceeds limit; original retained"));
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!("{name}.tmp"));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut f = options.open(&tmp).map_err(|e| e.to_string())?;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    std::fs::rename(tmp, dir.join(name)).map_err(|e| e.to_string())
}

pub fn load_organizer(dir: &Path) -> Result<crate::phone::organizer::LocalData, String> {
    let data: crate::phone::organizer::LocalData =
        load_json(dir, "organizer.json", 8 * 1024 * 1024)?;
    if data.groups.len() > 64
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
mod tests {
    use super::*;
    #[test]
    fn save_reload_and_corrupt_file_is_retained() {
        let dir = std::env::temp_dir().join(format!("ta1278-store-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut m = Mailbox::default();
        m.save(
            None,
            phone_core::sms::Message {
                to: "123".into(),
                body: "中文".into(),
                ..Default::default()
            },
        );
        save(&dir, &m).unwrap();
        assert_eq!(load(&dir).unwrap().drafts, m.drafts);
        std::fs::write(dir.join("messages.json"), b"{bad").unwrap();
        assert!(load(&dir).is_err());
        assert_eq!(std::fs::read(dir.join("messages.json")).unwrap(), b"{bad");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn preferences_roundtrip_and_invalid_values_preserve_original() {
        let dir =
            std::env::temp_dir().join(format!("ta1278-preferences-test-{}", std::process::id()));
        let prefs = Preferences {
            accent: 4,
            left_shortcut: 3,
            right_shortcut: 12,
            ..Preferences::default()
        };
        save_preferences(&dir, &prefs).unwrap();
        let restored = load_preferences(&dir).unwrap();
        assert_eq!(
            (
                restored.accent,
                restored.left_shortcut,
                restored.right_shortcut
            ),
            (4, 3, 12)
        );
        let invalid = b"{\"accent\":99}";
        std::fs::write(dir.join("preferences.json"), invalid).unwrap();
        assert!(load_preferences(&dir).is_err());
        assert_eq!(
            std::fs::read(dir.join("preferences.json")).unwrap(),
            invalid
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
