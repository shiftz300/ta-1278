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

/// Build device intent without exposing mutable application state.
pub fn playback(model: &Music) -> crate::platform::Playback {
    crate::platform::Playback {
        playing: model.playing,
        position: model.position,
        duration: model.duration(),
        volume: model.volume,
        generation: model.generation,
        seek_revision: model.seek_revision,
    }
}
/// Apply transport observations; repeat/shuffle/queue policy stays in the app model.
pub fn apply_playback(
    model: &mut Music,
    status: crate::platform::PlaybackStatus,
) -> Result<(), String> {
    if status.duration.is_some_and(|v| !v.is_finite() || v < 0.)
        || status.position.is_some_and(|v| !v.is_finite() || v < 0.)
    {
        model.playing = false;
        return Err("Invalid playback clock".into());
    }
    if let Some(duration) = status.duration
        && let Some(track) = model.tracks.get_mut(model.index)
    {
        track.1 = duration;
    }
    if let Some(error) = status.error {
        model.playing = false;
        return Err(error);
    }
    if status.stopped {
        model.playing = false;
    } else if status.ended {
        model.ended();
    } else if let Some(position) = status.position {
        model.position = position.min(model.duration());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlaybackStatus;

    #[test]
    fn device_eof_uses_application_queue_and_repeat_policy() {
        let mut model = Music::default();
        model.queue.extend([2, 0]);
        model.repeat = Repeat::All;
        model.play(2);
        let request = playback(&model);
        assert_eq!(request.generation, model.generation);
        let status = PlaybackStatus {
            ended: true,
            ..Default::default()
        };
        assert_eq!(model.index, 2); // A device report alone does not change the app.
        apply_playback(&mut model, status).unwrap();
        assert_eq!(model.index, 0);
        assert!(model.playing);
        model.repeat = Repeat::One;
        apply_playback(
            &mut model,
            PlaybackStatus {
                ended: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!((model.index, model.position, model.playing), (0, 0., true));
    }

    #[test]
    fn device_failure_preserves_position_and_partial_metadata_without_advancing_queue() {
        let mut model = Music::default();
        model.play(0);
        model.position = 12.;
        let result = apply_playback(
            &mut model,
            PlaybackStatus {
                duration: Some(100.),
                error: Some("lost output".into()),
                stopped: true,
                ..Default::default()
            },
        );
        assert_eq!(result.unwrap_err(), "lost output");
        assert_eq!(
            (model.position, model.playing, model.duration()),
            (12., false, 100.)
        );
        model.playing = true;
        assert!(
            apply_playback(
                &mut model,
                PlaybackStatus {
                    position: Some(f32::NAN),
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!((model.position, model.playing), (12., false));
    }
}
