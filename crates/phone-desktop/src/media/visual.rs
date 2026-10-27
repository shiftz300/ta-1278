//! Presentation/controller bridge. Decoders only exchange pixels and video commands.
use crate::phone::{
    Phone, Screen,
    media::{Purpose, Request},
};
pub(crate) use crate::platform::visual::helper;
use crate::platform::{Decoder, Video, Visual};
use crate::storage::files::Folder;
use std::{
    collections::HashSet,
    sync::{Arc, mpsc},
    thread,
};
pub struct VisualMedia {
    jobs: mpsc::Sender<(String, bool)>,
    results: mpsc::Receiver<(String, Result<Visual, String>)>,
    busy: bool,
    queue: Vec<(String, bool)>,
    attempted: HashSet<String>,
    video: Option<Box<dyn Video>>,
}
impl VisualMedia {
    pub fn new(files: Arc<Folder>, decoder: Arc<dyn Decoder>) -> Self {
        let (jobs, rx) = mpsc::channel::<(String, bool)>();
        let (tx, results) = mpsc::channel();
        thread::spawn(move || {
            while let Ok((path, video)) = rx.recv() {
                let result = decoder.load(&files, &path, video);
                if tx.send((path, result)).is_err() {
                    break;
                }
            }
        });
        Self {
            jobs,
            results,
            busy: false,
            queue: vec![],
            attempted: HashSet::new(),
            video: None,
        }
    }
    fn request(&mut self, path: String, video: bool) {
        if !self.queue.iter().any(|(p, _)| p == &path) {
            self.queue.push((path, video));
        }
    }
    pub fn tick(&mut self, files: &Folder, p: &mut Phone) {
        for request in std::mem::take(&mut p.media.ui.pending) {
            if p.screen != Screen::Media && !matches!(request, Request::Stop) {
                continue;
            }
            match request {
                Request::List(path) => match files.list(&path) {
                    Ok(entries) => {
                        p.media.ui.entries = entries
                            .into_iter()
                            .filter(|e| e.directory || p.media.ui.purpose.accepts(&e.path))
                            .collect();
                        p.media.ui.path = path;
                        p.index = p.index.min(p.media.ui.entries.len().saturating_sub(1));
                    }
                    Err(e) => p.notice(format!("无法读取：{e}")),
                },
                Request::Open(path) => {
                    self.video = None;
                    p.media.ui.position = 0.;
                    p.media.ui.duration = 0.;
                    if let Some(img) = p.media.pictures.get(&path) {
                        p.media.ui.frame = Some(img.clone());
                    } else {
                        self.request(path, p.media.ui.purpose == Purpose::Videos);
                    }
                    p.playback.model.playing = false;
                }
                Request::Command(line) => {
                    if let Some(v) = &mut self.video {
                        v.command(&line);
                    }
                }
                Request::Stop => self.video = None,
            }
        }
        if p.screen != Screen::Media || !p.media.ui.viewing {
            self.video = None;
        }
        let mut paths = vec![];
        if !p.features.pictures {
            p.media.pictures.clear();
            self.queue.retain(|(_, video)| *video);
        }
        if let Some(path) = &p.settings.preferences.wallpaper {
            paths.push(path.clone());
        }
        if matches!(p.screen, Screen::Contacts | Screen::Organizer(_)) {
            paths.extend(
                p.contacts
                    .records
                    .iter()
                    .filter_map(|r| r.first())
                    .filter(|s| !s.is_empty())
                    .take(24)
                    .cloned(),
            );
            if let Some(path) = p.organizer.draft.first().filter(|s| !s.is_empty()) {
                paths.push(path.clone());
            }
        }
        for path in paths.into_iter().filter(|_| p.features.pictures) {
            if !p.media.pictures.contains_key(&path) && self.attempted.insert(path.clone()) {
                self.request(path, false);
            }
        }
        while let Ok((path, result)) = self.results.try_recv() {
            self.busy = false;
            let active =
                p.screen == Screen::Media && p.media.ui.viewing && p.media.ui.selected == path;
            match result {
                Ok(Visual::Image(img)) => {
                    if active {
                        p.media.ui.frame = Some(img.clone());
                    }
                    if p.media.pictures.len() >= 32 {
                        p.media.pictures.clear();
                        self.attempted.clear();
                    }
                    p.media
                        .pictures
                        .insert(path, image::imageops::thumbnail(&img, 240, 320));
                }
                Ok(Visual::Video(video)) if active => {
                    p.media.ui.playing = true;
                    self.video = Some(video);
                }
                Err(e) if active => {
                    p.media.ui.viewing = false;
                    if p.media.ui.direct {
                        p.close_media();
                    }
                    p.notice(e);
                }
                _ => {}
            }
        }
        if let Some(video) = &mut self.video {
            while let Some(frame) = video.next_frame() {
                p.media.ui.frame = Some(frame.image);
                p.media.ui.position = frame.position;
                p.media.ui.duration = frame.duration;
                p.media.ui.playing = frame.playing;
            }
            if video.ended() {
                self.video = None;
                p.media.ui.playing = false;
                p.media.ui.viewing = false;
                p.notice("视频无法继续播放。");
            } else if video.elapsed() > std::time::Duration::from_secs(15)
                && p.media.ui.duration <= 0.
            {
                self.video = None;
                p.media.ui.viewing = false;
                p.media.ui.playing = false;
                p.notice("视频解码超时。");
            }
        }
        if !self.busy && !self.queue.is_empty() {
            let job = self.queue.remove(0);
            self.busy = self.jobs.send(job).is_ok();
        }
    }
}
#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    #[test]
    fn disabled_pictures_do_not_decode_saved_wallpaper() {
        let files = Arc::new(Folder::new(std::env::temp_dir(), true).unwrap());
        let mut service = VisualMedia::new(
            files.clone(),
            Arc::new(crate::platform::visual::SystemDecoder),
        );
        let mut p = Phone::new(Default::default());
        p.features.pictures = false;
        p.settings.preferences.wallpaper = Some("/photo.png".into());
        p.media
            .pictures
            .insert("/photo.png".into(), image::RgbaImage::new(1, 1));
        service.tick(&files, &mut p);
        assert!(p.media.pictures.is_empty());
        assert!(!service.busy);
        assert!(service.queue.is_empty());
    }
}
