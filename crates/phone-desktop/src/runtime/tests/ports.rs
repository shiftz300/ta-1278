//! State isolation and host substitution without a window, audio device or network.
use super::*;
use crate::{
    persistence::{Record, Resume, Store},
    platform::{Documents, Instance, Page, Platform},
};
use std::collections::HashMap;

#[derive(Default)]
struct Memory {
    records: HashMap<Record, Vec<u8>>,
    fail: Option<Record>,
}
impl Store for Memory {
    fn read(&mut self, record: Record) -> Result<Option<Vec<u8>>, String> {
        Ok(self.records.get(&record).cloned())
    }
    fn write(&mut self, record: Record, bytes: &[u8]) -> Result<(), String> {
        if self.fail == Some(record) {
            return Err("Injected write failure".into());
        }
        self.records.insert(record, bytes.to_vec());
        Ok(())
    }
}

#[test]
fn profile_and_checkpoint_share_state_and_preserve_clock_policies_and_resume() {
    let mut runtime = Runtime::new(Default::default(), time(0)).unwrap();
    let p = &mut runtime.shell;
    p.contacts.records.push(
        ["", "A", "B", "123", "", "", "", "", "Friends"]
            .map(str::to_owned)
            .to_vec(),
    );
    p.contacts.groups.push("Friends".into());
    p.calls.records.push(crate::shell::calls::Record {
        number: "123".into(),
        time: WALL,
    });
    p.playback.playlists.push(crate::shell::music::Playlist {
        name: "List".into(),
        paths: vec!["/song.wav".into()],
    });
    p.records_dirty = true;
    p.settings.preferences.volume = 2;
    p.settings.dirty = true;
    p.tools.timer.remaining_ms = 20_000;
    p.tools.timer.running = true;
    p.tools.dirty = true;
    p.playback
        .restore(crate::shell::music::SavedMusic {
            path: Some("/song.wav".into()),
            position: 1.5,
            volume: Some(3),
            ..Default::default()
        })
        .unwrap();
    let mut store = Memory::default();
    runtime.flush(&mut store, true).unwrap();
    runtime.save_to(&mut store).unwrap();
    let checkpoint = Runtime::restore_from(
        Default::default(),
        time(5000),
        &mut store,
        Box::new(Offline),
    )
    .unwrap();
    let profile = Runtime::load_profile(
        Default::default(),
        time(5000),
        &mut store,
        Box::new(Offline),
        Resume::WallClock,
    )
    .unwrap();
    assert_eq!(checkpoint.shell.tools.timer.remaining_ms, 20_000);
    assert_eq!(profile.shell.tools.timer.remaining_ms, 15_000);
    for restored in [checkpoint, profile] {
        assert_eq!(restored.shell.contacts.records[0][3], "123");
        assert_eq!(restored.shell.contacts.groups, ["Friends"]);
        assert_eq!(restored.shell.calls.records[0].number, "123");
        assert_eq!(restored.shell.playback.playlists[0].paths, ["/song.wav"]);
        assert_eq!(restored.shell.settings.preferences.volume, 2);
        assert_eq!(
            restored.shell.playback.resume.as_ref().unwrap().position,
            1.5
        );
        assert!(!restored.shell.playback.model.playing);
    }
    // Existing snapshot version 1 files had no music field.
    let mut old: serde_json::Value =
        serde_json::from_slice(&store.records[&Record::Snapshot]).unwrap();
    old.as_object_mut().unwrap().remove("music");
    assert!(
        Runtime::restore(
            Default::default(),
            time(0),
            &serde_json::to_vec(&old).unwrap(),
            Box::new(Offline)
        )
        .is_ok()
    );
}

#[test]
fn failed_message_record_rolls_back_send_without_losing_other_successful_sections() {
    let traffic = Rc::new(RefCell::new(Traffic::default()));
    let mut runtime = Runtime::with_backend(
        Default::default(),
        time(0),
        Box::new(Adapter(traffic.clone())),
    )
    .unwrap();
    runtime.shell.go(Screen::Compose);
    runtime.text("123");
    runtime.tab();
    runtime.text("hello");
    runtime.shell.settings.preferences.volume = 1;
    runtime.shell.settings.dirty = true;
    tap(&mut runtime, Key::Center);
    let mut store = Memory {
        fail: Some(Record::Messages),
        ..Default::default()
    };
    assert!(runtime.flush(&mut store, false).is_err());
    assert!(store.records.contains_key(&Record::Preferences));
    assert!(!runtime.shell.settings.dirty);
    assert_eq!(runtime.shell.messaging.compose.body.text(), "hello");
    assert!(runtime.mailbox().outbox.is_empty());
    runtime.poll_services().unwrap();
    assert!(traffic.borrow().commands.is_empty());
    store.fail = None;
    runtime.shell.overlay = None;
    tap(&mut runtime, Key::Center);
    runtime.flush(&mut store, false).unwrap();
    runtime.poll_services().unwrap();
    assert_eq!(traffic.borrow().commands.len(), 1);
    assert!(store.records.contains_key(&Record::Messages));
}

#[test]
fn receiver_ack_waits_for_record_commit_and_retry_preserves_the_event() {
    let traffic = Rc::new(RefCell::new(Traffic::default()));
    traffic.borrow_mut().events.push_back(Envelope {
        sequence: 1,
        event: ServiceEvent::MessageReceived {
            from: "123".into(),
            body: "reply".into(),
            time_ms: WALL,
        },
    });
    let mut runtime = Runtime::with_backend(
        Default::default(),
        time(0),
        Box::new(Adapter(traffic.clone())),
    )
    .unwrap();
    let mut store = Memory {
        fail: Some(Record::Messages),
        ..Default::default()
    };
    runtime.poll_services().unwrap();
    assert!(runtime.flush(&mut store, false).is_err());
    runtime.poll_services().unwrap();
    assert!(traffic.borrow().acks.is_empty());
    store.fail = None;
    runtime.flush(&mut store, false).unwrap();
    runtime.poll_services().unwrap();
    assert_eq!(traffic.borrow().acks, [1]);
    assert_eq!(runtime.mailbox().inbox.len(), 1);
}

struct Host {
    instance: Instance,
    submitted: Vec<Request>,
    completed: VecDeque<(Request, Result<Page, String>)>,
    cancelled: usize,
}
impl Platform for Host {
    fn instance(&self) -> Instance {
        self.instance
    }
    fn documents(&mut self) -> Option<&mut dyn Documents> {
        Some(self)
    }
}
impl Documents for Host {
    fn submit(&mut self, request: Request, _: Option<Page>) -> Result<(), String> {
        self.submitted.push(request);
        Ok(())
    }
    fn poll(&mut self) -> Option<(Request, Result<Page, String>)> {
        self.completed.pop_front()
    }
    fn cancel(&mut self) {
        self.cancelled += 1;
    }
}
#[test]
fn interchangeable_platform_rejects_old_results_and_restored_instance_bindings() {
    let mut runtime = Runtime::new(Default::default(), time(0)).unwrap();
    let mut host = Host {
        instance: runtime.instance(),
        submitted: vec![],
        completed: VecDeque::new(),
        cancelled: 0,
    };
    runtime.navigate("https://example.com/old").unwrap();
    runtime.poll_platform(&mut host).unwrap();
    let old = host.submitted[0].clone();
    runtime.navigate("https://example.com/new").unwrap();
    runtime.poll_platform(&mut host).unwrap();
    let new = host.submitted[1].clone();
    host.completed.push_back((
        old,
        Ok(crate::platform::parse(
            "https://example.com/old",
            "<title>old</title>",
        )),
    ));
    host.completed.push_back((
        new,
        Ok(crate::platform::parse(
            "https://example.com/new",
            "<title>new</title>",
        )),
    ));
    runtime.poll_platform(&mut host).unwrap();
    assert_eq!(runtime.shell.browsing.model.page.title, "new");
    let mut saved = Vec::new();
    runtime
        .save(|bytes| {
            saved = bytes.to_vec();
            Ok(())
        })
        .unwrap();
    let mut restored =
        Runtime::restore(Default::default(), time(0), &saved, Box::new(Offline)).unwrap();
    assert!(restored.poll_platform(&mut host).is_err());
    assert!(restored.shell.browsing.model.page.url.is_empty());
}

#[test]
fn painting_is_read_only_and_input_consumes_measured_geometry() {
    let mut runtime = Runtime::new(Default::default(), time(0)).unwrap();
    runtime.shell.go(Screen::Browser);
    runtime.shell.browsing.view = crate::shell::BrowserView::Page;
    runtime.shell.browsing.model.page = crate::platform::parse(
        "https://example.com",
        "<p>One</p><p>Two</p><a href='/next'>Next</a>",
    );
    runtime.shell.browsing.model.content_height = 0;
    runtime.shell.browsing.model.bounds.clear();
    let first = runtime.render().pixels.to_vec();
    assert_eq!(runtime.shell.browsing.model.content_height, 0);
    assert!(runtime.shell.browsing.model.bounds.is_empty());
    assert_eq!(first, runtime.render().pixels);
    tap(&mut runtime, Key::Down);
    assert_eq!(
        first, runtime.canvas.pixels,
        "Measurement does not repaint the frame"
    );
    assert!(runtime.shell.browsing.model.content_height > 0);
    assert!(!runtime.shell.browsing.model.bounds.is_empty());
}

#[test]
fn earlier_record_failure_blocks_send_until_retry_without_rolling_back_unattempted_mail() {
    let traffic = Rc::new(RefCell::new(Traffic::default()));
    let mut runtime = Runtime::with_backend(
        Default::default(),
        time(0),
        Box::new(Adapter(traffic.clone())),
    )
    .unwrap();
    runtime.shell.go(Screen::Compose);
    runtime.text("123");
    runtime.tab();
    runtime.text("hello");
    runtime.shell.settings.dirty = true;
    tap(&mut runtime, Key::Center);
    let mut store = Memory {
        fail: Some(Record::Preferences),
        ..Default::default()
    };
    assert!(runtime.flush(&mut store, false).is_err());
    assert!(runtime.shell.messaging.ui.pending.is_some());
    assert!(!store.records.contains_key(&Record::Messages));
    runtime.poll_services().unwrap();
    assert!(traffic.borrow().commands.is_empty());
    store.fail = None;
    runtime.flush(&mut store, false).unwrap();
    runtime.poll_services().unwrap();
    assert_eq!(traffic.borrow().commands.len(), 1);
}

#[test]
fn shared_close_keeps_draft_after_failure_and_blocks_mutating_files() {
    let mut runtime = Runtime::new(Default::default(), time(0)).unwrap();
    let mut store = Memory {
        fail: Some(Record::Messages),
        ..Default::default()
    };
    runtime.shell.go(Screen::Compose);
    runtime.text("123");
    runtime.tab();
    runtime.text("retain draft");
    assert!(!runtime.close(&mut store));
    assert_eq!(runtime.shell.messaging.compose.body.text(), "retain draft");
    assert_eq!(runtime.shell.screen, Screen::Compose);
    store.fail = None;
    assert!(runtime.close(&mut store));
    let restored = Runtime::load_profile(
        Default::default(),
        time(0),
        &mut store,
        Box::new(Offline),
        Resume::Paused,
    )
    .unwrap();
    assert_eq!(restored.mailbox().drafts.len(), 1);
    assert_eq!(restored.mailbox().drafts[0].body, "retain draft");
    runtime.shell.files.busy = true;
    assert!(!runtime.close(&mut store));
    assert!(
        matches!(&runtime.shell.overlay, Some(Overlay::Notice(message)) if message.contains("文件操作尚未结束"))
    );
}
