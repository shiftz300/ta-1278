# Native audio and MIDI synthesis

The replica now synthesizes the handset's original MIDI resources with its **original SF2 sound bank**. It no longer uses the macOS General MIDI instrument bank or sine-wave cues. Synthesis runs on a worker; cached PCM is played through the existing Rodio output. No handset firmware code is executed.

## Verified firmware data

The source is the existing 17.00.17.02 `user.bin`, SHA-256 `8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca`.

| Input | File offset | Length | Evidence |
|---|---:|---:|---|
| SF2 `gm1_A21.sf2` | `0x7aa358` | 2,586,984 | Literal at `0xa51020`; `0x88a50ea4` passes the bank to `0x882b2c52`, which forwards it to the RIFF/sfbk parser at `0x88a5545e`. |
| Snake normal food SMF | `0x2037f8` | 274 | Normal fruit increments score and calls sound event 5 at `0x8869079c`. |
| Snake end SMF | `0x20390a` | 1,744 | End handler calls sound event 6 at `0x88690c28`. |

The sound dispatcher at `0x8869054c` provides the SMF pointers and lengths. Bonus score handling has no call to this dispatcher; the replica's food cue follows body growth, not every score change. [Bounded listings](evidence/native-audio-code.txt).

The unmodified SF2 has version 2.1, 128 melodic GM presets, one bank-128 percussion preset, 129 instruments and 412 samples, plus terminal records. It retains sample loops, root keys, tuning, generators and instrument modulators. Its SHA-256 is `9309119628275607526a94cf746f25a7ca5a6d688feeba9e7991b02008414ef0`. [Complete preset/sample/modulator index](evidence/native-soundfont.json).

The existing sound resource directory has 75 entries. We expose 21 original preset names plus the two Snake SMFs and one original MP3 click. Clock1, Clock 2 and Message 2 pairings have matching SMF metadata; the remaining name-to-resource pairings are based on matching list order and remain **inferences**. Key-click association and default message/timer bindings are local policy. Fourteen adjacent short MP3 entries are byte-identical clicks, not distinct DTMF tones. [Per-entry provenance and hashes](../assets/audio/manifest.json).

## Independent synthesis

[OxiSynth 0.1.0](https://github.com/PolyMeilex/OxiSynth) interprets SF2 samples, envelopes, filters and modulators. [midly 0.5.3](https://github.com/negamartin/midly) parses SMF events. Both come from pinned crates.io releases; source checksums and license/source archives are retained. RustySynth was inspected but not selected because its instrument-region import uses generators without importing custom SF2 modulators present in this bank.

The scheduler merges format-0/1 tracks, preserves track order at equal ticks, integrates tempo changes without per-event rounding drift, and forwards program, note, velocity, pressure, controller and pitch-bend events. Channel 10 uses the recovered percussion bank. GM reset restores the bank and controllers. Unsupported SysEx, format 2 and SMPTE timing return errors rather than silently approximating them; all 23 bundled SMFs are supported.

The bank is loaded lazily once per worker. Synthesis and decoding do not block UI rendering. PCM caching is bounded to 32 MiB, each input to 128 KiB, 64 tracks, 32,768 events and 58 seconds of timeline plus up to 2 seconds of release. Preview stops on Back/Home, mute drops pending playback, and alarm stop drops its repeating sink. Ordinary music remains streamed MP3/WAV; this change does not add arbitrary MIDI files to the music browser.

**Local choices:** 32 kHz stereo, 64 voices, OxiSynth's default interpolation/reverb/chorus, gain 0.2, output saturation and a near-silence release cutoff. Original DAC rate, voice stealing, fixed-point rounding, effects, speaker response, stop policy and every native controller branch have not been recovered. This is original notes and samples through an independent synthesizer, **not proven bit-identical native DSP**. No physical audio recording was made in this pass.

## Reproduce

```sh
python3 scripts/recover-audio.py
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --release --locked -p phone-desktop
# New files only: existing output files are never overwritten.
target/release/phone-desktop --render-tone snake-food /tmp/ta1278-food.wav
target/release/phone-desktop --render-tone snake-over /tmp/ta1278-over.wav
target/release/phone-desktop --render-tone Clock1 /tmp/ta1278-clock.wav
# Optional real audio device check; plays short quiet excerpts.
cargo test -p phone-desktop --locked original_snake_and_clock_reach_the_output_and_stop -- --ignored --nocapture
```

The recovery script verifies source-image hashes, all 75 source sounds, exact Snake pointers/lengths and the SF2 structure/hash. It writes data/indexes and static listings only. The synthesizer test exercises every GM preset and the drum bank, all bundled MIDIs, tempo ordering, bounds and state reset. Host synthesis/output tests do not establish acoustic parity with the phone.

## Listening previews

[Normal food](audio/snake-food.wav) · [Snake end](audio/snake-over.wav). These are independent reconstruction outputs, not handset recordings. [Checksums](audio/SHA256.json). The signed build passed 127 default tests, the opt-in CoreAudio output test, Clippy and 118 window scenes; see [validation manifest](evidence/native-audio-validation.json).
