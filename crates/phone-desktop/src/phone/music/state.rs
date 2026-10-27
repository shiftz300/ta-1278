//! State owned by this application.
use super::*;

#[derive(Default)]
pub struct State {
    pub model: Music,
    pub playlists: Vec<Playlist>,
    pub resume: Option<SavedMusic>,
    pub paths: Vec<String>,
    pub lookup: std::collections::HashMap<String, usize>,
    pub playlist: usize,
    pub row: usize,
    pub refresh: bool,
    pub loading: bool,
    pub wait_dismissed: bool,
    pub return_file: bool,
    pub view: MusicView,
    pub started: bool,
}

#[derive(Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SavedMusic {
    pub queue: Vec<String>,
    pub volume: Option<u8>,
    pub repeat: u8,
    pub shuffle: bool,
    pub path: Option<String>,
    pub position: f32,
}
impl SavedMusic {
    pub fn validate(&self) -> Result<(), String> {
        if self.volume.is_some_and(|v| v > 10)
            || self.repeat > 2
            || !self.position.is_finite()
            || self.position < 0.
        {
            return Err("Invalid music settings; original retained".into());
        }
        Ok(())
    }
}
impl State {
    pub fn restore(&mut self, saved: SavedMusic) -> Result<(), String> {
        saved.validate()?;
        self.model.volume = saved.volume.unwrap_or(6);
        self.model.repeat = [Repeat::Off, Repeat::All, Repeat::One][saved.repeat as usize];
        self.model.shuffle = saved.shuffle;
        self.resume = Some(saved);
        Ok(())
    }
    pub fn snapshot(&self) -> SavedMusic {
        let mut saved = SavedMusic {
            queue: self
                .model
                .queue
                .iter()
                .filter_map(|i| self.paths.get(*i).cloned())
                .collect(),
            volume: Some(self.model.volume),
            repeat: match self.model.repeat {
                Repeat::Off => 0,
                Repeat::All => 1,
                Repeat::One => 2,
            },
            shuffle: self.model.shuffle,
            path: self
                .started
                .then(|| self.paths.get(self.model.index))
                .flatten()
                .cloned(),
            position: self.model.position,
        };
        if let Some(pending) = &self.resume {
            saved.path = pending.path.clone();
            saved.position = pending.position;
            saved.queue = pending.queue.clone();
        }
        saved
    }
}
