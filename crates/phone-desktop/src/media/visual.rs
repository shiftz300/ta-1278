//! Presentation/controller bridge. Decoders only exchange pixels and video commands.
use crate::platform::{Decoder, Video, Visual};
use crate::shell::media::{Purpose, Request, State};
use crate::storage::files::Folder;
use std::{
    collections::HashSet,
    sync::{Arc, mpsc},
    thread,
};
pub struct Context {
    pub active: bool,
    pub pictures: bool,
    pub paths: Vec<String>,
    pub index: usize,
}
#[derive(Default)]
pub struct Update {
    pub index: Option<usize>,
    pub pause_music: bool,
    pub close: bool,
    pub notice: Option<String>,
}
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
    pub fn tick(&mut self, files: &Folder, media: &mut State, context: Context) -> Update {
        let mut update = Update::default();
        for request in std::mem::take(&mut media.ui.pending) {
            if !context.active && !matches!(request, Request::Stop) {
                continue;
            }
            match request {
                Request::List(path) => match files.list(&path) {
                    Ok(entries) => {
                        media.ui.entries = entries
                            .into_iter()
                            .filter(|e| e.directory || media.ui.purpose.accepts(&e.path))
                            .collect();
                        media.ui.path = path;
                        update.index =
                            Some(context.index.min(media.ui.entries.len().saturating_sub(1)));
                    }
                    Err(e) => update.notice = Some(format!("无法读取：{e}")),
                },
                Request::Open(path) => {
                    self.video = None;
                    media.ui.position = 0.;
                    media.ui.duration = 0.;
                    if let Some(img) = media.pictures.get(&path) {
                        media.ui.frame = Some(img.clone());
                    } else {
                        self.request(path, media.ui.purpose == Purpose::Videos);
                    }
                    update.pause_music = true;
                }
                Request::Command(line) => {
                    if let Some(v) = &mut self.video {
                        v.command(&line);
                    }
                }
                Request::Stop => self.video = None,
            }
        }
        if !context.active || !media.ui.viewing {
            self.video = None;
        }
        if !context.pictures {
            media.pictures.clear();
            self.queue.retain(|(_, video)| *video);
        }
        for path in context.paths.into_iter().filter(|_| context.pictures) {
            if !media.pictures.contains_key(&path) && self.attempted.insert(path.clone()) {
                self.request(path, false);
            }
        }
        while let Ok((path, result)) = self.results.try_recv() {
            self.busy = false;
            let active = context.active && media.ui.viewing && media.ui.selected == path;
            match result {
                Ok(Visual::Image(img)) => {
                    if active {
                        media.ui.frame = Some(img.clone());
                    }
                    if media.pictures.len() >= 32 {
                        media.pictures.clear();
                        self.attempted.clear();
                    }
                    media
                        .pictures
                        .insert(path, image::imageops::thumbnail(&img, 240, 320));
                }
                Ok(Visual::Video(video)) if active => {
                    media.ui.playing = true;
                    self.video = Some(video);
                }
                Err(e) if active => {
                    media.ui.viewing = false;
                    if media.ui.direct {
                        update.close = true;
                    }
                    update.notice = Some(e);
                }
                _ => {}
            }
        }
        if let Some(video) = &mut self.video {
            while let Some(frame) = video.next_frame() {
                media.ui.frame = Some(frame.image);
                media.ui.position = frame.position;
                media.ui.duration = frame.duration;
                media.ui.playing = frame.playing;
            }
            if video.ended() {
                self.video = None;
                media.ui.playing = false;
                media.ui.viewing = false;
                update.notice = Some("视频无法继续播放。".into());
            } else if video.elapsed() > std::time::Duration::from_secs(15)
                && media.ui.duration <= 0.
            {
                self.video = None;
                media.ui.viewing = false;
                media.ui.playing = false;
                update.notice = Some("视频解码超时。".into());
            }
        }
        if !self.busy && !self.queue.is_empty() {
            let job = self.queue.remove(0);
            self.busy = self.jobs.send(job).is_ok();
        }
        update
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_pictures_do_not_decode_saved_wallpaper() {
        let files = Arc::new(Folder::new(std::env::temp_dir(), true).unwrap());
        let mut service = VisualMedia::new(
            files.clone(),
            Arc::new(crate::platform::visual::SystemDecoder),
        );
        let mut media = State::default();
        media
            .pictures
            .insert("/photo.png".into(), image::RgbaImage::new(1, 1));
        service.tick(
            &files,
            &mut media,
            Context {
                active: false,
                pictures: false,
                paths: vec!["/photo.png".into()],
                index: 0,
            },
        );
        assert!(media.pictures.is_empty());
        assert!(!service.busy);
        assert!(service.queue.is_empty());
    }
    #[test]
    fn decode_failure_returns_close_intent_without_owning_navigation() {
        let files = Arc::new(Folder::new(std::env::temp_dir(), true).unwrap());
        let mut service = VisualMedia::new(
            files.clone(),
            Arc::new(crate::platform::visual::SystemDecoder),
        );
        let (tx, rx) = mpsc::channel();
        service.results = rx;
        let mut media = State::default();
        media.ui.viewing = true;
        media.ui.direct = true;
        media.ui.selected = "/broken.png".into();
        tx.send(("/broken.png".into(), Err("decode failed".into())))
            .unwrap();
        let update = service.tick(
            &files,
            &mut media,
            Context {
                active: true,
                pictures: true,
                paths: vec![],
                index: 7,
            },
        );
        assert!(update.close);
        assert_eq!(update.notice.as_deref(), Some("decode failed"));
        assert_eq!(update.index, None);
        assert!(!media.ui.viewing);
    }
}
