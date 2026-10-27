//! Opt-in host acceptance: real files, localhost HTTP, mock messaging and CoreAudio.
use crate::{
    Runtime, Time,
    config::RuntimeConfig,
    input::Key,
    messaging::*,
    phone::{BrowserView, MusicView, Screen, conversation::Popup},
    platform::Native,
    storage,
};
use phone_core::browser::Element;
use std::{
    cell::RefCell,
    collections::VecDeque,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const WALL: u64 = 1_793_000_000_000;
#[derive(Default)]
struct Traffic {
    events: VecDeque<Envelope>,
    commands: Vec<Command>,
    acks: Vec<u64>,
}
struct Loopback(Rc<RefCell<Traffic>>);
impl Backend for Loopback {
    fn id(&self) -> &str {
        "acceptance"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            send_messages: true,
            receive_messages: true,
            alerts: false,
        }
    }
    fn submit(&mut self, command: Command) -> Result<(), PortError> {
        let mut t = self.0.borrow_mut();
        if t.commands.contains(&command) {
            return Ok(());
        }
        let Command::Send(message) = &command else {
            panic!("Call transport is outside acceptance");
        };
        t.events.push_back(Envelope {
            sequence: 1,
            event: Event::MessageStatus {
                request_id: message.request_id,
                state: SubmissionState::Sent,
            },
        });
        t.events.push_back(Envelope {
            sequence: 2,
            event: Event::MessageReceived {
                from: message.to.clone(),
                body: "Local reply".into(),
                time_ms: WALL + 60_000,
            },
        });
        t.commands.push(command);
        Ok(())
    }
    fn poll(&mut self) -> Result<Option<Envelope>, PortError> {
        Ok(self.0.borrow().events.front().cloned())
    }
    fn acknowledge(&mut self, sequence: u64) -> Result<(), PortError> {
        let mut t = self.0.borrow_mut();
        assert_eq!(t.events.front().unwrap().sequence, sequence);
        t.events.pop_front();
        t.acks.push(sequence);
        Ok(())
    }
}
struct Http {
    base: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Http {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let requests = Arc::new(Mutex::new(vec![]));
        let log = requests.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let done = stop.clone();
        let worker = thread::spawn(move || {
            while !done.load(Ordering::Relaxed) {
                let Ok((mut socket, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                };
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut bytes = [0; 8192];
                let n = socket.read(&mut bytes).unwrap();
                let request = String::from_utf8_lossy(&bytes[..n]);
                let path = request.split_whitespace().nth(1).unwrap().to_owned();
                log.lock().unwrap().push(path.clone());
                let body = if path == "/" {
                    "<a href='/next'>Next page</a>"
                } else if path == "/next" {
                    "<form action='/submit'><input name=q maxlength=32><input type=checkbox name=check value=yes><select name=color><option value=red>Red</option><option value=blue>Blue</option></select><button>Submit</button></form>"
                } else {
                    "<p>Submission received locally</p>"
                };
                write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
            }
        });
        Self {
            base,
            requests,
            stop,
            worker: Some(worker),
        }
    }
}
impl Drop for Http {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.worker.take().unwrap().join().unwrap();
    }
}
fn tap(r: &mut Runtime, key: Key) {
    r.press(42, key);
    r.release(42);
}
fn frame(r: &mut Runtime, root: &Path, name: &str) {
    r.render();
    r.canvas.save(&root.join(format!("{name}.png"))).unwrap();
}
fn save(r: &mut Runtime, root: &Path) {
    r.save(|bytes| fs::write(root.join("runtime.json"), bytes).map_err(|e| e.to_string()))
        .unwrap();
}
fn pump(r: &mut Runtime, native: &mut Native, clock: Instant, until: impl Fn(&Runtime) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let ms = clock.elapsed().as_millis() as u64;
        r.advance(Time {
            elapsed_ms: ms,
            unix_ms: WALL + ms,
        })
        .unwrap();
        native.poll(r).unwrap();
        assert!(r.status().is_empty(), "{}", r.status());
        if until(r) {
            break;
        }
        assert!(Instant::now() < deadline, "acceptance step timed out");
        thread::sleep(Duration::from_millis(15));
    }
}
fn focus(r: &mut Runtime, predicate: impl Fn(&Element) -> bool) {
    r.phone.browser.focus = r
        .phone
        .browser
        .page
        .elements
        .iter()
        .position(predicate)
        .unwrap();
}

#[test]
#[ignore = "Requires real audio output, localhost HTTP and TA1278_ACCEPTANCE_MEDIA with quiet WAV/MP3 files"]
fn complete_use_and_restart() {
    let media = PathBuf::from(
        std::env::var_os("TA1278_ACCEPTANCE_MEDIA").expect("Set an isolated test media folder"),
    );
    let root = std::env::var_os("TA1278_ACCEPTANCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("ta1278-accept-{}", std::process::id()))
        });
    assert!(!root.exists());
    fs::create_dir(&root).unwrap();
    eprintln!("Acceptance evidence: {}", root.display());
    let folder = storage::setup::select(&root, Some(media.clone()), false)
        .unwrap()
        .unwrap();
    assert!(
        storage::setup::select(&root, None, false)
            .unwrap()
            .is_some()
    );
    let test_dir = format!("/Acceptance-{}", std::process::id());
    folder.mkdir(&test_dir).unwrap();
    storage::save_json(&root, "music.json", &serde_json::json!({"volume":1}), 65536).unwrap();
    let traffic = Rc::new(RefCell::new(Traffic::default()));
    let mut config = RuntimeConfig::default();
    config.capabilities.sounds = false;
    let mut r = Runtime::with_backend(
        config.clone(),
        Time {
            elapsed_ms: 0,
            unix_ms: WALL,
        },
        Box::new(Loopback(traffic.clone())),
    )
    .unwrap();
    r.phone.preferences.greeting = "Acceptance profile".into();
    r.phone.preferences.accent = 4;
    r.phone.accent = 4;
    let mut native = Native::from_folder(&mut r, Some(folder), &root).unwrap();
    let clock = Instant::now();
    pump(&mut r, &mut native, clock, |r| !r.phone.music_loading);
    assert!(r.phone.music.tracks.iter().any(|t| t.0.ends_with(".mp3")));
    assert!(r.phone.music.tracks.iter().any(|t| t.0.ends_with(".wav")));
    r.poll_services().unwrap();
    r.phone.launch(Screen::Messages);
    tap(&mut r, Key::Center);
    r.text("12345");
    r.tab();
    r.text("Local messagX");
    tap(&mut r, Key::Delete);
    r.text("e");
    assert_eq!(r.phone.compose.body.text(), "Local message");
    frame(&mut r, &root, "message-edited");
    save(&mut r, &root);
    drop(native);
    let bytes = fs::read(root.join("runtime.json")).unwrap();
    r = Runtime::restore(
        config.clone(),
        r.time(),
        &bytes,
        Box::new(Loopback(traffic.clone())),
    )
    .unwrap();
    assert_eq!(r.mailbox().drafts[0].body, "Local message");
    let folder = storage::setup::select(&root, None, false).unwrap().unwrap();
    native = Native::from_folder(&mut r, Some(folder), &root).unwrap();
    pump(&mut r, &mut native, clock, |r| !r.phone.music_loading);
    r.phone.open_thread("12345".into());
    assert_eq!(r.phone.compose.body.text(), "Local message");
    frame(&mut r, &root, "message-draft-restored");
    tap(&mut r, Key::Center);
    assert!(r.pending_commit());
    save(&mut r, &root);
    for _ in 0..8 {
        r.poll_services().unwrap();
        if r.pending_commit() {
            save(&mut r, &root);
        }
    }
    assert_eq!(traffic.borrow().commands.len(), 1);
    assert_eq!(traffic.borrow().acks, [1, 2]);
    assert!(r.mailbox().outbox[0].sent);
    assert_eq!(r.mailbox().inbox[0].body, "Local reply");
    r.phone.overlay = None;
    r.phone.open_thread("12345".into());
    assert_eq!(r.mailbox().unread(), 0);
    frame(&mut r, &root, "message-received");
    tap(&mut r, Key::Up);
    tap(&mut r, Key::SoftLeft);
    assert!(r.phone.chat.popup == Popup::Bubble);
    tap(&mut r, Key::Center);
    assert!(r.phone.chat.popup == Popup::Delete);
    tap(&mut r, Key::Center);
    assert!(r.mailbox().inbox.is_empty());
    save(&mut r, &root);

    let http = Http::new();
    r.navigate(&http.base).unwrap();
    pump(&mut r, &mut native, clock, |r| !r.phone.browser.loading);
    focus(&mut r, |e| matches!(e, Element::Link { .. }));
    tap(&mut r, Key::Center);
    pump(&mut r, &mut native, clock, |r| !r.phone.browser.loading);
    assert!(r.phone.browser.page.url.ends_with("/next"));
    focus(&mut r, |e| matches!(e, Element::Field { .. }));
    tap(&mut r, Key::Center);
    r.text("hello world");
    tap(&mut r, Key::Center);
    focus(&mut r, |e| matches!(e, Element::Check { .. }));
    tap(&mut r, Key::Center);
    focus(&mut r, |e| matches!(e, Element::Select { .. }));
    tap(&mut r, Key::Center);
    tap(&mut r, Key::Down);
    tap(&mut r, Key::Center);
    frame(&mut r, &root, "browser-form");
    focus(&mut r, |e| matches!(e, Element::Submit { .. }));
    tap(&mut r, Key::Center);
    pump(&mut r, &mut native, clock, |r| !r.phone.browser.loading);
    let url = url::Url::parse(&r.phone.browser.page.url).unwrap();
    let query = url
        .query_pairs()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(query.get("q").unwrap(), "hello world");
    assert_eq!(query.get("check").unwrap(), "yes");
    assert_eq!(query.get("color").unwrap(), "blue");
    assert_eq!(http.requests.lock().unwrap().len(), 3);
    frame(&mut r, &root, "browser-submitted");
    tap(&mut r, Key::SoftRight);
    assert!(r.phone.browser.page.url.ends_with("/next"));
    assert_eq!(r.phone.browser_view, BrowserView::Page);
    assert!(
        r.phone
            .browser
            .page
            .elements
            .iter()
            .any(|e| matches!(e,Element::Field{value,..} if value=="hello world"))
    );
    save(&mut r, &root);

    r.phone.launch(Screen::Music);
    tap(&mut r, Key::Center);
    assert_eq!(r.phone.music_view, MusicView::Player);
    assert_eq!(r.phone.music.volume, 1);
    pump(&mut r, &mut native, clock, |r| r.phone.music.position > 0.1);
    frame(&mut r, &root, "music-wav-playing");
    tap(&mut r, Key::Center);
    assert!(!r.phone.music.playing);
    native.poll(&mut r).unwrap();
    let paused = r.phone.music.position;
    thread::sleep(Duration::from_millis(200));
    native.poll(&mut r).unwrap();
    assert!((r.phone.music.position - paused).abs() < 0.05);
    tap(&mut r, Key::Right);
    tap(&mut r, Key::Center);
    assert!(
        r.phone.music.tracks[r.phone.music.index]
            .0
            .ends_with(".mp3")
    );
    pump(&mut r, &mut native, clock, |r| r.phone.music.position > 0.1);
    frame(&mut r, &root, "music-mp3-playing");
    tap(&mut r, Key::Center);
    native.poll(&mut r).unwrap();
    let position = r.phone.music.position;
    native.save(&r, &root, true).unwrap();
    save(&mut r, &root);
    drop(native);
    let bytes = fs::read(root.join("runtime.json")).unwrap();
    let now = r.time();
    drop(r);
    let mut restored =
        Runtime::restore(config, now, &bytes, Box::new(Loopback(traffic.clone()))).unwrap();
    assert_eq!(restored.phone.screen, Screen::Home);
    assert_eq!(restored.phone.preferences.greeting, "Acceptance profile");
    assert_eq!(restored.phone.preferences.accent, 4);
    assert!(restored.mailbox().outbox[0].sent);
    assert!(restored.mailbox().inbox.is_empty());
    let selected = storage::setup::select(&root, None, false).unwrap().unwrap();
    let mut native = Native::from_folder(&mut restored, Some(selected), &root).unwrap();
    pump(&mut restored, &mut native, clock, |r| {
        !r.phone.music_loading
    });
    assert!(!restored.phone.music.playing);
    assert!(
        restored.phone.music.tracks[restored.phone.music.index]
            .0
            .ends_with(".mp3")
    );
    assert!((restored.phone.music.position - position).abs() < 0.08);
    restored.phone.launch(Screen::Music);
    restored.phone.music_view = MusicView::Player;
    restored.phone.music.queue = vec![restored.phone.music.index];
    restored.phone.music.position = restored.phone.music.duration() - 0.1;
    restored.phone.music.seek_revision += 1;
    restored.phone.music.playing = true;
    pump(&mut restored, &mut native, clock, |r| {
        !r.phone.music.playing
    });
    frame(&mut restored, &root, "music-eof");
    eprintln!(
        "ACCEPTANCE PASS: folder reuse, edit/send/receive/read/delete, HTTP link/GET/back, WAV/MP3 output/pause/skip/restore/EOF. Evidence: {}",
        root.display()
    );
}
