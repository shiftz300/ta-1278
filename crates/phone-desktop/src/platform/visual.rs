//! System decoding implementation and its app-independent pixel/session ports.
use crate::storage::files::Folder;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
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
fn materialize(files: &Folder, path: &str, max: u64) -> Result<Temp, String> {
    let mut file = files.open(path).map_err(|e| e.to_string())?;
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
    let copied = std::io::copy(&mut Read::by_ref(&mut file.reader).take(max + 1), &mut out)
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
pub struct VideoFrame {
    pub image: image::RgbaImage,
    pub position: f64,
    pub duration: f64,
    pub playing: bool,
}
struct NativeVideo {
    started: std::time::Instant,
    child: Child,
    input: ChildStdin,
    frames: mpsc::Receiver<VideoFrame>,
    _temp: Temp,
}
impl Drop for NativeVideo {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
pub enum Visual {
    Image(image::RgbaImage),
    Video(Box<dyn Video>),
}
fn load(files: &Folder, path: &str, video: bool) -> Result<NativeVisual, String> {
    let temp = materialize(
        files,
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
            .map(|i| NativeVisual::Image(i.to_rgba8()))
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
            if let Err(mpsc::TrySendError::Disconnected(_)) = tx.try_send(VideoFrame {
                image,
                position,
                duration,
                playing: bytes[16] != 0,
            }) {
                break;
            }
        }
    });
    Ok(NativeVisual::Video(NativeVideo {
        started: std::time::Instant::now(),
        child,
        input,
        frames,
        _temp: temp,
    }))
}

/// Decoder results contain pixels or a bounded live video session, never app state.
pub trait Decoder: Send + Sync {
    fn load(&self, files: &Folder, path: &str, video: bool) -> Result<Visual, String>;
    fn audio_duration(&self, _files: &Folder, _path: &str) -> Result<f32, String> {
        Err("此宿主未提供音频解码。".into())
    }
}
pub trait Video: Send {
    fn command(&mut self, command: &str);
    fn next_frame(&mut self) -> Option<VideoFrame>;
    fn ended(&mut self) -> bool;
    fn elapsed(&self) -> std::time::Duration;
}
impl Video for NativeVideo {
    fn command(&mut self, command: &str) {
        let _ = writeln!(self.input, "{command}");
    }
    fn next_frame(&mut self) -> Option<VideoFrame> {
        self.frames.try_recv().ok()
    }
    fn ended(&mut self) -> bool {
        self.child.try_wait().is_ok_and(|s| s.is_some())
    }
    fn elapsed(&self) -> std::time::Duration {
        self.started.elapsed()
    }
}
pub(crate) struct SystemDecoder;
impl Decoder for SystemDecoder {
    fn audio_duration(&self, files: &Folder, path: &str) -> Result<f32, String> {
        if crate::audio::midi::player::is_file(path) {
            return crate::audio::midi::player::duration(files, path).map(|d| d.as_secs_f32());
        }
        crate::audio::decode(files, path)?
            .total_duration()
            .map(|duration| duration.as_secs_f32())
            .ok_or("无法识别音频时长。".into())
    }

    fn load(&self, files: &Folder, path: &str, video: bool) -> Result<Visual, String> {
        Ok(match load(files, path, video)? {
            NativeVisual::Image(image) => Visual::Image(image),
            NativeVisual::Video(video) => Visual::Video(Box::new(video)),
        })
    }
}

enum NativeVisual {
    Image(image::RgbaImage),
    Video(NativeVideo),
}
#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    fn fixture(name: &str) -> (Folder, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("ta1278-visual-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let files = Folder::new(&root, false).unwrap();
        (files, root)
    }
    #[cfg(feature = "desktop")]
    #[test]
    fn native_image_decode_is_bounded_and_temp_copy_is_removed() {
        let (files, root) = fixture("image");
        let mut image = image::RgbaImage::from_pixel(40, 20, image::Rgba([255, 0, 0, 255]));
        for y in 10..20 {
            for x in 0..40 {
                image.put_pixel(x, y, image::Rgba([0, 0, 255, 255]));
            }
        }
        image.save(root.join("photo.png")).unwrap();
        let temp = materialize(&files, "/photo.png", 65536).unwrap();
        let path = temp.0.clone();
        assert!(path.exists());
        drop(temp);
        assert!(!path.exists());
        let NativeVisual::Image(decoded) = load(&files, "/photo.png", false).unwrap() else {
            panic!("Expected image")
        };
        assert_eq!(decoded.dimensions(), (40, 20));
        assert_eq!(decoded.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(decoded.get_pixel(0, 19).0, [0, 0, 255, 255]);
        let oversized = std::fs::File::create(root.join("large.png")).unwrap();
        oversized.set_len(16 * 1024 * 1024 + 1).unwrap();
        assert!(load(&files, "/large.png", false).is_err());
        std::fs::write(root.join("bad.png"), b"not an image").unwrap();
        assert!(load(&files, "/bad.png", false).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "Requires TA1278_VIDEO_FIXTURE and native video/audio output"]
    fn native_video_frames_pause_seek_and_cleanup() {
        let (files, root) = fixture("video");
        std::fs::copy(
            std::env::var_os("TA1278_VIDEO_FIXTURE").expect("set TA1278_VIDEO_FIXTURE"),
            root.join("clip.mp4"),
        )
        .unwrap();
        let NativeVisual::Video(mut video) = load(&files, "/clip.mp4", true).unwrap() else {
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
