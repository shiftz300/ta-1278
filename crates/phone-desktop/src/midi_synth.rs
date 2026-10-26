//! Original SF2 samples and SMF events, synthesized locally by OxiSynth.
//! Rate, polyphony, interpolation and effect algorithms are replica choices.
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use oxisynth::{MidiEvent, SoundFont, Synth, SynthDescriptor};
use std::io::Cursor;

pub const RATE: u32 = 32000;
const MAX_FRAMES: usize = RATE as usize * 60;
struct Event {
    frame: usize,
    message: MidiEvent,
}
struct Sequence {
    events: Vec<Event>,
    end: usize,
}

// Stable track order at equal ticks preserves program changes on shared channels.
fn sequence(bytes: &[u8]) -> Result<Sequence, String> {
    if bytes.len() > 128 * 1024 {
        return Err("MIDI 文件超过限制。".into());
    }
    let smf = Smf::parse(bytes).map_err(|e| e.to_string())?;
    if smf.header.format == Format::Sequential || smf.tracks.len() > 64 {
        return Err("不支持此 MIDI 曲目格式。".into());
    }
    let Timing::Metrical(ppq) = smf.header.timing else {
        return Err("不支持 SMPTE MIDI 时间轴。".into());
    };
    let ppq = u128::from(ppq.as_int());
    if ppq == 0 {
        return Err("MIDI 时间单位无效。".into());
    }
    let mut raw = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u64;
        for event in track {
            tick += u64::from(event.delta.as_int());
            raw.push((tick, event.kind));
            if raw.len() > 32768 {
                return Err("MIDI 事件过多。".into());
            }
            if matches!(event.kind, TrackEventKind::Meta(MetaMessage::EndOfTrack)) {
                break;
            }
        }
    }
    raw.sort_by_key(|e| e.0);
    let mut events = Vec::new();
    let (mut previous, mut time, mut tempo) = (0, 0u128, 500000u128);
    let mut end = 0;
    for (tick, kind) in raw {
        time += u128::from(tick - previous) * tempo;
        previous = tick;
        let frame = time * u128::from(RATE) / (ppq * 1_000_000);
        if frame > (MAX_FRAMES - RATE as usize * 2) as u128 {
            return Err("MIDI 时长超过限制。".into());
        }
        let frame = frame as usize;
        end = end.max(frame);
        let message = match kind {
            TrackEventKind::Meta(MetaMessage::Tempo(value)) => {
                tempo = u128::from(value.as_int());
                if tempo == 0 {
                    return Err("MIDI 速度无效。".into());
                }
                None
            }
            TrackEventKind::Midi { channel, message } => {
                let channel = channel.as_int();
                Some(match message {
                    MidiMessage::NoteOff { key, .. } => MidiEvent::NoteOff {
                        channel,
                        key: key.as_int(),
                    },
                    MidiMessage::NoteOn { key, vel } => MidiEvent::NoteOn {
                        channel,
                        key: key.as_int(),
                        vel: vel.as_int(),
                    },
                    MidiMessage::Aftertouch { key, vel } => MidiEvent::PolyphonicKeyPressure {
                        channel,
                        key: key.as_int(),
                        value: vel.as_int(),
                    },
                    MidiMessage::Controller { controller, value } => MidiEvent::ControlChange {
                        channel,
                        ctrl: controller.as_int(),
                        value: value.as_int(),
                    },
                    MidiMessage::ProgramChange { program } => MidiEvent::ProgramChange {
                        channel,
                        program_id: program.as_int(),
                    },
                    MidiMessage::ChannelAftertouch { vel } => MidiEvent::ChannelPressure {
                        channel,
                        value: vel.as_int(),
                    },
                    MidiMessage::PitchBend { bend } => MidiEvent::PitchBend {
                        channel,
                        value: bend.0.as_int(),
                    },
                })
            }
            TrackEventKind::SysEx(data)
                if data.len() == 5 && data[0] == 0x7e && data[2..] == [9, 1, 0xf7] =>
            {
                Some(MidiEvent::SystemReset)
            }
            TrackEventKind::SysEx(_) | TrackEventKind::Escape(_) => {
                return Err("此 MIDI 含尚未实现的系统专用事件。".into());
            }
            _ => None,
        };
        if let Some(message) = message {
            events.push(Event { frame, message });
        }
    }
    Ok(Sequence { events, end })
}

pub struct MidiSynth {
    synth: Synth,
}
impl MidiSynth {
    pub fn new() -> Result<Self, String> {
        let bytes = phone_core::embedded::decode(crate::packed::SOUNDFONT)?;
        let font =
            SoundFont::load(&mut Cursor::new(bytes)).map_err(|e| format!("原机音色库：{e:?}"))?;
        let mut synth = Synth::new(SynthDescriptor {
            sample_rate: RATE as f32,
            polyphony: 64,
            gain: 0.2,
            ..Default::default()
        })
        .map_err(|e| format!("MIDI 合成器：{e:?}"))?;
        synth.add_font(font, true);
        Ok(Self { synth })
    }
    fn send(&mut self, message: MidiEvent) -> Result<(), String> {
        self.synth.send_event(message).map_err(|e| e.to_string())?;
        if message == MidiEvent::SystemReset {
            // OxiSynth's reset initializes every channel to piano. Reapply GM drums.
            self.synth.reset_program();
        }
        Ok(())
    }
    pub fn render(&mut self, bytes: &[u8]) -> Result<Vec<f32>, String> {
        let sequence = sequence(bytes)?;
        self.send(MidiEvent::SystemReset)?;
        // Drain the engine's previous partial 64-frame output block after reset.
        self.synth.write(&mut [0.; 128][..]);
        let mut samples = Vec::with_capacity((sequence.end + RATE as usize) * 2);
        let mut cursor = 0;
        for event in sequence.events {
            self.append(&mut samples, event.frame - cursor)?;
            cursor = event.frame;
            self.send(event.message)?;
        }
        self.append(&mut samples, sequence.end - cursor)?;
        for channel in 0..16 {
            self.send(MidiEvent::ControlChange {
                channel,
                ctrl: 64,
                value: 0,
            })?;
            self.send(MidiEvent::AllNotesOff { channel })?;
        }
        // Preserve release/effect tails, stopping after 100 ms of near silence.
        // This bounded policy is local; the native DAC stop policy is unknown.
        let mut quiet = 0;
        for _ in 0..200 {
            let start = samples.len();
            self.append(&mut samples, RATE as usize / 100)?;
            if samples[start..].iter().all(|v| v.abs() < 0.00003) {
                quiet += 1;
            } else {
                quiet = 0;
            }
            if quiet >= 10 {
                break;
            }
        }
        Ok(samples)
    }
    fn append(&mut self, samples: &mut Vec<f32>, frames: usize) -> Result<(), String> {
        if samples.len() / 2 + frames > MAX_FRAMES {
            return Err("MIDI 合成结果超过限制。".into());
        }
        let start = samples.len();
        samples.resize(start + frames * 2, 0.);
        self.synth.write(&mut samples[start..]);
        if samples[start..].iter().any(|s| !s.is_finite()) {
            return Err("MIDI 合成结果包含无效采样。".into());
        }
        // Device output saturates at full scale; don't normalize individual clips.
        for sample in &mut samples[start..] {
            *sample = sample.clamp(-1., 1.);
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn smf(tracks: &[&[u8]]) -> Vec<u8> {
        let mut data = b"MThd\0\0\0\x06\0\x01\0\0\0\x60".to_vec();
        data[10..12].copy_from_slice(&(tracks.len() as u16).to_be_bytes());
        for track in tracks {
            data.extend_from_slice(b"MTrk");
            data.extend_from_slice(&(track.len() as u32).to_be_bytes());
            data.extend_from_slice(track);
        }
        data
    }
    #[test]
    fn tempo_map_and_equal_tick_program_order() {
        let data = smf(&[
            &[
                0, 0xff, 0x51, 3, 7, 0xa1, 0x20, 96, 0xff, 0x51, 3, 0x0f, 0x42, 0x40, 96, 0xff,
                0x2f, 0,
            ],
            &[
                0, 0xc0, 115, 0, 0x90, 60, 80, 96, 0x80, 60, 0, 96, 0xff, 0x2f, 0,
            ],
            &[0, 0xc0, 113, 0, 0xff, 0x2f, 0],
        ]);
        let seq = sequence(&data).unwrap();
        assert_eq!(seq.end, RATE as usize * 3 / 2);
        assert!(matches!(
            seq.events[0].message,
            MidiEvent::ProgramChange {
                program_id: 115,
                ..
            }
        ));
        assert!(matches!(
            seq.events[2].message,
            MidiEvent::ProgramChange {
                program_id: 113,
                ..
            }
        ));
        assert_eq!(seq.events[3].frame, RATE as usize / 2);
    }
    #[test]
    fn malformed_and_excessive_input_is_rejected_before_synthesis() {
        assert!(sequence(b"not midi").is_err());
        assert!(sequence(&vec![0; 128 * 1024 + 1]).is_err());
        assert!(sequence(&smf(&[&[0, 0xf0, 2, 1, 0xf7]])).is_err());
        assert!(sequence(&smf(&[&[0xff, 0xff, 0xff, 0x7f, 0xff, 0x2f, 0]])).is_err());
    }
    #[test]
    fn native_bank_has_all_gm_programs_and_drums_and_reset_clears_notes() {
        let mut synth = MidiSynth::new().unwrap();
        // select_program validates actual bank entries, not fallback presets.
        let id = synth.synth.program(0).unwrap().0.unwrap();
        for program in 0..128 {
            synth.synth.select_program(0, id, 0, program).unwrap();
        }
        synth.synth.select_program(9, id, 128, 0).unwrap();
        let bytes = include_bytes!("../../../assets/audio/snake-food.mid");
        let first = synth.render(bytes).unwrap();
        assert!(first.iter().any(|v| v.abs() > 0.001));
        synth
            .render(include_bytes!("../../../assets/audio/snake-over.mid"))
            .unwrap();
        let second = synth.render(bytes).unwrap();
        // Engine phase/effect state need not be bit-identical after reuse, but no
        // notes, pedals, programs or old effect tails may leak into a new clip.
        let energy = |s: &[f32]| s.iter().map(|v| v * v).sum::<f32>();
        assert!((energy(&first) - energy(&second)).abs() / energy(&first) < 0.01);
    }
}
