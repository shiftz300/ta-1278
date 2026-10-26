//! Audio device transport. The phone owns controls; the sink owns elapsed time.
use phone_core::music::Music;
use phone_vfs::Vfs;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::{io::BufReader, time::Duration};
pub type AudioDecoder = Decoder<BufReader<Box<dyn phone_vfs::ReadSeek>>>;
pub fn decode(vfs: &Vfs, path: &str) -> Result<AudioDecoder, String> {
    let file = vfs.open(path).map_err(|e| e.to_string())?;
    Decoder::builder()
        .with_data(BufReader::new(file.reader))
        .with_byte_len(file.len)
        .with_seekable(true)
        .build()
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
        let vfs = Vfs::default();
        let deadline = Instant::now() + Duration::from_secs(10);
        while audio.alarm.is_none() && Instant::now() < deadline {
            audio.alert(&vfs, Some("Clock1"), 1).unwrap();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(audio.alarm.is_some());
        std::thread::sleep(Duration::from_millis(80));
        assert!(!audio.alarm.as_ref().unwrap().empty());
        audio.alert(&vfs, None, 0).unwrap();
        assert!(audio.alarm.is_none());
        eprintln!("Original Snake food and Clock1 reached CoreAudio; mute and alert stop passed.");
    }
}
#[derive(Default)]
pub struct Audio {
    effect: Option<Sink>,
    native: crate::native_audio::NativeAudio,
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
    pub fn preview(&mut self, vfs: &Vfs, path: &str, volume: u8) -> Result<(), String> {
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
        let file = vfs.open(path).map_err(|e| e.to_string())?;
        if file.len > 20 * 1024 * 1024 {
            return Err("铃声文件超过 20 MiB。".into());
        }
        let source = decode(vfs, path)?.take_duration(Duration::from_secs(15));
        let sink = Sink::connect_new(self.output()?.mixer());
        sink.set_volume(volume as f32 / 10.);
        sink.append(source);
        self.effect = Some(sink);
        Ok(())
    }
    pub fn alert(&mut self, vfs: &Vfs, tone: Option<&str>, volume: u8) -> Result<(), String> {
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
                    if vfs.open(tone).map_err(|e| e.to_string())?.len > 20 * 1024 * 1024 {
                        return Err("铃声文件超过 20 MiB。".into());
                    }
                    let source = decode(vfs, tone)?
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
    pub fn sync(&mut self, vfs: &Vfs, path: Option<&str>, music: &mut Music) -> Result<(), String> {
        let result = self.update(vfs, path, music);
        if result.is_err() {
            self.stop();
            music.playing = false;
        }
        result
    }
    fn update(&mut self, vfs: &Vfs, path: Option<&str>, music: &mut Music) -> Result<(), String> {
        let Some(path) = path else {
            self.stop();
            music.playing = false;
            return Ok(());
        };
        let changed = self
            .loaded
            .as_ref()
            .is_none_or(|(p, g)| p != path || *g != music.generation);
        if changed {
            self.stop();
            if !music.playing {
                return Ok(());
            }
            let source = decode(vfs, path)?;
            if let Some(duration) = source.total_duration() {
                music.tracks[music.index].1 = duration.as_secs_f32();
            }
            let sink = Sink::connect_new(self.output()?.mixer());
            sink.pause();
            sink.set_volume(music.volume as f32 / 10.);
            sink.append(source);
            // Restore through the sink so its clock starts at the saved position.
            if music.position > 0. {
                sink.try_seek(Duration::from_secs_f32(music.position))
                    .map_err(|e| e.to_string())?;
            }
            self.sink = Some(sink);
            self.loaded = Some((path.into(), music.generation));
            self.seek_revision = music.seek_revision;
        }
        let Some(sink) = &self.sink else {
            return Ok(());
        };
        sink.set_volume(music.volume as f32 / 10.);
        if self.seek_revision != music.seek_revision {
            sink.try_seek(Duration::from_secs_f32(music.position))
                .map_err(|e| e.to_string())?;
            self.seek_revision = music.seek_revision;
        }
        if music.playing {
            sink.play();
        } else {
            sink.pause();
        }
        if music.playing && sink.empty() {
            music.ended();
            self.stop();
        } else {
            music.position = sink.get_pos().as_secs_f32().min(music.duration());
        }
        Ok(())
    }
}
