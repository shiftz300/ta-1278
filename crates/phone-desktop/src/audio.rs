//! Audio device transport. The shell owns controls; the sink owns elapsed time.
pub(crate) mod midi;
pub(crate) mod native;
use crate::platform::{Playback, PlaybackStatus};
use crate::storage::files::Folder;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::{io::BufReader, time::Duration};
pub type AudioSource = Box<dyn Source<Item = f32> + Send>;
pub fn decode(files: &Folder, path: &str) -> Result<AudioSource, String> {
    if midi::player::is_file(path) {
        return midi::player::Player::open(files, path).map(|s| Box::new(s) as AudioSource);
    }
    let file = files.open(path).map_err(|e| e.to_string())?;
    Decoder::builder()
        .with_data(BufReader::new(file.reader))
        .with_byte_len(file.len)
        .with_seekable(true)
        .build()
        .map(|source| Box::new(source) as AudioSource)
        .map_err(|e| e.to_string())
}
#[cfg(test)]
mod native_tests {
    use super::*;
    #[test]
    #[ignore = "Requires the real CoreAudio output"]
    fn original_snake_and_clock_reach_the_output_and_stop() {
        use std::time::Instant;
        let mut audio = Audio::default();
        audio.feedback(crate::effects::Cue::Food, 1).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while audio.pending_effect.is_some() && Instant::now() < deadline {
            audio.poll_effects().unwrap();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(audio.effect.is_some());
        std::thread::sleep(Duration::from_millis(80));
        assert!(audio.effect.as_ref().unwrap().get_pos() > Duration::from_millis(20));
        audio.effect_volume(0);
        assert!(audio.effect.is_none() && audio.pending_effect.is_none());
        let files = Folder::new(std::env::temp_dir(), true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while audio.alarm.is_none() && Instant::now() < deadline {
            audio.alert(&files, Some("Clock1"), 1).unwrap();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(audio.alarm.is_some());
        std::thread::sleep(Duration::from_millis(80));
        assert!(!audio.alarm.as_ref().unwrap().empty());
        audio.alert(&files, None, 0).unwrap();
        assert!(audio.alarm.is_none());
        eprintln!("Original Snake food and Clock1 reached CoreAudio; mute and alert stop passed.");
    }
}
#[derive(Default)]
pub struct Audio {
    effect: Option<Sink>,
    native: crate::audio::native::NativeAudio,
    pending_effect: Option<(String, u8)>,
    alarm_failed: bool,
    alarm: Option<Sink>,
    alarm_tone: Option<String>,
    stream: Option<OutputStream>,
    sink: Option<Sink>,
    loaded: Option<(String, u64)>,
    seek_revision: u64,
}
impl Audio {
    pub fn effect_volume(&mut self, volume: u8) {
        if volume == 0 {
            self.effect = None;
            self.pending_effect = None;
        } else if let Some(sink) = &self.effect {
            sink.set_volume(volume as f32 / 10.);
        }
        if let Some((_, pending_volume)) = &mut self.pending_effect {
            *pending_volume = volume;
        }
    }
    fn output(&mut self) -> Result<&OutputStream, String> {
        if self.stream.is_none() {
            let mut stream =
                OutputStreamBuilder::open_default_stream().map_err(|e| e.to_string())?;
            stream.log_on_drop(false);
            self.stream = Some(stream);
        }
        Ok(self.stream.as_ref().unwrap())
    }
    pub fn feedback(&mut self, cue: crate::effects::Cue, volume: u8) -> Result<(), String> {
        self.pending_effect = Some((crate::effects::resource(cue).into(), volume));
        self.poll_effects()
    }
    pub fn poll_effects(&mut self) -> Result<(), String> {
        self.native.poll();
        let Some((key, volume)) = self.pending_effect.clone() else {
            return Ok(());
        };
        let clip = match self.native.get(&key) {
            Ok(Some(clip)) => clip,
            Ok(None) => return Ok(()),
            Err(e) => {
                self.pending_effect = None;
                return Err(e);
            }
        };
        self.pending_effect = None;
        let sink = Sink::connect_new(self.output()?.mixer());
        sink.set_volume(volume as f32 / 10.);
        sink.append(clip.take_duration(Duration::from_secs(15)));
        self.effect = Some(sink);
        Ok(())
    }
    pub fn preview(&mut self, files: &Folder, path: &str, volume: u8) -> Result<(), String> {
        self.pending_effect = None;
        self.effect = None;
        if path == "无" {
            return Ok(());
        }
        if !path.starts_with('/') {
            let key = if matches!(path, "预设" | "默认") {
                "Default Tune"
            } else {
                path
            };
            self.pending_effect = Some((key.into(), volume));
            return self.poll_effects();
        }
        let file = files.open(path).map_err(|e| e.to_string())?;
        if file.len > 20 * 1024 * 1024 {
            return Err("铃声文件超过 20 MiB。".into());
        }
        let source = decode(files, path)?.take_duration(Duration::from_secs(15));
        let sink = Sink::connect_new(self.output()?.mixer());
        sink.set_volume(volume as f32 / 10.);
        sink.append(source);
        self.effect = Some(sink);
        Ok(())
    }
    pub fn alert(&mut self, files: &Folder, tone: Option<&str>, volume: u8) -> Result<(), String> {
        let Some(tone) = tone.filter(|s| *s != "无" && volume > 0) else {
            self.alarm = None;
            self.alarm_tone = None;
            self.alarm_failed = false;
            return Ok(());
        };
        if self.alarm_tone.as_deref() != Some(tone) {
            self.alarm = None;
            self.alarm_tone = Some(tone.into());
            self.alarm_failed = false;
        }
        if self.alarm.is_none() && !self.alarm_failed {
            let result = (|| -> Result<Option<Sink>, String> {
                if tone.starts_with('/') {
                    if files.open(tone).map_err(|e| e.to_string())?.len > 20 * 1024 * 1024 {
                        return Err("铃声文件超过 20 MiB。".into());
                    }
                    let source = decode(files, tone)?
                        .take_duration(Duration::from_secs(15))
                        .repeat_infinite();
                    let sink = Sink::connect_new(self.output()?.mixer());
                    sink.set_volume(volume as f32 / 10.);
                    sink.append(source);
                    Ok(Some(sink))
                } else {
                    let key = if matches!(tone, "默认" | "预设") {
                        "Clock1"
                    } else {
                        tone
                    };
                    let Some(clip) = self.native.get(key)? else {
                        return Ok(None);
                    };
                    let sink = Sink::connect_new(self.output()?.mixer());
                    sink.set_volume(volume as f32 / 10.);
                    sink.append(clip.repeat_infinite());
                    Ok(Some(sink))
                }
            })();
            match result {
                Ok(sink) => self.alarm = sink,
                Err(e) => {
                    self.alarm_failed = true;
                    return Err(e);
                }
            }
        }
        if let Some(sink) = &self.alarm {
            sink.set_volume(volume as f32 / 10.);
        }
        Ok(())
    }
    pub fn stop(&mut self) {
        self.sink.take();
        self.loaded = None;
    }
    pub fn sync(
        &mut self,
        files: &Folder,
        path: Option<&str>,
        playback: Playback,
    ) -> PlaybackStatus {
        let mut status = PlaybackStatus::default();
        status.error = self.update(files, path, playback, &mut status).err();
        if status.error.is_some() {
            self.stop();
            status.stopped = true;
        }
        status
    }
    fn update(
        &mut self,
        files: &Folder,
        path: Option<&str>,
        playback: Playback,
        status: &mut PlaybackStatus,
    ) -> Result<(), String> {
        let Some(path) = path else {
            self.stop();
            status.stopped = true;
            return Ok(());
        };
        let changed = self
            .loaded
            .as_ref()
            .is_none_or(|(p, g)| p != path || *g != playback.generation);
        if changed {
            self.stop();
            if !playback.playing {
                return Ok(());
            }
            let source = decode(files, path)?;
            if let Some(duration) = source.total_duration() {
                status.duration = Some(duration.as_secs_f32());
            }
            let sink = Sink::connect_new(self.output()?.mixer());
            sink.pause();
            sink.set_volume(playback.volume as f32 / 10.);
            sink.append(source);
            // Restore through the sink so its clock starts at the saved position.
            if playback.position > 0. {
                sink.try_seek(Duration::from_secs_f32(playback.position))
                    .map_err(|e| e.to_string())?;
            }
            self.sink = Some(sink);
            self.loaded = Some((path.into(), playback.generation));
            self.seek_revision = playback.seek_revision;
        }
        let Some(sink) = &self.sink else {
            return Ok(());
        };
        sink.set_volume(playback.volume as f32 / 10.);
        if self.seek_revision != playback.seek_revision {
            sink.try_seek(Duration::from_secs_f32(playback.position))
                .map_err(|e| e.to_string())?;
            self.seek_revision = playback.seek_revision;
        }
        if playback.playing {
            sink.play();
        } else {
            sink.pause();
        }
        if playback.playing && sink.empty() {
            status.ended = true;
            self.stop();
        } else {
            status.position = Some(
                sink.get_pos()
                    .as_secs_f32()
                    .min(status.duration.unwrap_or(playback.duration)),
            );
        }
        Ok(())
    }
}

impl crate::platform::Sound for Audio {
    fn effect_volume(&mut self, volume: u8) {
        Self::effect_volume(self, volume);
    }
    fn feedback(&mut self, cue: crate::Cue, volume: u8) -> Result<(), String> {
        Self::feedback(self, cue, volume)
    }
    fn poll_effects(&mut self) -> Result<(), String> {
        Self::poll_effects(self)
    }
    fn preview(&mut self, files: &Folder, path: &str, volume: u8) -> Result<(), String> {
        Self::preview(self, files, path, volume)
    }
    fn alert(&mut self, files: &Folder, tone: Option<&str>, volume: u8) -> Result<(), String> {
        Self::alert(self, files, tone, volume)
    }
    fn stop(&mut self) {
        Self::stop(self);
    }
    fn sync(&mut self, files: &Folder, path: Option<&str>, playback: Playback) -> PlaybackStatus {
        Self::sync(self, files, path, playback)
    }
}
