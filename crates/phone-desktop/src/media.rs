//! Shared local folder service for file browsing and the music library.
mod files;
pub(crate) mod visual;
#[cfg(test)]
use crate::storage;
use crate::storage::files::Folder;
use crate::{
    audio,
    phone::{
        Phone, Screen,
        files::{FileRequest, FileView},
    },
};
#[cfg(test)]
use phone_core::music::Repeat;
use rodio::Source;
#[cfg(test)]
use std::{
    path::Path,
    time::{Duration, Instant},
};
use std::{
    sync::{Arc, mpsc},
    thread,
};
#[derive(Clone, Debug)]
pub struct Track {
    pub path: String,
    pub title: String,
    pub seconds: f32,
}
#[cfg(test)]
use crate::phone::music::SavedMusic;
pub fn supported(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .is_some_and(|e| e.eq_ignore_ascii_case("mp3") || e.eq_ignore_ascii_case("wav"))
}
pub fn probe(files: &Folder, path: &str) -> Result<Track, String> {
    if !supported(path) {
        return Err("暂不支持此文件类型。支持 MP3 和 WAV。".into());
    }
    let decoder = audio::decode(files, path)?;
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
fn scan(files: &Folder) -> (Vec<Track>, Vec<String>) {
    let mut queue = vec![("/".to_owned(), 0)];
    let mut tracks = vec![];
    let mut warnings = vec![];
    let mut visited = 0;
    while let Some((dir, depth)) = queue.pop() {
        let entries = match files.list(&dir) {
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
                match probe(files, &e.path) {
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
enum FileResult {
    Transfer(TransferResult),
    Remove {
        removed: Vec<String>,
        failed: Vec<(String, String)>,
    },
}
pub struct Media {
    unread: usize,
    transfers: mpsc::Receiver<FileResult>,
    transfer_results: mpsc::Sender<FileResult>,
    visual: crate::media::visual::VisualMedia,
    pub files: Arc<Folder>,
    pub tracks: Vec<Track>,
    audio: Box<dyn crate::platform::Sound>,
    jobs: mpsc::SyncSender<()>,
    results: mpsc::Receiver<(Vec<Track>, Vec<String>)>,
    scanning: bool,
}
impl Media {
    #[cfg(test)]
    pub fn new(files: Folder, dir: &Path, phone: &mut Phone) -> Result<Self, String> {
        let saved = crate::persistence::read(
            &mut crate::persistence::Directory::new(dir),
            crate::persistence::Record::Music,
        )?;
        phone.playback.restore(saved)?;
        Ok(Self::attach(crate::platform::Devices::native(files), phone))
    }
    pub fn attach(devices: crate::platform::Devices, phone: &mut Phone) -> Self {
        phone.playback.model.tracks.clear();
        phone.playback.model.external_clock = true;
        let files = devices.folder;
        let worker = files.clone();
        let (jobs, rx) = mpsc::sync_channel(1);
        let (tx, results) = mpsc::channel();
        thread::spawn(move || {
            while rx.recv().is_ok() {
                if tx.send(scan(&worker)).is_err() {
                    break;
                }
            }
        });
        phone.playback.refresh = phone.features.music;
        phone.playback.loading = phone.features.music;
        let (transfer_results, transfers) = mpsc::channel();
        Self {
            unread: phone.messaging.mailbox.unread(),
            transfers,
            transfer_results,
            visual: crate::media::visual::VisualMedia::new(files.clone(), devices.decoder),
            files,
            tracks: vec![],
            audio: devices.sound,
            jobs,
            results,
            scanning: false,
        }
    }
    fn apply_scan(&mut self, phone: &mut Phone, tracks: Vec<Track>, warnings: Vec<String>) {
        self.audio.stop();
        phone.playback.model.playing = false;
        phone.playback.model.index = 0;
        phone.playback.model.position = 0.;
        phone.playback.started = false;
        phone.playback.loading = false;
        phone.playback.model.tracks = tracks
            .iter()
            .map(|t| (t.title.clone(), t.seconds))
            .collect();
        phone.set_music_paths(tracks.iter().map(|t| t.path.clone()).collect());
        phone.playback.model.queue.clear();
        self.tracks = tracks;
        if let Some(saved) = phone.playback.resume.take() {
            phone.playback.model.queue = saved
                .queue
                .iter()
                .filter_map(|p| phone.music_index(p))
                .collect();
            if let Some(i) = self
                .tracks
                .iter()
                .position(|t| Some(&t.path) == saved.path.as_ref())
            {
                phone.playback.model.index = i;
                phone.playback.model.position = saved.position.min(self.tracks[i].seconds);
                phone.playback.started = true;
            }
        }
        if phone.screen == Screen::Music
            && phone.playback.model.tracks.is_empty()
            && phone.playback.view == crate::phone::MusicView::Player
        {
            phone.playback.view = crate::phone::MusicView::Root;
        }
        // Songs and queue depend on the new library. Do not move a repeat-picker
        // selection or another app's shared focus while a background scan ends.
        if phone.screen == Screen::Music
            && matches!(
                phone.playback.view,
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
        self.visual.tick(&self.files, phone);
        let volume = if phone.settings.preferences.silent || !phone.features.sounds {
            0
        } else {
            phone.settings.preferences.volume
        };
        let unread = phone.messaging.mailbox.unread();
        if unread > self.unread {
            phone.audio_preview = Some(
                if matches!(
                    phone.settings.preferences.ringtone.as_str(),
                    "预设" | "默认"
                ) {
                    "Message 1".into()
                } else {
                    phone.settings.preferences.ringtone.clone()
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
                && let Err(e) = self.audio.preview(&self.files, &path, volume)
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
        if phone.active_reminder().is_some() {
            phone.playback.model.playing = false;
            if phone.media.ui.playing {
                phone
                    .media
                    .ui
                    .pending
                    .push(crate::phone::media::Request::Command("pause".into()));
                phone.media.ui.playing = false;
            }
        }
        let tone = phone.active_reminder().map(|a| a.tone.as_str());
        if let Err(e) = self.audio.alert(&self.files, tone, volume) {
            phone.status = format!("闹钟声音不可用：{e}");
        }
        self.finish_file_jobs(phone);
        if let Some(request) = phone.files.pending.take()
            && let Err(e) = self.file_request(phone, request)
        {
            phone.notice(e);
        }
        if phone.playback.refresh {
            self.prepare_refresh(phone);
        }
        while let Ok((tracks, warnings)) = self.results.try_recv() {
            self.scanning = false;
            // A mutation/refresh requested after this job began invalidates it.
            // Keep the old library paused until a fresh scan completes.
            if !phone.playback.refresh {
                self.apply_scan(phone, tracks, warnings);
            }
        }
        if phone.playback.refresh
            && !self.scanning
            && phone.features.music
            && self.jobs.try_send(()).is_ok()
        {
            phone.playback.refresh = false;
            self.scanning = true;
        }
        if phone.features.music && !phone.playback.loading {
            let path = self
                .tracks
                .get(phone.playback.model.index)
                .map(|t| t.path.as_str());
            if let Err(e) = self
                .audio
                .sync(&self.files, path, &mut phone.playback.model)
            {
                phone.notice(format!("无法播放：{e}"));
            }
        } else {
            self.audio.stop();
            phone.playback.model.playing = false;
        }
    }
    #[cfg(test)]
    pub fn save(&mut self, dir: &Path, phone: &Phone, _: bool) -> Result<(), String> {
        crate::persistence::write(
            &mut crate::persistence::Directory::new(dir),
            crate::persistence::Record::Music,
            &phone.playback.snapshot(),
        )
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
