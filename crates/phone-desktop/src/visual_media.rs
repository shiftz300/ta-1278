//! Decode media outside the event loop. The native helper receives local VFS copies only.
use crate::phone::{
    Phone, Screen,
    media_browser::{Purpose, Request},
};
use phone_vfs::Vfs;
use std::{
    collections::HashSet,
    io::{Read, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
};
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn materialize(vfs: &Vfs, path: &str, max: u64) -> Result<Temp, String> {
    let mut file = vfs.open(path).map_err(|e| e.to_string())?;
    if file.len > max {
        return Err("文件过大。".into());
    }
    static ID: AtomicU64 = AtomicU64::new(0);
    let ext = path
        .rsplit('.')
        .next()
        .unwrap_or("bin")
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(8)
        .collect::<String>();
    let dest = std::env::temp_dir().join(format!(
        "ta1278-media-{}-{}.{}",
        std::process::id(),
        ID.fetch_add(1, Ordering::Relaxed),
        ext
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let temp = Temp(dest);
    let mut out = options.open(&temp.0).map_err(|e| e.to_string())?;
    let copied = std::io::copy(&mut file.reader.by_ref().take(max + 1), &mut out)
        .map_err(|e| e.to_string())?;
    if copied > max {
        return Err("文件过大。".into());
    }
    Ok(temp)
}
pub(crate) fn helper() -> Result<PathBuf, String> {
    if let Ok(exe) = std::env::current_exe() {
        let path = exe.with_file_name("ta1278-media");
        if path.is_file() {
            return Ok(path);
        }
    }
    if let Some(path) = option_env!("TA1278_MEDIA_HELPER") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err("此构建没有图像/视频解码组件。".into())
}
struct Frame {
    image: image::RgbaImage,
    position: f64,
    duration: f64,
    playing: bool,
}
struct Video {
    started: std::time::Instant,
    child: Child,
    input: ChildStdin,
    frames: mpsc::Receiver<Frame>,
    _temp: Temp,
}
impl Drop for Video {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
enum Loaded {
    Image(image::RgbaImage),
    Video(Video),
}
fn load(vfs: &Vfs, path: &str, video: bool) -> Result<Loaded, String> {
    let temp = materialize(
        vfs,
        path,
        if video {
            512 * 1024 * 1024
        } else {
            16 * 1024 * 1024
        },
    )?;
    let mut command = Command::new(helper()?);
    command
        .arg(if video { "video" } else { "image" })
        .arg(&temp.0);
    if !video {
        let result = command.output().map_err(|e| e.to_string())?;
        if !result.status.success() {
            return Err("无法解码此图片。".into());
        }
        return image::load_from_memory_with_format(&result.stdout, image::ImageFormat::Png)
            .map(|i| Loaded::Image(i.to_rgba8()))
            .map_err(|e| e.to_string());
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let input = child.stdin.take().unwrap();
    let mut output = child.stdout.take().unwrap();
    let (tx, frames) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut bytes = vec![0; 17 + 240 * 240 * 4];
        while output.read_exact(&mut bytes).is_ok() {
            let position = f64::from_ne_bytes(bytes[..8].try_into().unwrap());
            let duration = f64::from_ne_bytes(bytes[8..16].try_into().unwrap());
            if !position.is_finite() || !duration.is_finite() {
                break;
            }
            let image = image::RgbaImage::from_raw(240, 240, bytes[17..].to_vec()).unwrap();
            if let Err(mpsc::TrySendError::Disconnected(_)) = tx.try_send(Frame {
                image,
                position,
                duration,
                playing: bytes[16] != 0,
            }) {
                break;
            }
        }
    });
    Ok(Loaded::Video(Video {
        started: std::time::Instant::now(),
        child,
        input,
        frames,
        _temp: temp,
    }))
}
pub struct VisualMedia {
    jobs: mpsc::Sender<(String, bool)>,
    results: mpsc::Receiver<(String, Result<Loaded, String>)>,
    busy: bool,
    queue: Vec<(String, bool)>,
    attempted: HashSet<String>,
    video: Option<Video>,
}
impl VisualMedia {
    pub fn new(vfs: Arc<Vfs>) -> Self {
        let (jobs, rx) = mpsc::channel::<(String, bool)>();
        let (tx, results) = mpsc::channel();
        thread::spawn(move || {
            while let Ok((path, video)) = rx.recv() {
                let result = load(&vfs, &path, video);
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
    pub fn tick(&mut self, vfs: &Vfs, p: &mut Phone) {
        for request in std::mem::take(&mut p.media_ui.pending) {
            if p.screen != Screen::Media && !matches!(request, Request::Stop) {
                continue;
            }
            match request {
                Request::List(path) => match vfs.list(&path) {
                    Ok(entries) => {
                        p.media_ui.entries = entries
                            .into_iter()
                            .filter(|e| e.directory || p.media_ui.purpose.accepts(&e.path))
                            .collect();
                        p.media_ui.path = path;
                        p.index = p.index.min(p.media_ui.entries.len().saturating_sub(1));
                    }
                    Err(e) => p.notice(format!("无法读取：{e}")),
                },
                Request::Open(path) => {
                    self.video = None;
                    p.media_ui.position = 0.;
                    p.media_ui.duration = 0.;
                    if let Some(img) = p.pictures.get(&path) {
                        p.media_ui.frame = Some(img.clone());
                    } else {
                        self.request(path, p.media_ui.purpose == Purpose::Videos);
                    }
                    p.music.playing = false;
                }
                Request::Command(line) => {
                    if let Some(v) = &mut self.video {
                        let _ = writeln!(v.input, "{line}");
                    }
                }
                Request::Stop => self.video = None,
            }
        }
        if p.screen != Screen::Media || !p.media_ui.viewing {
            self.video = None;
        }
        let mut paths = vec![];
        if !p.features.pictures {
            p.pictures.clear();
            self.queue.retain(|(_, video)| *video);
        }
        if let Some(path) = &p.preferences.wallpaper {
            paths.push(path.clone());
        }
        if matches!(p.screen, Screen::Contacts | Screen::Organizer(_)) {
            paths.extend(
                p.local_data
                    .contacts
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
            if !p.pictures.contains_key(&path) && self.attempted.insert(path.clone()) {
                self.request(path, false);
            }
        }
        while let Ok((path, result)) = self.results.try_recv() {
            self.busy = false;
            let active =
                p.screen == Screen::Media && p.media_ui.viewing && p.media_ui.selected == path;
            match result {
                Ok(Loaded::Image(img)) => {
                    if active {
                        p.media_ui.frame = Some(img.clone());
                    }
                    if p.pictures.len() >= 32 {
                        p.pictures.clear();
                        self.attempted.clear();
                    }
                    p.pictures
                        .insert(path, image::imageops::thumbnail(&img, 240, 320));
                }
                Ok(Loaded::Video(video)) if active => {
                    p.media_ui.playing = true;
                    self.video = Some(video);
                }
                Err(e) if active => {
                    p.media_ui.viewing = false;
                    if p.media_ui.direct {
                        p.close_media();
                    }
                    p.notice(e);
                }
                _ => {}
            }
        }
        if let Some(video) = &mut self.video {
            while let Ok(frame) = video.frames.try_recv() {
                p.media_ui.frame = Some(frame.image);
                p.media_ui.position = frame.position;
                p.media_ui.duration = frame.duration;
                p.media_ui.playing = frame.playing;
            }
            if video.child.try_wait().is_ok_and(|s| s.is_some()) {
                self.video = None;
                p.media_ui.playing = false;
                p.media_ui.viewing = false;
                p.notice("视频无法继续播放。");
            } else if video.started.elapsed() > std::time::Duration::from_secs(15)
                && p.media_ui.duration <= 0.
            {
                self.video = None;
                p.media_ui.viewing = false;
                p.media_ui.playing = false;
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
        let vfs = Arc::new(Vfs::default());
        let mut service = VisualMedia::new(vfs.clone());
        let mut p = Phone::new(Default::default());
        p.features.pictures = false;
        p.preferences.wallpaper = Some("/card/photo.png".into());
        p.pictures
            .insert("/card/photo.png".into(), image::RgbaImage::new(1, 1));
        service.tick(&vfs, &mut p);
        assert!(p.pictures.is_empty());
        assert!(!service.busy);
        assert!(service.queue.is_empty());
    }
    fn fixture(name: &str) -> (Vfs, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("ta1278-visual-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let mut vfs = Vfs::default();
        vfs.mount(
            "card",
            "card",
            Box::new(phone_vfs::Directory::new(&root).unwrap()),
            false,
        )
        .unwrap();
        (vfs, root)
    }
    #[test]
    fn native_image_decode_is_bounded_and_temp_copy_is_removed() {
        let (vfs, root) = fixture("image");
        let mut image = image::RgbaImage::from_pixel(40, 20, image::Rgba([255, 0, 0, 255]));
        for y in 10..20 {
            for x in 0..40 {
                image.put_pixel(x, y, image::Rgba([0, 0, 255, 255]));
            }
        }
        image.save(root.join("photo.png")).unwrap();
        let temp = materialize(&vfs, "/card/photo.png", 65536).unwrap();
        let path = temp.0.clone();
        assert!(path.exists());
        drop(temp);
        assert!(!path.exists());
        let Loaded::Image(decoded) = load(&vfs, "/card/photo.png", false).unwrap() else {
            panic!("Expected image")
        };
        assert_eq!(decoded.dimensions(), (40, 20));
        assert_eq!(decoded.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(decoded.get_pixel(0, 19).0, [0, 0, 255, 255]);
        let oversized = std::fs::File::create(root.join("large.png")).unwrap();
        oversized.set_len(16 * 1024 * 1024 + 1).unwrap();
        assert!(load(&vfs, "/card/large.png", false).is_err());
        std::fs::write(root.join("bad.png"), b"not an image").unwrap();
        assert!(load(&vfs, "/card/bad.png", false).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "Requires TA1278_VIDEO_FIXTURE and native video/audio output"]
    fn native_video_frames_pause_seek_and_cleanup() {
        let (vfs, root) = fixture("video");
        std::fs::copy(
            std::env::var_os("TA1278_VIDEO_FIXTURE").expect("set TA1278_VIDEO_FIXTURE"),
            root.join("clip.mp4"),
        )
        .unwrap();
        let Loaded::Video(mut video) = load(&vfs, "/card/clip.mp4", true).unwrap() else {
            panic!("Expected video")
        };
        let temp = video._temp.0.clone();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let mut seen = None;
        while std::time::Instant::now() < deadline {
            if let Ok(frame) = video
                .frames
                .recv_timeout(std::time::Duration::from_millis(300))
                && frame.duration > 0.
                && frame.position > 0.2
                && frame.image.pixels().any(|p| p[0] > 100 || p[1] > 100)
            {
                seen = Some(frame);
                break;
            }
        }
        let first = seen.expect("No decoded native video frame");
        if let Some(path) = std::env::var_os("TA1278_MEDIA_FRAME") {
            first.image.save(path).unwrap();
        }
        eprintln!(
            "Native video: {}x{}, duration {:.2}s; first frame at {:.2}s",
            first.image.width(),
            first.image.height(),
            first.duration,
            first.position
        );
        assert!(first.playing);
        assert!(first.duration >= 5.);
        writeln!(video.input, "pause").unwrap();
        let paused = loop {
            let f = video
                .frames
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap();
            if !f.playing {
                break f;
            }
        };
        let next = video
            .frames
            .recv_timeout(std::time::Duration::from_secs(3))
            .unwrap();
        assert!((next.position - paused.position).abs() < 0.15);
        writeln!(video.input, "seek 3").unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut sought = false;
        while std::time::Instant::now() < deadline {
            if let Ok(f) = video
                .frames
                .recv_timeout(std::time::Duration::from_millis(200))
                && (f.position - 3.).abs() < 0.15
            {
                sought = true;
                break;
            }
        }
        assert!(sought);
        writeln!(video.input, "play").unwrap();
        let resumed = loop {
            let f = video
                .frames
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap();
            if f.playing {
                break f;
            }
        };
        assert!(resumed.position >= 2.85);
        let pid = video.child.id();
        drop(video);
        assert!(!temp.exists());
        assert_ne!(pid, 0);
        std::fs::remove_dir_all(root).unwrap();
    }
}
