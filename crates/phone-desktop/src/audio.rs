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
#[derive(Default)]
pub struct Audio {
    stream: Option<OutputStream>,
    sink: Option<Sink>,
    loaded: Option<(String, u64)>,
    seek_revision: u64,
}
impl Audio {
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
            if self.stream.is_none() {
                let mut stream =
                    OutputStreamBuilder::open_default_stream().map_err(|e| e.to_string())?;
                stream.log_on_drop(false);
                self.stream = Some(stream);
            }
            let sink = Sink::connect_new(self.stream.as_ref().unwrap().mixer());
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
