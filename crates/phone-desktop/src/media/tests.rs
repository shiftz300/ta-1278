use super::*;
use crate::audio::{self, Audio};
use crate::shell::{files::FileRequest, tools::ToolPage};
use rodio::Source;
use std::{fs, io::Write};
fn wav(path: &Path, seconds: u32) {
    let rate = 8000u32;
    let size = rate * seconds * 2;
    let mut f = fs::File::create(path).unwrap();
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(36 + size).to_le_bytes()).unwrap();
    f.write_all(b"WAVEfmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap();
    for n in [1u16, 1u16] {
        f.write_all(&n.to_le_bytes()).unwrap();
    }
    for n in [rate, rate * 2] {
        f.write_all(&n.to_le_bytes()).unwrap();
    }
    for n in [2u16, 16u16] {
        f.write_all(&n.to_le_bytes()).unwrap();
    }
    f.write_all(b"data").unwrap();
    f.write_all(&size.to_le_bytes()).unwrap();
    f.write_all(&vec![0; size as usize]).unwrap();
}
fn fixture(name: &str, read_only: bool) -> (Folder, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("ta1278-media-{name}-{}", std::process::id()));
    fs::create_dir_all(dir.join("card/Music")).unwrap();
    wav(&dir.join("card/Music/tone.wav"), 3);
    let files = Folder::new(&dir, read_only).unwrap();
    (files, dir)
}
#[test]
fn scan_decode_corrupt_and_stream_seek() {
    let (files, dir) = fixture("scan", false);
    fs::write(dir.join("card/bad.mp3"), b"not audio").unwrap();
    fs::write(dir.join("card/notes.txt"), b"notes").unwrap();
    let (tracks, errors) = scan(&files, &crate::platform::visual::SystemDecoder);
    assert_eq!(tracks.len(), 1);
    assert_eq!(errors.len(), 1);
    assert!((tracks[0].seconds - 3.).abs() < 0.01);
    let mut source = audio::decode(&files, &tracks[0].path).unwrap();
    source.try_seek(Duration::from_secs(2)).unwrap();
    assert!(source.count() <= 8001);
    assert_eq!(fs::read(dir.join("card/bad.mp3")).unwrap(), b"not audio");
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn background_move_updates_links_without_moving_another_apps_focus() {
    let (files, dir) = fixture("move-links", false);
    let mut p = Shell::new(Default::default());
    p.features.music = false;
    p.settings.preferences.ringtone = "/card/Music/tone.wav".into();
    p.playback.playlists.push(crate::shell::Playlist {
        name: "test".into(),
        paths: vec!["/card/Music/tone.wav".into()],
    });
    let mut media = Media::new(files, &dir, &mut p).unwrap();
    media
        .file_request(
            &mut p,
            FileRequest::Transfer("/card/Music".into(), "/card/Songs".into(), true),
        )
        .unwrap();
    p.go(Screen::Menu);
    p.index = 7;
    let deadline = Instant::now() + Duration::from_secs(5);
    while p.files.busy && Instant::now() < deadline {
        media.tick(&mut p);
        thread::sleep(Duration::from_millis(5));
    }
    assert!(!p.files.busy);
    assert_eq!(p.index, 7);
    assert_eq!(p.screen, Screen::Menu);
    assert_eq!(p.settings.preferences.ringtone, "/card/Songs/tone.wav");
    assert_eq!(p.playback.playlists[0].paths, ["/card/Songs/tone.wav"]);
    assert!(p.records_dirty && p.settings.dirty);
    assert!(!dir.join("card/Music").exists());
    assert!(dir.join("card/Songs/tone.wav").exists());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn file_actions_and_flags_reach_service_boundary() {
    let (files, dir) = fixture("files", true);
    let mut shell = Shell::new(Default::default());
    shell.features.music = false;
    let mut media = Media::new(files, &dir, &mut shell).unwrap();
    media
        .file_request(&mut shell, FileRequest::List("/card".into()))
        .unwrap();
    assert_eq!(shell.files.entries[0].name, "Music");
    assert!(!shell.files.writable);
    assert!(
        media
            .file_request(&mut shell, FileRequest::Mkdir("/card/no".into()))
            .is_err()
    );
    assert!(
        media
            .file_request(&mut shell, FileRequest::Play("/card/Music/tone.wav".into()))
            .is_err()
    );
    shell.features.files = false;
    assert!(
        media
            .file_request(&mut shell, FileRequest::List("/card".into()))
            .is_err()
    );
    shell.launch(Screen::Tools(ToolPage::Files));
    assert_eq!(shell.screen, Screen::Home);
    assert!(shell.files.pending.is_none());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn saved_track_uses_virtual_path_and_restarts_paused() {
    let (files, dir) = fixture("save", false);
    let saved = SavedMusic {
        path: Some("/card/Music/tone.wav".into()),
        position: 1.5,
        volume: Some(3),
        repeat: 2,
        shuffle: true,
        ..Default::default()
    };
    storage::save_json(&dir, "music.json", &saved, 65536).unwrap();
    let mut shell = Shell::new(Default::default());
    let mut media = Media::new(files, &dir, &mut shell).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while shell.playback.resume.is_some() && Instant::now() < deadline {
        media.tick(&mut shell);
        thread::sleep(Duration::from_millis(5));
    }
    assert!(shell.playback.resume.is_none());
    assert_eq!(shell.playback.model.tracks.len(), 1);
    assert_eq!(shell.playback.model.position, 1.5);
    assert!(!shell.playback.model.playing);
    assert_eq!(shell.playback.model.volume, 3);
    assert_eq!(shell.playback.model.repeat, Repeat::One);
    assert!(shell.playback.model.external_clock);
    shell.playback.model.tick(100.);
    assert_eq!(shell.playback.model.position, 1.5);
    media.save(&dir, &shell, true).unwrap();
    let restored: SavedMusic = storage::load_json(&dir, "music.json", 65536).unwrap();
    assert!(restored == saved);
    fs::remove_dir_all(dir).unwrap();
}
#[test]
#[ignore = "requires a real audio output; plays a silent WAV for under two seconds"]
fn real_output_pause_seek_and_eof() {
    let (files, dir) = fixture("audio", false);
    let path = if let Ok(fixture) = std::env::var("TA1278_AUDIO_FIXTURE") {
        fs::copy(fixture, dir.join("card/Music/test.mp3")).unwrap();
        "/card/Music/test.mp3"
    } else {
        "/card/Music/tone.wav"
    };
    let duration = probe(&files, &crate::platform::visual::SystemDecoder, path)
        .unwrap()
        .seconds;
    let mut music = phone_core::music::Music::default();
    music.external_clock = true;
    music.tracks = vec![("tone".into(), duration)];
    music.play(0);
    let mut audio = Audio::default();
    let path = Some(path);
    let status = audio.sync(&files, path, crate::shell::music::playback(&music));
    crate::shell::music::apply_playback(&mut music, status).unwrap();
    thread::sleep(Duration::from_millis(200));
    let status = audio.sync(&files, path, crate::shell::music::playback(&music));
    crate::shell::music::apply_playback(&mut music, status).unwrap();
    assert!(music.position > 0.05, "output clock did not advance");
    music.playing = false;
    let status = audio.sync(&files, path, crate::shell::music::playback(&music));
    crate::shell::music::apply_playback(&mut music, status).unwrap();
    thread::sleep(Duration::from_millis(60));
    let status = audio.sync(&files, path, crate::shell::music::playback(&music));
    crate::shell::music::apply_playback(&mut music, status).unwrap();
    let paused = music.position;
    thread::sleep(Duration::from_millis(150));
    let status = audio.sync(&files, path, crate::shell::music::playback(&music));
    crate::shell::music::apply_playback(&mut music, status).unwrap();
    assert!((music.position - paused).abs() < 0.04);
    music.position = duration - 0.2;
    music.seek_revision += 1;
    music.playing = true;
    let status = audio.sync(&files, path, crate::shell::music::playback(&music));
    crate::shell::music::apply_playback(&mut music, status).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while music.playing && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(30));
        let status = audio.sync(&files, path, crate::shell::music::playback(&music));
        crate::shell::music::apply_playback(&mut music, status).unwrap();
    }
    assert!(!music.playing, "EOF not delivered");
    assert_eq!(music.position, duration);
    eprintln!("AUDIO PASS: {path:?}, duration={duration}, paused_at={paused}, seek_to_eof=true");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn file_navigation_rename_delete_cancel_and_player_return() {
    use phone_core::input::{Event, Key, Phase};
    let (files, dir) = fixture("navigation", false);
    let mut shell = Shell::new(Default::default());
    let mut media = Media::new(files, &dir, &mut shell).unwrap();
    shell.playback.refresh = false;
    let (tracks, warnings) = scan(&media.files, media.decoder.as_ref());
    media.apply_scan(&mut shell, tracks, warnings);
    shell.launch(Screen::Tools(ToolPage::Files));
    let run = |p: &mut Shell, m: &mut Media| {
        if let Some(r) = p.files.pending.take() {
            m.file_request(p, r).unwrap();
        }
    };
    let key = |p: &mut Shell, key| {
        p.handle(
            Event {
                key,
                phase: Phase::Down,
            },
            0,
        )
    };
    run(&mut shell, &mut media);
    assert_eq!(shell.files.view, FileView::Storage);
    key(&mut shell, Key::Center);
    run(&mut shell, &mut media);
    key(&mut shell, Key::Center);
    run(&mut shell, &mut media);
    key(&mut shell, Key::Center);
    run(&mut shell, &mut media);
    assert_eq!(shell.files.path, "/card/Music");
    key(&mut shell, Key::SoftLeft);
    key(&mut shell, Key::Down);
    key(&mut shell, Key::Center);
    assert_eq!(shell.files.view, FileView::DeleteSelect);
    key(&mut shell, Key::SoftRight);
    assert!(dir.join("card/Music/tone.wav").exists());
    key(&mut shell, Key::Center);
    run(&mut shell, &mut media);
    assert_eq!(shell.screen, Screen::Music);
    assert!(shell.playback.model.playing);
    key(&mut shell, Key::SoftRight);
    assert_eq!(shell.screen, Screen::Tools(ToolPage::Files));
    assert_eq!(shell.files.path, "/card/Music");
    media
        .file_request(
            &mut shell,
            FileRequest::Rename("/card/Music/tone.wav".into(), "/card/Music/new.wav".into()),
        )
        .unwrap();
    assert_eq!(shell.files.entries[0].name, "new.wav");
    media
        .file_request(
            &mut shell,
            FileRequest::RemoveMany(vec!["/card/Music/new.wav".into()]),
        )
        .unwrap();
    finish_jobs(&mut media, &mut shell);
    assert!(shell.files.entries.is_empty());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_directory_navigation_preserves_history_and_focus() {
    let (files, dir) = fixture("failed-nav", false);
    let mut p = Shell::new(Default::default());
    let mut media = Media::new(files, &dir, &mut p).unwrap();
    media
        .file_request(&mut p, FileRequest::List("/".into()))
        .unwrap();
    media
        .file_request(&mut p, FileRequest::Enter("/card".into()))
        .unwrap();
    let before = (
        p.files.path.clone(),
        p.files.history.clone(),
        p.index,
        p.files.entries.clone(),
    );
    assert!(
        media
            .file_request(&mut p, FileRequest::Enter("/card/missing".into()))
            .is_err()
    );
    assert_eq!(
        (
            p.files.path.clone(),
            p.files.history.clone(),
            p.index,
            p.files.entries.clone()
        ),
        before
    );
    media
        .file_request(&mut p, FileRequest::Enter("/card/Music".into()))
        .unwrap();
    fs::rename(dir.join("card"), dir.join("removed-card")).unwrap();
    let history = p.files.history.clone();
    assert!(media.file_request(&mut p, FileRequest::Parent).is_err());
    assert_eq!(p.files.history, history);
    assert_eq!(p.files.path, "/card/Music");
    fs::rename(dir.join("removed-card"), dir.join("card")).unwrap();
    media.file_request(&mut p, FileRequest::Parent).unwrap();
    assert_eq!(p.files.path, "/card");
    assert_eq!(p.files.history.len(), 1);
    p.files.pending = Some(FileRequest::Enter("/card/Music".into()));
    p.go(Screen::Home);
    assert!(p.files.pending.is_none());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn pending_scan_saves_controls_and_keeps_resume_path_even_on_close() {
    let (files, dir) = fixture("pending-save", false);
    let original = SavedMusic {
        path: Some("/card/Music/tone.wav".into()),
        position: 1.25,
        volume: Some(6),
        ..Default::default()
    };
    storage::save_json(&dir, "music.json", &original, 65536).unwrap();
    let mut p = Shell::new(Default::default());
    let mut media = Media::new(files, &dir, &mut p).unwrap();
    p.playback.model.volume = 2;
    p.playback.model.repeat = Repeat::All;
    p.playback.model.shuffle = true;
    media.save(&dir, &p, true).unwrap();
    let state: SavedMusic = storage::load_json(&dir, "music.json", 65536).unwrap();
    assert_eq!(state.path, original.path);
    assert_eq!(state.position, 1.25);
    assert_eq!(state.volume, Some(2));
    assert_eq!(state.repeat, 1);
    assert!(state.shuffle);
    let bytes = fs::read(dir.join("music.json")).unwrap();
    fs::remove_file(dir.join("music.json.bak")).unwrap();
    fs::create_dir(dir.join("music.json.bak")).unwrap();
    p.playback.model.volume = 3;
    assert!(media.save(&dir, &p, true).is_err());
    assert_eq!(fs::read(dir.join("music.json")).unwrap(), bytes);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn stale_scan_cannot_restore_renamed_or_deleted_track() {
    let (files, dir) = fixture("stale-scan", false);
    let mut p = Shell::new(Default::default());
    let mut media = Media::new(files, &dir, &mut p).unwrap();
    let (old, warnings) = scan(&media.files, media.decoder.as_ref());
    media.apply_scan(&mut p, old.clone(), warnings);
    p.playback.refresh = false;
    p.playback.model.position = 1.25;
    p.playback.started = true;
    p.screen = Screen::Music;
    p.playback.view = crate::shell::MusicView::Repeat;
    p.index = 2;
    media.prepare_refresh(&mut p);
    p.playback.refresh = false;
    media.scanning = true;
    let (tx, rx) = mpsc::channel();
    media.results = rx;
    media
        .file_request(
            &mut p,
            FileRequest::Rename("/card/Music".into(), "/card/Songs".into()),
        )
        .unwrap();
    assert_eq!(
        p.playback.resume.as_ref().unwrap().path.as_deref(),
        Some("/card/Songs/tone.wav")
    );
    // File listing affects the shared selection; restore the open repeat picker.
    p.index = 2;
    tx.send((old, vec![])).unwrap();
    media.tick(&mut p);
    assert!(p.playback.loading);
    assert_eq!(p.index, 2);
    p.play_track(0);
    assert!(!p.playback.model.playing);
    tx.send(scan(&media.files, media.decoder.as_ref())).unwrap();
    media.tick(&mut p);
    assert!(!p.playback.loading);
    assert_eq!(media.tracks[0].path, "/card/Songs/tone.wav");
    assert_eq!(p.playback.model.position, 1.25);
    assert_eq!(p.index, 2);
    p.files.path = "/card/Songs".into();
    media
        .file_request(
            &mut p,
            FileRequest::RemoveMany(vec!["/card/Songs/tone.wav".into()]),
        )
        .unwrap();
    finish_jobs(&mut media, &mut p);
    assert!(p.playback.resume.as_ref().unwrap().path.is_none());
    media.save(&dir, &p, true).unwrap();
    let saved: SavedMusic = storage::load_json(&dir, "music.json", 65536).unwrap();
    assert!(saved.path.is_none());
    assert!(!path_contains("/card/Music", "/card/Music-old/song.wav"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn scanning_a_library_does_not_claim_a_track_was_played() {
    let (files, dir) = fixture("unplayed", false);
    let mut shell = Shell::new(Default::default());
    let mut media = Media::new(files, &dir, &mut shell).unwrap();
    let (tracks, warnings) = scan(&media.files, media.decoder.as_ref());
    media.apply_scan(&mut shell, tracks, warnings);
    media.save(&dir, &shell, true).unwrap();
    let saved: SavedMusic = storage::load_json(&dir, "music.json", 65536).unwrap();
    assert!(saved.path.is_none());
    assert!(!shell.playback.started);
    fs::remove_dir_all(dir).unwrap();
}

fn finish_jobs(media: &mut Media, shell: &mut Shell) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while shell.files.busy && Instant::now() < deadline {
        media.finish_file_jobs(shell);
        thread::sleep(Duration::from_millis(1));
    }
    assert!(!shell.files.busy);
}

#[test]
fn library_metadata_uses_injected_decoder_and_rejects_invalid_duration() {
    struct Metadata(f32);
    impl crate::platform::Decoder for Metadata {
        fn load(&self, _: &Folder, _: &str, _: bool) -> Result<crate::platform::Visual, String> {
            Err("not a visual decoder".into())
        }
        fn audio_duration(&self, _: &Folder, _: &str) -> Result<f32, String> {
            Ok(self.0)
        }
    }
    let (files, dir) = fixture("metadata-port", false);
    let track = probe(&files, &Metadata(42.), "/card/Music/tone.wav").unwrap();
    assert_eq!((track.seconds, track.title.as_str()), (42., "tone.wav"));
    assert!(probe(&files, &Metadata(f32::INFINITY), "/card/Music/tone.wav").is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn midi_library_and_file_open_share_the_player_without_audio_output() {
    use phone_core::input::{Event, Key, Phase};
    let (files, dir) = fixture("midi", false);
    let midi = include_bytes!("../../../../assets/audio/snake-food.mid");
    for name in ["lower.mid", "upper.MID", "mixed.MiDi"] {
        fs::write(dir.join("card/Music").join(name), midi).unwrap();
    }
    fs::write(dir.join("card/bad.MID"), b"not MIDI").unwrap();
    let (tracks, errors) = scan(&files, &crate::platform::visual::SystemDecoder);
    assert_eq!(tracks.len(), 4);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("bad.MID"));
    for track in tracks.iter().filter(|t| !t.path.ends_with("wav")) {
        let mut source = audio::decode(&files, &track.path).unwrap();
        assert_eq!(
            source.total_duration().unwrap().as_secs_f32(),
            track.seconds
        );
        assert!(source.by_ref().take(32000).any(|v| v.abs() > 0.001));
        source
            .try_seek(Duration::from_secs_f32(track.seconds))
            .unwrap();
        assert!(source.next().is_none());
    }
    let mut shell = Shell::new(Default::default());
    let mut media = Media::new(files, &dir, &mut shell).unwrap();
    shell.playback.refresh = false;
    media.apply_scan(&mut shell, tracks, errors);
    shell.go(Screen::Tools(ToolPage::Files));
    media
        .file_request(
            &mut shell,
            FileRequest::Play("/card/Music/upper.MID".into()),
        )
        .unwrap();
    assert_eq!(shell.screen, Screen::Music);
    assert!(shell.playback.model.playing && shell.playback.return_file);
    assert_eq!(
        shell.playback.paths[shell.playback.model.index],
        "/card/Music/upper.MID"
    );
    shell.handle(
        Event {
            key: Key::Center,
            phase: Phase::Down,
        },
        0,
    );
    assert!(!shell.playback.model.playing);
    shell.handle(
        Event {
            key: Key::SoftRight,
            phase: Phase::Down,
        },
        0,
    );
    assert_eq!(shell.screen, Screen::Tools(ToolPage::Files));
    fs::remove_dir_all(dir).unwrap();
}
