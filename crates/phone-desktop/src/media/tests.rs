use super::*;
use crate::audio::Audio;
use crate::phone::{files::FileRequest, tools::ToolPage};
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
    let (tracks, errors) = scan(&files);
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
    let mut p = Phone::new(Default::default());
    p.features.music = false;
    p.settings.preferences.ringtone = "/card/Music/tone.wav".into();
    p.playback.playlists.push(crate::phone::Playlist {
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
    let mut phone = Phone::new(Default::default());
    phone.features.music = false;
    let mut media = Media::new(files, &dir, &mut phone).unwrap();
    media
        .file_request(&mut phone, FileRequest::List("/card".into()))
        .unwrap();
    assert_eq!(phone.files.entries[0].name, "Music");
    assert!(!phone.files.writable);
    assert!(
        media
            .file_request(&mut phone, FileRequest::Mkdir("/card/no".into()))
            .is_err()
    );
    assert!(
        media
            .file_request(&mut phone, FileRequest::Play("/card/Music/tone.wav".into()))
            .is_err()
    );
    phone.features.files = false;
    assert!(
        media
            .file_request(&mut phone, FileRequest::List("/card".into()))
            .is_err()
    );
    phone.launch(Screen::Tools(ToolPage::Files));
    assert_eq!(phone.screen, Screen::Home);
    assert!(phone.files.pending.is_none());
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
    let mut phone = Phone::new(Default::default());
    let mut media = Media::new(files, &dir, &mut phone).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while phone.playback.resume.is_some() && Instant::now() < deadline {
        media.tick(&mut phone);
        thread::sleep(Duration::from_millis(5));
    }
    assert!(phone.playback.resume.is_none());
    assert_eq!(phone.playback.model.tracks.len(), 1);
    assert_eq!(phone.playback.model.position, 1.5);
    assert!(!phone.playback.model.playing);
    assert_eq!(phone.playback.model.volume, 3);
    assert_eq!(phone.playback.model.repeat, Repeat::One);
    assert!(phone.playback.model.external_clock);
    phone.playback.model.tick(100.);
    assert_eq!(phone.playback.model.position, 1.5);
    media.save(&dir, &phone, true).unwrap();
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
    let duration = probe(&files, path).unwrap().seconds;
    let mut music = phone_core::music::Music::default();
    music.external_clock = true;
    music.tracks = vec![("tone".into(), duration)];
    music.play(0);
    let mut audio = Audio::default();
    let path = Some(path);
    audio.sync(&files, path, &mut music).unwrap();
    thread::sleep(Duration::from_millis(200));
    audio.sync(&files, path, &mut music).unwrap();
    assert!(music.position > 0.05, "output clock did not advance");
    music.playing = false;
    audio.sync(&files, path, &mut music).unwrap();
    thread::sleep(Duration::from_millis(60));
    audio.sync(&files, path, &mut music).unwrap();
    let paused = music.position;
    thread::sleep(Duration::from_millis(150));
    audio.sync(&files, path, &mut music).unwrap();
    assert!((music.position - paused).abs() < 0.04);
    music.position = duration - 0.2;
    music.seek_revision += 1;
    music.playing = true;
    audio.sync(&files, path, &mut music).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while music.playing && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(30));
        audio.sync(&files, path, &mut music).unwrap();
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
    let mut phone = Phone::new(Default::default());
    let mut media = Media::new(files, &dir, &mut phone).unwrap();
    phone.playback.refresh = false;
    let (tracks, warnings) = scan(&media.files);
    media.apply_scan(&mut phone, tracks, warnings);
    phone.launch(Screen::Tools(ToolPage::Files));
    let run = |p: &mut Phone, m: &mut Media| {
        if let Some(r) = p.files.pending.take() {
            m.file_request(p, r).unwrap();
        }
    };
    let key = |p: &mut Phone, key| {
        p.handle(
            Event {
                key,
                phase: Phase::Down,
            },
            0,
        )
    };
    run(&mut phone, &mut media);
    assert_eq!(phone.files.view, FileView::Storage);
    key(&mut phone, Key::Center);
    run(&mut phone, &mut media);
    key(&mut phone, Key::Center);
    run(&mut phone, &mut media);
    key(&mut phone, Key::Center);
    run(&mut phone, &mut media);
    assert_eq!(phone.files.path, "/card/Music");
    key(&mut phone, Key::SoftLeft);
    key(&mut phone, Key::Down);
    key(&mut phone, Key::Center);
    assert_eq!(phone.files.view, FileView::DeleteSelect);
    key(&mut phone, Key::SoftRight);
    assert!(dir.join("card/Music/tone.wav").exists());
    key(&mut phone, Key::Center);
    run(&mut phone, &mut media);
    assert_eq!(phone.screen, Screen::Music);
    assert!(phone.playback.model.playing);
    key(&mut phone, Key::SoftRight);
    assert_eq!(phone.screen, Screen::Tools(ToolPage::Files));
    assert_eq!(phone.files.path, "/card/Music");
    media
        .file_request(
            &mut phone,
            FileRequest::Rename("/card/Music/tone.wav".into(), "/card/Music/new.wav".into()),
        )
        .unwrap();
    assert_eq!(phone.files.entries[0].name, "new.wav");
    media
        .file_request(
            &mut phone,
            FileRequest::RemoveMany(vec!["/card/Music/new.wav".into()]),
        )
        .unwrap();
    finish_jobs(&mut media, &mut phone);
    assert!(phone.files.entries.is_empty());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_directory_navigation_preserves_history_and_focus() {
    let (files, dir) = fixture("failed-nav", false);
    let mut p = Phone::new(Default::default());
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
    let mut p = Phone::new(Default::default());
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
    let mut p = Phone::new(Default::default());
    let mut media = Media::new(files, &dir, &mut p).unwrap();
    let (old, warnings) = scan(&media.files);
    media.apply_scan(&mut p, old.clone(), warnings);
    p.playback.refresh = false;
    p.playback.model.position = 1.25;
    p.playback.started = true;
    p.screen = Screen::Music;
    p.playback.view = crate::phone::MusicView::Repeat;
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
    tx.send(scan(&media.files)).unwrap();
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
    let mut phone = Phone::new(Default::default());
    let mut media = Media::new(files, &dir, &mut phone).unwrap();
    let (tracks, warnings) = scan(&media.files);
    media.apply_scan(&mut phone, tracks, warnings);
    media.save(&dir, &phone, true).unwrap();
    let saved: SavedMusic = storage::load_json(&dir, "music.json", 65536).unwrap();
    assert!(saved.path.is_none());
    assert!(!phone.playback.started);
    fs::remove_dir_all(dir).unwrap();
}

fn finish_jobs(media: &mut Media, phone: &mut Phone) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while phone.files.busy && Instant::now() < deadline {
        media.finish_file_jobs(phone);
        thread::sleep(Duration::from_millis(1));
    }
    assert!(!phone.files.busy);
}
