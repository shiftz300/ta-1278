//! Shared local folder service for file browsing and the music library.
mod files;
pub(crate) mod visual;
use crate::shell::{
    Screen, Shell,
    files::{FileRequest, FileView},
};
#[cfg(test)]
use crate::storage;
use crate::storage::files::Folder;
#[cfg(test)]
use phone_core::music::Repeat;
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
use crate::shell::music::SavedMusic;
pub fn supported(path: &str) -> bool {
    path.rsplit('.').next().is_some_and(|e| {
        ["mp3", "wav", "mid", "midi"]
            .iter()
            .any(|extension| e.eq_ignore_ascii_case(extension))
    })
}
pub fn probe(
    files: &Folder,
    decoder: &dyn crate::platform::Decoder,
    path: &str,
) -> Result<Track, String> {
    if !supported(path) {
        return Err("暂不支持此文件类型。支持 MP3、WAV 和 MIDI。".into());
    }
    let seconds = decoder.audio_duration(files, path)?;
    if seconds <= 0. || !seconds.is_finite() {
        return Err("音频时长无效。".into());
    }
    Ok(Track {
        path: path.into(),
        title: path.rsplit('/').next().unwrap_or(path).into(),
        seconds,
    })
}
fn scan(files: &Folder, decoder: &dyn crate::platform::Decoder) -> (Vec<Track>, Vec<String>) {
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
                match probe(files, decoder, &e.path) {
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
    decoder: Arc<dyn crate::platform::Decoder>,
    jobs: mpsc::SyncSender<()>,
    results: mpsc::Receiver<(Vec<Track>, Vec<String>)>,
    scanning: bool,
}
impl Media {
    #[cfg(test)]
    pub fn new(files: Folder, dir: &Path, shell: &mut Shell) -> Result<Self, String> {
        let saved = crate::persistence::read(
            &mut crate::persistence::Directory::new(dir),
            crate::persistence::Record::Music,
        )?;
        shell.playback.restore(saved)?;
        Ok(Self::attach(crate::platform::Devices::native(files), shell))
    }
    pub fn attach(devices: crate::platform::Devices, shell: &mut Shell) -> Self {
        shell.playback.model.tracks.clear();
        shell.playback.model.external_clock = true;
        let files = devices.folder;
        let worker = files.clone();
        let decoder = devices.decoder.clone();
        let (jobs, rx) = mpsc::sync_channel(1);
        let (tx, results) = mpsc::channel();
        thread::spawn(move || {
            while rx.recv().is_ok() {
                if tx.send(scan(&worker, decoder.as_ref())).is_err() {
                    break;
                }
            }
        });
        shell.playback.refresh = shell.features.music;
        shell.playback.loading = shell.features.music;
        let (transfer_results, transfers) = mpsc::channel();
        Self {
            unread: shell.messaging.mailbox.unread(),
            transfers,
            transfer_results,
            visual: crate::media::visual::VisualMedia::new(files.clone(), devices.decoder.clone()),
            files,
            tracks: vec![],
            audio: devices.sound,
            decoder: devices.decoder,
            jobs,
            results,
            scanning: false,
        }
    }
    fn apply_scan(&mut self, shell: &mut Shell, tracks: Vec<Track>, warnings: Vec<String>) {
        self.audio.stop();
        shell.playback.model.playing = false;
        shell.playback.model.index = 0;
        shell.playback.model.position = 0.;
        shell.playback.started = false;
        shell.playback.loading = false;
        shell.playback.model.tracks = tracks
            .iter()
            .map(|t| (t.title.clone(), t.seconds))
            .collect();
        shell.set_music_paths(tracks.iter().map(|t| t.path.clone()).collect());
        shell.playback.model.queue.clear();
        self.tracks = tracks;
        if let Some(saved) = shell.playback.resume.take() {
            shell.playback.model.queue = saved
                .queue
                .iter()
                .filter_map(|p| shell.music_index(p))
                .collect();
            if let Some(i) = self
                .tracks
                .iter()
                .position(|t| Some(&t.path) == saved.path.as_ref())
            {
                shell.playback.model.index = i;
                shell.playback.model.position = saved.position.min(self.tracks[i].seconds);
                shell.playback.started = true;
            }
        }
        if shell.screen == Screen::Music
            && shell.playback.model.tracks.is_empty()
            && shell.playback.view == crate::shell::MusicView::Player
        {
            shell.playback.view = crate::shell::MusicView::Root;
        }
        // Songs and queue depend on the new library. Do not move a repeat-picker
        // selection or another app's shared focus while a background scan ends.
        if shell.screen == Screen::Music
            && matches!(
                shell.playback.view,
                crate::shell::MusicView::Songs
                    | crate::shell::MusicView::Queue
                    | crate::shell::MusicView::Playlist
                    | crate::shell::MusicView::Root
            )
        {
            shell.index = shell.index.min(shell.music_items().len().saturating_sub(1));
        }
        if !warnings.is_empty() {
            eprintln!("Media scan: {} warning(s). {}", warnings.len(), warnings[0]);
            if shell.screen == Screen::Music {
                shell.notice(format!("部分文件无法读取（{} 项）。", warnings.len()));
            }
        }
    }
    pub fn tick(&mut self, shell: &mut Shell) {
        let mut paths: Vec<_> = shell
            .settings
            .preferences
            .wallpaper
            .iter()
            .cloned()
            .collect();
        if matches!(shell.screen, Screen::Contacts | Screen::Organizer(_)) {
            paths.extend(
                shell
                    .contacts
                    .records
                    .iter()
                    .filter_map(|r| r.first())
                    .filter(|s| !s.is_empty())
                    .take(24)
                    .cloned(),
            );
            if let Some(path) = shell.contacts.draft.first().filter(|s| !s.is_empty()) {
                paths.push(path.clone());
            }
        }
        let update = self.visual.tick(
            &self.files,
            &mut shell.media,
            visual::Context {
                active: shell.screen == Screen::Media,
                pictures: shell.features.pictures,
                paths,
                index: shell.index,
            },
        );
        if let Some(index) = update.index {
            shell.index = index;
        }
        if update.pause_music {
            shell.playback.model.playing = false;
        }
        if update.close {
            shell.close_media();
        }
        if let Some(message) = update.notice {
            shell.notice(message);
        }
        let volume = if shell.settings.preferences.silent || !shell.features.sounds {
            0
        } else {
            shell.settings.preferences.volume
        };
        let unread = shell.messaging.mailbox.unread();
        if unread > self.unread {
            shell.audio_preview = Some(
                if matches!(
                    shell.settings.preferences.ringtone.as_str(),
                    "预设" | "默认"
                ) {
                    "Message 1".into()
                } else {
                    shell.settings.preferences.ringtone.clone()
                },
            );
        }
        self.unread = unread;
        self.audio.effect_volume(volume);
        if volume > 0 {
            for cue in std::mem::take(&mut shell.cues) {
                if let Err(e) = self.audio.feedback(cue, volume) {
                    shell.status = format!("音效不可用：{e}");
                    break;
                }
            }
            if let Some(path) = shell.audio_preview.take()
                && let Err(e) = self.audio.preview(&self.files, &path, volume)
            {
                shell.notice(format!("无法播放铃声：{e}"));
            }
        } else {
            shell.cues.clear();
            shell.audio_preview = None;
        }
        if let Err(e) = self.audio.poll_effects() {
            shell.status = format!("原机音效无法播放：{e}");
        }
        if shell.active_reminder().is_some() {
            shell.playback.model.playing = false;
            if shell.media.ui.playing {
                shell
                    .media
                    .ui
                    .pending
                    .push(crate::shell::media::Request::Command("pause".into()));
                shell.media.ui.playing = false;
            }
        }
        let tone = shell.active_reminder().map(|a| a.tone.as_str());
        if let Err(e) = self.audio.alert(&self.files, tone, volume) {
            shell.status = format!("闹钟声音不可用：{e}");
        }
        self.finish_file_jobs(shell);
        if let Some(request) = shell.files.pending.take()
            && let Err(e) = self.file_request(shell, request)
        {
            shell.notice(e);
        }
        if shell.playback.refresh {
            self.prepare_refresh(shell);
        }
        while let Ok((tracks, warnings)) = self.results.try_recv() {
            self.scanning = false;
            // A mutation/refresh requested after this job began invalidates it.
            // Keep the old library paused until a fresh scan completes.
            if !shell.playback.refresh {
                self.apply_scan(shell, tracks, warnings);
            }
        }
        if shell.playback.refresh
            && !self.scanning
            && shell.features.music
            && self.jobs.try_send(()).is_ok()
        {
            shell.playback.refresh = false;
            self.scanning = true;
        }
        if shell.features.music && !shell.playback.loading {
            let path = self
                .tracks
                .get(shell.playback.model.index)
                .map(|t| t.path.as_str());
            let status = self.audio.sync(
                &self.files,
                path,
                crate::shell::music::playback(&shell.playback.model),
            );
            if let Err(e) = crate::shell::music::apply_playback(&mut shell.playback.model, status) {
                shell.notice(format!("无法播放：{e}"));
            }
        } else {
            self.audio.stop();
            shell.playback.model.playing = false;
        }
    }
    #[cfg(test)]
    pub fn save(&mut self, dir: &Path, shell: &Shell, _: bool) -> Result<(), String> {
        crate::persistence::write(
            &mut crate::persistence::Directory::new(dir),
            crate::persistence::Record::Music,
            &shell.playback.snapshot(),
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
