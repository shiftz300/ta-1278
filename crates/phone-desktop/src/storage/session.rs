//! Desktop wall-clock catch-up and periodic utility-file saves.
#[cfg(test)]
use super::utilities::Snapshot;
#[cfg(test)]
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(test)]
impl Snapshot {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let saved: Self = super::load_json(dir, "utilities.json", 128 * 1024)?;
        saved.validate()?;
        Ok(saved)
    }
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        self.validate()?;
        super::save_json(dir, "utilities.json", self, 128 * 1024)
    }
}
pub fn wall_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}
