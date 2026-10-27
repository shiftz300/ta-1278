//! Desktop wall-clock catch-up and periodic utility-file saves.
use super::utilities::{Data, Snapshot};
use crate::phone::tools::Tools;
use std::{
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
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
pub struct Session {
    saved: Data,
    last_check: Instant,
}
impl Session {
    pub fn load(dir: &Path, tools: &mut Tools) -> Result<Self, String> {
        let snapshot = Snapshot::load(dir)?;
        snapshot.apply(tools, wall_ms());
        Ok(Self {
            saved: snapshot.data,
            last_check: Instant::now(),
        })
    }
    pub fn save(&mut self, dir: &Path, tools: &mut Tools, force: bool) -> Result<(), String> {
        if !force && !tools.dirty && self.last_check.elapsed() < Duration::from_secs(5) {
            return Ok(());
        }
        let current = Snapshot::capture(tools, wall_ms());
        if current.data != self.saved {
            current.save(dir)?;
            self.saved = current.data;
        }
        tools.dirty = false;
        self.last_check = Instant::now();
        Ok(())
    }
}
