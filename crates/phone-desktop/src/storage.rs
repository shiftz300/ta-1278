mod atomic;
pub(crate) mod files;
#[cfg(any(feature = "desktop", test))]
mod profile;
#[cfg(feature = "desktop")]
pub(crate) mod setup;
#[cfg(any(feature = "desktop", test))]
pub use profile::*;
#[cfg(any(feature = "desktop", test))]
pub mod session;
pub mod utilities;
#[cfg(any(feature = "desktop", test))]
pub use atomic::DataLock;
use std::path::Path;
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

pub(crate) fn validate_organizer(data: &crate::phone::organizer::LocalData) -> Result<(), String> {
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
        || data.calls.len() > crate::phone::calls::LIMIT
        || data
            .calls
            .iter()
            .any(|r| !crate::phone::calls::valid(&r.number))
        || data.contacts.iter().any(|c| c.len() != 9)
        || data.alarms.iter().any(|a| a.fields.len() != 5)
        || data.events.iter().any(|e| {
            e.fields.len() != 7
                || phone_core::calendar::Date::new(e.date.year, e.date.month, e.date.day).is_none()
        })
    {
        return Err("Invalid organizer records; original retained".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests;
