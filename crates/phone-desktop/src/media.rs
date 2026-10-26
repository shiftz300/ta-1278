//! Shared VFS service for file browsing and the music library.
mod files;
use crate::{
    audio::{self, Audio},
    phone::{
        Phone, Screen,
        files::{FileRequest, FileView},
    },
    storage,
};
use phone_core::music::Repeat;
use phone_vfs::Vfs;
use rodio::Source;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, mpsc},
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Debug)]
pub struct Track {
    pub path: String,
    pub title: String,
    pub seconds: f32,
}
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedMusic {
    queue: Vec<String>,
    volume: Option<u8>,
    repeat: u8,
    shuffle: bool,
    path: Option<String>,
    position: f32,
}
impl SavedMusic {
    fn capture(phone: &Phone, tracks: &[Track]) -> Self {
        Self {
            queue: phone
                .music
                .queue
                .iter()
                .filter_map(|i| tracks.get(*i).map(|t| t.path.clone()))
                .collect(),
            volume: Some(phone.music.volume),
            repeat: match phone.music.repeat {
                Repeat::Off => 0,
                Repeat::All => 1,
                Repeat::One => 2,
            },
            shuffle: phone.music.shuffle,
            path: phone
                .music_started
                .then(|| tracks.get(phone.music.index))
                .flatten()
                .map(|t| t.path.clone()),
            position: phone.music.position,
        }
    }
}
pub fn supported(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .is_some_and(|e| e.eq_ignore_ascii_case("mp3") || e.eq_ignore_ascii_case("wav"))
}
pub fn probe(vfs: &Vfs, path: &str) -> Result<Track, String> {
    if !supported(path) {
        return Err("暂不支持此文件类型。支持 MP3 和 WAV。".into());
    }
    let decoder = audio::decode(vfs, path)?;
    let seconds = decoder
        .total_duration()
        .ok_or("无法识别音频时长。")?
        .as_secs_f32();
    if seconds <= 0. || !seconds.is_finite() {
        return Err("音频时长无效。".into());
    }
    Ok(Track {
        path: path.into(),
        title: path.rsplit('/').next().unwrap_or(path).into(),
        seconds,
    })
}
fn scan(vfs: &Vfs) -> (Vec<Track>, Vec<String>) {
    let mut queue = vec![("/".to_owned(), 0)];
    let mut tracks = vec![];
    let mut warnings = vec![];
    let mut visited = 0;
    while let Some((dir, depth)) = queue.pop() {
        let entries = match vfs.list(&dir) {
            Ok(e) => e,
            Err(e) => {
                warnings.push(format!("{dir}: {e}"));
                continue;
            }
        };
        for e in entries {
            visited += 1;
            if visited > 10000 {
                warnings.push("Music scan stopped at 10000 entries".into());
                return (tracks, warnings);
            }
            if e.directory {
                if depth < 16 {
                    queue.push((e.path, depth + 1));
                } else {
                    warnings.push(format!("Scan depth limit: {}", e.path));
                }
            } else if supported(&e.path) {
                match probe(vfs, &e.path) {
                    Ok(t) => tracks.push(t),
                    Err(err) => warnings.push(format!("{}: {err}", e.path)),
                }
            }
        }
    }
    tracks.sort_by(|a, b| a.path.cmp(&b.path));
    (tracks, warnings)
}
struct TransferResult {
    result: Result<(), String>,
    from: String,
    to: String,
    moving: bool,
}
pub struct Media {
    unread: usize,
    transfers: mpsc::Receiver<TransferResult>,
    transfer_results: mpsc::Sender<TransferResult>,
    visual: crate::visual_media::VisualMedia,
    pub vfs: Arc<Vfs>,
    pub tracks: Vec<Track>,
    audio: Audio,
    jobs: mpsc::SyncSender<()>,
    results: mpsc::Receiver<(Vec<Track>, Vec<String>)>,
    scanning: bool,
    restore: Option<SavedMusic>,
    saved: SavedMusic,
    last_save: Instant,
}
impl Media {
    pub fn new(vfs: Vfs, dir: &Path, phone: &mut Phone) -> Result<Self, String> {
        let saved: SavedMusic = storage::load_json(dir, "music.json", 65536)?;
        if saved.volume.is_some_and(|v| v > 10)
            || saved.repeat > 2
            || !saved.position.is_finite()
            || saved.position < 0.
        {
            return Err("Invalid music settings; original retained".into());
        }
        phone.music.volume = saved.volume.unwrap_or(6);
        phone.music.repeat = [Repeat::Off, Repeat::All, Repeat::One][saved.repeat as usize];
        phone.music.shuffle = saved.shuffle;
        phone.music.tracks.clear();
        phone.music.external_clock = true;
        let vfs = Arc::new(vfs);
        let worker = vfs.clone();
        let (jobs, rx) = mpsc::sync_channel(1);
        let (tx, results) = mpsc::channel();
        thread::spawn(move || {
            while rx.recv().is_ok() {
                if tx.send(scan(&worker)).is_err() {
                    break;
                }
            }
        });
        phone.music_refresh = phone.features.music;
        phone.music_loading = phone.features.music;
        let (transfer_results, transfers) = mpsc::channel();
        Ok(Self {
            unread: phone.mailbox.unread(),
            transfers,
            transfer_results,
            visual: crate::visual_media::VisualMedia::new(vfs.clone()),
            vfs,
            tracks: vec![],
            audio: Audio::default(),
            jobs,
            results,
            scanning: false,
            restore: Some(saved.clone()),
            saved,
            last_save: Instant::now(),
        })
    }
    fn apply_scan(&mut self, phone: &mut Phone, tracks: Vec<Track>, warnings: Vec<String>) {
        self.audio.stop();
        phone.music.playing = false;
        phone.music.index = 0;
        phone.music.position = 0.;
        phone.music_started = false;
        phone.music_loading = false;
        phone.music.tracks = tracks
            .iter()
            .map(|t| (t.title.clone(), t.seconds))
            .collect();
        phone.set_music_paths(tracks.iter().map(|t| t.path.clone()).collect());
        phone.music.queue.clear();
        self.tracks = tracks;
        if let Some(saved) = self.restore.take() {
            phone.music.queue = saved
                .queue
                .iter()
                .filter_map(|p| phone.music_index(p))
                .collect();
            if let Some(i) = self
                .tracks
                .iter()
                .position(|t| Some(&t.path) == saved.path.as_ref())
            {
                phone.music.index = i;
                phone.music.position = saved.position.min(self.tracks[i].seconds);
                phone.music_started = true;
            }
        }
        if phone.screen == Screen::Music
            && phone.music.tracks.is_empty()
            && phone.music_view == crate::phone::MusicView::Player
        {
            phone.music_view = crate::phone::MusicView::Root;
        }
        // Songs and queue depend on the new library. Do not move a repeat-picker
        // selection or another app's shared focus while a background scan ends.
        if phone.screen == Screen::Music
            && matches!(
                phone.music_view,
                crate::phone::MusicView::Songs
                    | crate::phone::MusicView::Queue
                    | crate::phone::MusicView::Playlist
                    | crate::phone::MusicView::Root
            )
        {
            phone.index = phone.index.min(phone.music_items().len().saturating_sub(1));
        }
        if !warnings.is_empty() {
            eprintln!("Media scan: {} warning(s). {}", warnings.len(), warnings[0]);
            if phone.screen == Screen::Music {
                phone.notice(format!("部分文件无法读取（{} 项）。", warnings.len()));
            }
        }
    }
    pub fn tick(&mut self, phone: &mut Phone) {
        self.visual.tick(&self.vfs, phone);
        let volume = if phone.preferences.silent || !phone.features.sounds {
            0
        } else {
            phone.preferences.volume
        };
        let unread = phone.mailbox.unread();
        if unread > self.unread {
            phone.audio_preview = Some(
                if matches!(phone.preferences.ringtone.as_str(), "预设" | "默认") {
                    "Message 1".into()
                } else {
                    phone.preferences.ringtone.clone()
                },
            );
        }
        self.unread = unread;
        self.audio.effect_volume(volume);
        if volume > 0 {
            for cue in std::mem::take(&mut phone.cues) {
                if let Err(e) = self.audio.feedback(cue, volume) {
                    phone.status = format!("音效不可用：{e}");
                    break;
                }
            }
            if let Some(path) = phone.audio_preview.take()
                && let Err(e) = self.audio.preview(&self.vfs, &path, volume)
            {
                phone.notice(format!("无法播放铃声：{e}"));
            }
        } else {
            phone.cues.clear();
            phone.audio_preview = None;
        }
        if let Err(e) = self.audio.poll_effects() {
            phone.status = format!("原机音效无法播放：{e}");
        }
        let tone = phone
            .local_data
            .reminders
            .alerts
            .front()
            .map(|a| a.tone.as_str());
        if tone.is_some() {
            phone.music.playing = false;
            if phone.media_ui.playing {
                phone
                    .media_ui
                    .pending
                    .push(crate::phone::media_browser::Request::Command(
                        "pause".into(),
                    ));
                phone.media_ui.playing = false;
            }
        }
        if let Err(e) = self.audio.alert(&self.vfs, tone, volume) {
            phone.status = format!("闹钟声音不可用：{e}");
        }
        while let Ok(TransferResult {
            result,
            from,
            to,
            moving,
        }) = self.transfers.try_recv()
        {
            phone.files.busy = false;
            if let Err(e) = result {
                phone.notice(format!("复制/移动失败：{e}"));
            } else {
                phone.files.clipboard = None;
                self.prepare_refresh(phone);
                if moving {
                    self.remap_links(phone, &from, &to);
                }
            }
            if let Err(e) = self.list(phone, phone.files.path.clone()) {
                phone.notice(e);
            }
        }
        if let Some(request) = phone.files.pending.take()
            && let Err(e) = self.file_request(phone, request)
        {
            phone.notice(e);
        }
        if phone.music_refresh {
            self.prepare_refresh(phone);
        }
        while let Ok((tracks, warnings)) = self.results.try_recv() {
            self.scanning = false;
            // A mutation/refresh requested after this job began invalidates it.
            // Keep the old library paused until a fresh scan completes.
            if !phone.music_refresh {
                self.apply_scan(phone, tracks, warnings);
            }
        }
        if phone.music_refresh
            && !self.scanning
            && phone.features.music
            && self.jobs.try_send(()).is_ok()
        {
            phone.music_refresh = false;
            self.scanning = true;
        }
        if phone.features.music && !phone.music_loading {
            let path = self.tracks.get(phone.music.index).map(|t| t.path.as_str());
            if let Err(e) = self.audio.sync(&self.vfs, path, &mut phone.music) {
                phone.notice(format!("无法播放：{e}"));
            }
        } else {
            self.audio.stop();
            phone.music.playing = false;
        }
    }
    pub fn save(&mut self, dir: &Path, phone: &Phone, force: bool) -> Result<(), String> {
        if !force && self.last_save.elapsed() < Duration::from_secs(5) {
            return Ok(());
        }
        let mut state = SavedMusic::capture(phone, &self.tracks);
        if let Some(pending) = &self.restore {
            state.path = pending.path.clone();
            state.position = pending.position;
            state.queue = pending.queue.clone();
        }
        if state != self.saved {
            storage::save_json(dir, "music.json", &state, 65536)?;
            self.saved = state;
        }
        self.last_save = Instant::now();
        Ok(())
    }
}
fn path_contains(parent: &str, path: &str) -> bool {
    path == parent
        || path
            .strip_prefix(parent)
            .is_some_and(|suffix| suffix.starts_with('/'))
}
#[cfg(test)]
mod tests;
