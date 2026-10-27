//! Verified original audio inputs; MIDI uses the embedded handset SF2 bank.
use rodio::{Decoder, Source, buffer::SamplesBuffer};
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
    thread,
};
pub struct Sound {
    pub key: &'static str,
    pub name: &'static str,
    pub midi: bool,
    pub bytes: &'static [u8],
}
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/audio/catalog.rs"
));
pub fn sound(key: &str) -> Option<&'static Sound> {
    SOUNDS
        .iter()
        .find(|s| s.key == key || s.name == key)
        .or_else(|| {
            if key == "Clock 1" {
                SOUNDS.iter().find(|s| s.key == "Clock1")
            } else {
                None
            }
        })
}
pub fn tone_labels(default: &str) -> Vec<String> {
    std::iter::once(default.to_owned())
        .chain(std::iter::once("无".into()))
        .chain(RINGTONES.iter().map(|s| (*s).to_owned()))
        .chain(std::iter::once("从文件选择".into()))
        .collect()
}
pub fn file_index() -> usize {
    RINGTONES.len() + 2
}
struct Decoded {
    clip: SamplesBuffer,
    bytes: usize,
}
pub struct NativeAudio {
    jobs: mpsc::Sender<&'static Sound>,
    results: mpsc::Receiver<(&'static str, Result<Decoded, String>)>,
    pending: HashSet<&'static str>,
    cache: HashMap<&'static str, Result<Decoded, String>>,
    bytes: usize,
}
impl Default for NativeAudio {
    fn default() -> Self {
        let (jobs, rx) = mpsc::channel::<&'static Sound>();
        let (tx, results) = mpsc::channel();
        thread::spawn(move || {
            let mut synth = None;
            while let Ok(sound) = rx.recv() {
                let decoded = decode(sound, &mut synth);
                if tx.send((sound.key, decoded)).is_err() {
                    break;
                }
            }
        });
        Self {
            jobs,
            results,
            pending: HashSet::new(),
            cache: HashMap::new(),
            bytes: 0,
        }
    }
}
impl NativeAudio {
    pub fn poll(&mut self) {
        while let Ok((key, result)) = self.results.try_recv() {
            self.pending.remove(key);
            if let Ok(decoded) = &result {
                if self.bytes + decoded.bytes > 32 * 1024 * 1024 {
                    self.cache.clear();
                    self.bytes = 0;
                }
                self.bytes += decoded.bytes;
            }
            self.cache.insert(key, result);
        }
    }
    pub fn get(&mut self, key: &str) -> Result<Option<SamplesBuffer>, String> {
        self.poll();
        let sound = sound(key).ok_or_else(|| format!("未知预设铃声：{key}"))?;
        if let Some(result) = self.cache.get(sound.key) {
            return match result {
                Ok(d) => Ok(Some(d.clip.clone())),
                Err(e) => Err(e.clone()),
            };
        }
        if self.pending.insert(sound.key) {
            self.jobs
                .send(sound)
                .map_err(|_| "声音解码服务已停止。".to_owned())?;
        }
        Ok(None)
    }
}
fn decode(
    sound: &Sound,
    synth: &mut Option<crate::audio::midi::MidiSynth>,
) -> Result<Decoded, String> {
    if sound.midi {
        if synth.is_none() {
            *synth = Some(crate::audio::midi::MidiSynth::new()?);
        }
        let samples = synth.as_mut().unwrap().render(sound.bytes)?;
        let bytes = samples.len() * 4;
        return Ok(Decoded {
            clip: SamplesBuffer::new(2, crate::audio::midi::RATE, samples),
            bytes,
        });
    }
    let decoder =
        Decoder::try_from(std::io::Cursor::new(sound.bytes)).map_err(|e| e.to_string())?;
    let rate = decoder.sample_rate();
    let channels = decoder.channels();
    if rate > 48000 || channels > 2 || channels == 0 {
        return Err("原机音频格式超出限制。".into());
    }
    let limit = rate as usize * channels as usize * 60;
    let samples: Vec<f32> = decoder.take(limit + 1).collect();
    if samples.len() > limit || samples.iter().any(|s| !s.is_finite()) {
        return Err("原机音频长度或采样值无效。".into());
    }
    let bytes = samples.len() * 4;
    Ok(Decoded {
        clip: SamplesBuffer::new(channels, rate, samples),
        bytes,
    })
}
/// Offline research preview. Refuses to overwrite an existing file.
#[cfg(feature = "desktop")]
pub fn export(key: &str, path: &std::path::Path) -> Result<(), String> {
    use std::io::Write;
    let entry = sound(key).ok_or_else(|| format!("Unknown tone: {key}"))?;
    let clip = decode(entry, &mut None)?.clip;
    let rate = clip.sample_rate();
    let channels = clip.channels();
    let len = clip.clone().count() as u32 * 2;
    let mut wav = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?,
    );
    let result = (|| -> std::io::Result<()> {
        wav.write_all(b"RIFF")?;
        wav.write_all(&(36 + len).to_le_bytes())?;
        wav.write_all(b"WAVEfmt ")?;
        wav.write_all(&16u32.to_le_bytes())?;
        wav.write_all(&1u16.to_le_bytes())?;
        wav.write_all(&channels.to_le_bytes())?;
        wav.write_all(&rate.to_le_bytes())?;
        wav.write_all(&(rate * u32::from(channels) * 2).to_le_bytes())?;
        wav.write_all(&(channels * 2).to_le_bytes())?;
        wav.write_all(&16u16.to_le_bytes())?;
        wav.write_all(b"data")?;
        wav.write_all(&len.to_le_bytes())?;
        for sample in clip {
            wav.write_all(&((sample.clamp(-1., 1.) * 32767.).round() as i16).to_le_bytes())?;
        }
        wav.flush()
    })();
    result.map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_click_decodes_without_a_device_or_midi_renderer() {
        let clip = decode(sound("key").unwrap(), &mut None).unwrap().clip;
        assert!(clip.total_duration().unwrap().as_secs_f32() < 0.2);
        assert!(clip.clone().any(|s| s.abs() > 0.01));
        assert!(sound("Clock1").unwrap().midi);
        assert_eq!(RINGTONES.len(), 21);
    }
    #[test]
    fn original_midi_resources_render_notes_and_decode() {
        let mut synth = None;
        for entry in SOUNDS.iter().filter(|s| s.midi) {
            let key = entry.key;
            let d = decode(entry, &mut synth).unwrap();
            let duration = d.clip.total_duration().unwrap().as_secs_f32();
            assert!(duration > 0.2 && duration < 60.);
            assert!(d.clip.clone().any(|s| s.abs() > 0.001));
            eprintln!(
                "NATIVE MIDI PASS: {key}, {duration:.3}s, {} bytes decoded",
                d.bytes
            );
        }
    }
}
