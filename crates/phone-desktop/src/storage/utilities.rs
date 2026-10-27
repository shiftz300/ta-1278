//! Versioned utility state. Running clocks include time spent outside the app.
use crate::shell::tools::Tools;
use phone_core::utilities::{Stopwatch, Timer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub stopwatch: Stopwatch,
    pub timer: Timer,
    pub timer_seconds: u64,
    pub timer_name: String,
    pub frequency: u16,
    pub stations: Vec<u16>,
    pub radio_speaker: bool,
}
impl Default for Data {
    fn default() -> Self {
        Self::capture(&Tools::default())
    }
}
impl Data {
    fn capture(tools: &Tools) -> Self {
        Self {
            stopwatch: tools.stopwatch.clone(),
            timer: tools.timer.clone(),
            timer_seconds: tools.timer_seconds,
            timer_name: tools.timer_name.clone(),
            frequency: tools.frequency,
            stations: tools.stations.clone(),
            radio_speaker: tools.radio_speaker,
        }
    }
    fn valid(&self) -> bool {
        self.stopwatch.laps.len() <= 100
            && self
                .stopwatch
                .laps
                .iter()
                .all(|lap| *lap <= self.stopwatch.elapsed_ms)
            && self.stopwatch.laps.windows(2).all(|v| v[0] >= v[1])
            && self.timer_seconds <= 359999
            && self.timer.remaining_ms <= 359999000
            && (!self.timer.expired || (!self.timer.running && self.timer.remaining_ms == 0))
            && (!self.timer.running || self.timer.remaining_ms > 0)
            && self.timer_name.chars().count() <= 256
            && (875..=1080).contains(&self.frequency)
            && self.stations.len() <= 206
            && self.stations.iter().all(|f| (875..=1080).contains(f))
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Snapshot {
    pub schema_version: u32,
    pub saved_at_ms: u64,
    pub data: Data,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            schema_version: 1,
            saved_at_ms: 0,
            data: Data::default(),
        }
    }
}
impl Snapshot {
    pub fn capture(tools: &Tools, now: u64) -> Self {
        Self {
            schema_version: 1,
            saved_at_ms: now,
            data: Data::capture(tools),
        }
    }
    pub fn apply(&self, tools: &mut Tools, now: u64) {
        let data = &self.data;
        tools.stopwatch = data.stopwatch.clone();
        tools.timer = data.timer.clone();
        tools.timer_seconds = data.timer_seconds;
        tools.timer_name = data.timer_name.clone();
        tools.frequency = data.frequency;
        tools.stations = data.stations.clone();
        tools.radio_speaker = data.radio_speaker;
        let elapsed = if self.saved_at_ms == 0 {
            0
        } else {
            now.saturating_sub(self.saved_at_ms)
        };
        tools.stopwatch.tick(elapsed);
        tools.timer.tick(elapsed);
        tools.timer_notice = false;
        tools.dirty = Data::capture(tools) != self.data;
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || !self.data.valid() || self.saved_at_ms > 253402300799999 {
            Err("Unsupported or invalid utility data; original retained".into())
        } else {
            Ok(())
        }
    }
}
