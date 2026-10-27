//! Streaming SMF playback through the existing SF2 synth; no device or full-song PCM buffer.
use super::*;
use crate::storage::files::Folder;
use rodio::{Source, source::SeekError};
use std::{io::Read, time::Duration};

const MAX_BYTES: u64 = 2 * 1024 * 1024;
const BLOCK: usize = 256;

pub fn is_file(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .is_some_and(|e| e.eq_ignore_ascii_case("mid") || e.eq_ignore_ascii_case("midi"))
}
fn read(files: &Folder, path: &str) -> Result<Sequence, String> {
    let file = files.open(path).map_err(|e| e.to_string())?;
    if file.len > MAX_BYTES {
        return Err("MIDI 文件超过 2 MiB。".into());
    }
    let mut bytes = Vec::new();
    file.reader
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    parse(&bytes, true)
}
pub fn duration(files: &Folder, path: &str) -> Result<Duration, String> {
    Ok(length(&read(files, path)?))
}
fn length(sequence: &Sequence) -> Duration {
    Duration::from_secs_f64((sequence.end + RATE as usize * 2) as f64 / RATE as f64)
}

pub struct Player {
    sequence: Sequence,
    engine: MidiSynth,
    event: usize,
    rendered: usize,
    delivered: usize,
    released: bool,
    buffer: [f32; BLOCK * 2],
    offset: usize,
    len: usize,
}
impl Player {
    pub fn open(files: &Folder, path: &str) -> Result<Self, String> {
        Self::new(read(files, path)?)
    }
    fn new(sequence: Sequence) -> Result<Self, String> {
        let mut player = Self {
            sequence,
            engine: MidiSynth::new()?,
            event: 0,
            rendered: 0,
            delivered: 0,
            released: false,
            buffer: [0.; BLOCK * 2],
            offset: 0,
            len: 0,
        };
        player.prime()?;
        Ok(player)
    }
    fn prime(&mut self) -> Result<(), String> {
        self.engine.send(MidiEvent::SystemReset)?;
        self.engine.synth.write(&mut [0.; 128][..]);
        Ok(())
    }
    fn frames(&self) -> usize {
        self.sequence.end + RATE as usize * 2
    }
    fn fill(&mut self) -> Result<bool, String> {
        if self.rendered >= self.frames() {
            return Ok(false);
        }
        while let Some(event) = self.sequence.events.get(self.event)
            && event.frame <= self.rendered
        {
            self.engine.send(event.message)?;
            self.event += 1;
        }
        if self.rendered >= self.sequence.end && !self.released {
            for channel in 0..16 {
                self.engine.send(MidiEvent::ControlChange {
                    channel,
                    ctrl: 64,
                    value: 0,
                })?;
                self.engine.send(MidiEvent::AllNotesOff { channel })?;
            }
            self.released = true;
        }
        let next = self
            .sequence
            .events
            .get(self.event)
            .map_or(self.frames(), |e| e.frame);
        let boundary = if self.released {
            self.frames()
        } else {
            self.sequence.end
        };
        let count = BLOCK
            .min(next - self.rendered)
            .min(boundary - self.rendered);
        self.len = count * 2;
        self.offset = 0;
        self.engine.synth.write(&mut self.buffer[..self.len]);
        for sample in &mut self.buffer[..self.len] {
            if !sample.is_finite() {
                return Err("MIDI 合成结果包含无效采样。".into());
            }
            *sample = sample.clamp(-1., 1.);
        }
        self.rendered += count;
        Ok(true)
    }
    fn seek(&mut self, samples: usize) -> Result<(), String> {
        let target = samples.min(self.frames() * 2);
        if target == self.frames() * 2 {
            self.delivered = target;
            self.rendered = self.frames();
            self.offset = 0;
            self.len = 0;
            return Ok(());
        }
        if target < self.delivered {
            self.engine = MidiSynth::new()?;
            self.event = 0;
            self.rendered = 0;
            self.delivered = 0;
            self.released = false;
            self.offset = 0;
            self.len = 0;
            self.prime()?;
        }
        // Re-synthesize discarded samples to preserve held notes, controllers and effect state.
        while self.delivered < target {
            if self.offset == self.len && !self.fill()? {
                break;
            }
            let count = (target - self.delivered).min(self.len - self.offset);
            self.offset += count;
            self.delivered += count;
        }
        Ok(())
    }
}
impl Iterator for Player {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.offset == self.len && !self.fill().ok()? {
            return None;
        }
        let value = self.buffer[self.offset];
        self.offset += 1;
        self.delivered += 1;
        Some(value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some((self.frames() * 2).saturating_sub(self.delivered)))
    }
}
impl Source for Player {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        2
    }
    fn sample_rate(&self) -> u32 {
        RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(length(&self.sequence))
    }
    fn try_seek(&mut self, position: Duration) -> Result<(), SeekError> {
        let frame = (position.as_secs_f64() * RATE as f64).min(self.frames() as f64) as usize;
        // Keep interleaved channel phase if the mixer seeks between left/right samples.
        self.seek(frame * 2 + self.delivered % 2)
            .map_err(|e| SeekError::Other(Box::new(std::io::Error::other(e))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song(track: &[u8]) -> Vec<u8> {
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\0\x60MTrk".to_vec();
        bytes.extend_from_slice(&(track.len() as u32).to_be_bytes());
        bytes.extend_from_slice(track);
        bytes
    }
    #[test]
    fn streaming_matches_native_synthesis_and_seeks_without_a_device() {
        let bytes = song(&[
            0, 0xc0, 10, 0, 0x90, 60, 80, 96, 0xb0, 64, 127, 0, 0x80, 60, 0, 96, 0xb0, 64, 0, 0,
            0xff, 0x2f, 0,
        ]);
        let expected = MidiSynth::new().unwrap().render(&bytes).unwrap();
        let mut player = Player::new(parse(&bytes, true).unwrap()).unwrap();
        let actual: Vec<_> = player.by_ref().collect();
        assert_eq!(actual.len(), RATE as usize * 3 * 2);
        assert!(actual.iter().all(|v| v.is_finite() && v.abs() <= 1.));
        assert!(actual.iter().any(|v| v.abs() > 0.001));
        assert!(
            expected
                .iter()
                .zip(&actual)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        );
        assert!(player.next().is_none());
        for milliseconds in [250, 800, 100, 1500] {
            player
                .try_seek(Duration::from_millis(milliseconds))
                .unwrap();
            let start = milliseconds as usize * RATE as usize / 1000 * 2;
            let chunk: Vec<_> = player.by_ref().take(1000).collect();
            assert!(
                chunk
                    .iter()
                    .zip(&actual[start..])
                    .all(|(a, b)| a.to_bits() == b.to_bits())
            );
            assert_eq!(chunk.len(), 1000);
        }
        player.try_seek(Duration::MAX).unwrap();
        assert!(player.next().is_none());
        player.try_seek(Duration::ZERO).unwrap();
        assert_eq!(player.next(), actual.first().copied());
    }
    #[test]
    fn whole_songs_have_bounded_buffers_and_explicit_format_limits() {
        // 19,200 ticks = 100 seconds; this exceeds the short-effect renderer limit.
        let bytes = song(&[
            0, 0x90, 60, 80, 0x81, 0x96, 0, 0x80, 60, 0, 0, 0xff, 0x2f, 0,
        ]);
        assert!(sequence(&bytes).is_err());
        let player = Player::new(parse(&bytes, true).unwrap()).unwrap();
        assert_eq!(player.total_duration(), Some(Duration::from_secs(102)));
        assert_eq!(player.buffer.len(), 512);
        assert!(parse(&vec![0; MAX_BYTES as usize + 1], true).is_err());
        assert!(parse(&song(&[0xff, 0xff, 0xff, 0x7f, 0xff, 0x2f, 0]), true).is_err());
        let mut sequential = bytes.clone();
        sequential[9] = 2;
        assert!(parse(&sequential, true).is_err());
        let mut smpte = bytes;
        smpte[12..14].copy_from_slice(&[0xe8, 40]);
        assert!(parse(&smpte, true).is_err());
        let vendor = song(&[0, 0xf0, 2, 1, 0xf7, 96, 0xff, 0x2f, 0]);
        assert!(parse(&vendor, true).is_ok());
        assert!(sequence(&vendor).is_err());
    }
}
