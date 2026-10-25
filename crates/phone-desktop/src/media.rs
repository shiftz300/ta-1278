//! Shared VFS service for file browsing and the music library.
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
    volume: Option<u8>,
    repeat: u8,
    shuffle: bool,
    path: Option<String>,
    position: f32,
}
impl SavedMusic {
    fn capture(phone: &Phone, tracks: &[Track]) -> Self {
        Self {
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
pub struct Media {
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
        Ok(Self {
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
    fn list(&self, phone: &mut Phone, path: String) -> Result<(), String> {
        let entries = self.vfs.list(&path).map_err(|e| e.to_string())?;
        phone.files.writable = self.vfs.writable(&path);
        phone.files.path = path;
        phone.files.entries = entries;
        phone.files.view = FileView::Browse;
        phone.index = phone.index.min(phone.files.entries.len().saturating_sub(1));
        Ok(())
    }
    fn file_request(&mut self, phone: &mut Phone, request: FileRequest) -> Result<(), String> {
        if !phone.features.files {
            return Err("文件功能已停用。".into());
        }
        match request {
            FileRequest::List(path) => self.list(phone, path),
            FileRequest::Enter(path) => {
                let previous = (phone.files.path.clone(), phone.index);
                self.list(phone, path)?;
                phone.files.history.push(previous);
                phone.index = 0;
                Ok(())
            }
            FileRequest::Parent => {
                if let Some((path, index)) = phone.files.history.last().cloned() {
                    self.list(phone, path)?;
                    phone.files.history.pop();
                    phone.index = index.min(phone.files.entries.len().saturating_sub(1));
                }
                Ok(())
            }
            FileRequest::Play(path) => {
                if self.scanning || phone.music_loading {
                    return Err("正在刷新曲库，请稍候。".into());
                }
                if !phone.features.music {
                    return Err("音乐功能已停用。".into());
                }
                let track = probe(&self.vfs, &path)?;
                let index = if let Some(i) = self.tracks.iter().position(|t| t.path == path) {
                    i
                } else {
                    phone
                        .music
                        .tracks
                        .push((track.title.clone(), track.seconds));
                    self.tracks.push(track);
                    self.tracks.len() - 1
                };
                self.restore = None;
                phone.files.selected = phone.index;
                phone.go(Screen::Music);
                phone.play_track(index);
                phone.music_return_file = true;
                Ok(())
            }
            FileRequest::Mkdir(path) => {
                self.vfs.mkdir(&path).map_err(|e| e.to_string())?;
                self.list(phone, phone.files.path.clone())
            }
            FileRequest::Remove(path) => {
                self.vfs.remove(&path).map_err(|e| e.to_string())?;
                self.prepare_refresh(phone);
                if let Some(saved) = &mut self.restore
                    && saved.path.as_ref().is_some_and(|p| path_contains(&path, p))
                {
                    saved.path = None;
                    saved.position = 0.;
                }
                self.list(phone, phone.files.path.clone())
            }
            FileRequest::Rename(from, to) => {
                self.vfs.rename(&from, &to).map_err(|e| e.to_string())?;
                self.prepare_refresh(phone);
                if let Some(saved) = &mut self.restore
                    && let Some(path) = &mut saved.path
                    && path_contains(&from, path)
                {
                    *path = format!("{to}{}", &path[from.len()..]);
                }
                self.list(phone, phone.files.path.clone())
            }
        }
    }
    fn prepare_refresh(&mut self, phone: &mut Phone) {
        if self.restore.is_none() {
            self.restore = Some(SavedMusic::capture(phone, &self.tracks));
        }
        self.audio.stop();
        phone.music.playing = false;
        phone.music_loading = phone.features.music;
        phone.music_refresh = phone.features.music;
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
        self.tracks = tracks;
        if let Some(saved) = self.restore.take()
            && let Some(i) = self
                .tracks
                .iter()
                .position(|t| Some(&t.path) == saved.path.as_ref())
        {
            phone.music.index = i;
            phone.music.position = saved.position.min(self.tracks[i].seconds);
            phone.music_started = true;
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
