use super::*;
use phone_core::services::{
    Alert, AlertLevel, Capabilities, Command, Envelope, Event as ServiceEvent, PortError,
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
const WALL: u64 = 1_793_000_000_000;
fn time(elapsed_ms: u64) -> Time {
    Time {
        elapsed_ms,
        unix_ms: WALL + elapsed_ms,
    }
}
fn tap(runtime: &mut Runtime, key: Key) {
    runtime.press(1, key);
    runtime.release(1);
}
#[derive(Default)]
struct Traffic {
    events: VecDeque<Envelope>,
    commands: Vec<Command>,
    acks: Vec<u64>,
}
struct Adapter(Rc<RefCell<Traffic>>);
impl Backend for Adapter {
    fn id(&self) -> &str {
        "test"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            send_messages: true,
            receive_messages: true,
            alerts: true,
        }
    }
    fn submit(&mut self, command: Command) -> Result<(), PortError> {
        self.0.borrow_mut().commands.push(command);
        Ok(())
    }
    fn poll(&mut self) -> Result<Option<Envelope>, PortError> {
        Ok(self.0.borrow().events.front().cloned())
    }
    fn acknowledge(&mut self, sequence: u64) -> Result<(), PortError> {
        let mut traffic = self.0.borrow_mut();
        traffic.acks.push(sequence);
        traffic.events.pop_front();
        Ok(())
    }
}
#[test]
fn host_clock_controls_timers_messages_and_rendering_without_advancing_on_render() {
    let mut runtime = Runtime::new(RuntimeConfig::default(), time(0)).unwrap();
    runtime.shell.tools.timer.remaining_ms = 2000;
    runtime.shell.tools.timer.running = true;
    let pixels = runtime.render().pixels.to_vec();
    assert_eq!(pixels, runtime.render().pixels);
    assert_eq!(runtime.shell.tools.timer.remaining_ms, 2000);
    runtime.advance(time(500)).unwrap();
    assert_eq!(runtime.shell.tools.timer.remaining_ms, 1500);
    assert!(runtime.advance(time(100)).is_err());
    assert_eq!(runtime.time(), time(500));
    runtime.shell.go(Screen::Messages);
    tap(&mut runtime, Key::Center);
    assert_eq!(runtime.shell.messaging.compose.time, WALL + 500);
    runtime.text("123");
    runtime.tab();
    runtime.text("hello");
    runtime.text("不允许宿主输入法");
    assert_eq!(runtime.shell.messaging.compose.body.text(), "hello");
    runtime.press(2, Key::End);
    runtime.cancel_input();
    runtime.advance(time(700)).unwrap();
    assert_eq!(runtime.shell.tools.timer.remaining_ms, 1300);
    let mut saved = Vec::new();
    runtime
        .save(|bytes| {
            saved = bytes.to_vec();
            Ok(())
        })
        .unwrap();
    let restored = Runtime::restore(
        RuntimeConfig::default(),
        time(60000),
        &saved,
        Box::new(Offline),
    )
    .unwrap();
    assert_eq!(restored.shell.tools.timer.remaining_ms, 1300);
    assert_eq!(restored.shell.screen, Screen::Home);
    assert!(restored.mailbox().drafts.iter().any(|m| m.body == "hello"));
}
#[test]
fn host_save_failure_rolls_back_send_and_success_precedes_submit() {
    let traffic = Rc::new(RefCell::new(Traffic::default()));
    let mut runtime = Runtime::with_backend(
        RuntimeConfig::default(),
        time(0),
        Box::new(Adapter(traffic.clone())),
    )
    .unwrap();
    runtime.shell.go(Screen::Messages);
    tap(&mut runtime, Key::Center);
    runtime.text("123");
    runtime.tab();
    runtime.text("hello");
    tap(&mut runtime, Key::Center);
    assert!(runtime.shell.messaging.ui.pending.is_some());
    runtime.poll_services().unwrap();
    assert!(traffic.borrow().commands.is_empty());
    assert!(
        runtime
            .save(|_| Err("host transaction failed".into()))
            .is_err()
    );
    assert!(runtime.mailbox().outbox.is_empty());
    assert_eq!(runtime.shell.messaging.compose.body.text(), "hello");
    tap(&mut runtime, Key::Center);
    let mut saved = Vec::new();
    runtime
        .save(|bytes| {
            saved = bytes.to_vec();
            Ok(())
        })
        .unwrap();
    assert!(traffic.borrow().commands.is_empty());
    runtime.poll_services().unwrap();
    runtime.poll_services().unwrap();
    assert_eq!(traffic.borrow().commands.len(), 1);
    assert!(!runtime.mailbox().outbox[0].sent);
    let mut restored = Runtime::restore(
        RuntimeConfig::default(),
        time(0),
        &saved,
        Box::new(Adapter(traffic.clone())),
    )
    .unwrap();
    restored.poll_services().unwrap();
    assert_eq!(traffic.borrow().commands.len(), 2);
    assert_eq!(
        traffic.borrow().commands[0],
        traffic.borrow().commands[1],
        "Restart retains the idempotency key; backend must deduplicate"
    );
}
#[test]
fn incoming_ack_replay_and_alert_expiry_use_host_commit_and_clock() {
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
        RuntimeConfig::default(),
        time(0),
        Box::new(Adapter(traffic.clone())),
    )
    .unwrap();
    runtime.poll_services().unwrap();
    assert_eq!(runtime.mailbox().inbox.len(), 1);
    assert!(runtime.save(|_| Err("disk unavailable".into())).is_err());
    runtime.poll_services().unwrap();
    assert!(traffic.borrow().acks.is_empty());
    let mut saved = Vec::new();
    runtime
        .save(|bytes| {
            saved = bytes.to_vec();
            Ok(())
        })
        .unwrap();
    // Restore between persistence and ack; the backend replays sequence 1.
    let mut runtime = Runtime::restore(
        RuntimeConfig::default(),
        time(0),
        &saved,
        Box::new(Adapter(traffic.clone())),
    )
    .unwrap();
    runtime.poll_services().unwrap();
    runtime.poll_services().unwrap();
    assert_eq!(runtime.mailbox().inbox.len(), 1);
    assert_eq!(traffic.borrow().acks, [1]);
    traffic.borrow_mut().events.push_back(Envelope {
        sequence: 2,
        event: ServiceEvent::AlertPublished {
            alert: Alert {
                id: "notice".into(),
                title: "Test".into(),
                body: "Notice".into(),
                level: AlertLevel::Information,
                issued_ms: WALL,
                expires_ms: Some(WALL + 1000),
            },
        },
    });
    runtime.poll_services().unwrap();
    runtime.save(|_| Ok(())).unwrap();
    runtime.poll_services().unwrap();
    assert_eq!(runtime.mailbox().services.alerts.len(), 1);
    runtime.render();
    runtime.poll_services().unwrap();
    assert_eq!(runtime.mailbox().services.alerts.len(), 1);
    runtime.advance(time(1001)).unwrap();
    runtime.poll_services().unwrap();
    assert!(runtime.mailbox().services.alerts.is_empty());
}
#[test]
fn browser_rejects_stale_generation_and_different_instance_results() {
    let mut runtime = Runtime::new(RuntimeConfig::default(), time(0)).unwrap();
    runtime.navigate("about:one").unwrap();
    let first = runtime.take_request().unwrap();
    runtime.navigate("about:two").unwrap();
    let second = runtime.take_request().unwrap();
    assert!(!runtime.complete(&first, Ok(crate::platform::parse(&first.url, "old"))));
    assert!(runtime.complete(
        &second,
        Ok(crate::platform::parse(&second.url, "<title>New</title>new"))
    ));
    assert_eq!(runtime.shell.browsing.model.page.title, "New");
    assert_eq!(runtime.shell.browsing.view, crate::shell::BrowserView::Page);
    let mut saved = Vec::new();
    runtime
        .save(|bytes| {
            saved = bytes.to_vec();
            Ok(())
        })
        .unwrap();
    let mut restored =
        Runtime::restore(RuntimeConfig::default(), time(0), &saved, Box::new(Offline)).unwrap();
    restored.navigate("about:three").unwrap();
    let third = restored.take_request().unwrap();
    assert!(!restored.complete(
        &first,
        Ok(crate::platform::parse(&first.url, "wrong instance"))
    ));
    assert!(restored.complete(
        &third,
        Ok(crate::platform::parse(&third.url, "new instance"))
    ));
    let before = restored.render().pixels.to_vec();
    assert!(Runtime::restore(RuntimeConfig::default(), time(0), b"{}", Box::new(Offline)).is_err());
    assert_eq!(before, restored.render().pixels);
}

mod ports;

#[test]
fn host_suspend_cancels_held_keys_and_pauses_playback_without_discarding_records() {
    let mut runtime = Runtime::new(RuntimeConfig::default(), time(0)).unwrap();
    runtime.shell.playback.model.playing = true;
    runtime.shell.playback.model.position = 12.;
    runtime.shell.messaging.compose.body = phone_core::editor::Editor::new("keep this draft", 4096);
    runtime.press(123, Key::Digit(2));
    runtime.suspend_media();
    assert!(!runtime.shell.playback.model.playing);
    assert_eq!(runtime.shell.playback.model.position, 12.);
    assert_eq!(
        runtime.shell.messaging.compose.body.text(),
        "keep this draft"
    );
    assert!(runtime.shell.cues.is_empty());
    assert!(runtime.shell.audio_preview.is_none());
    let frame = runtime.render().pixels.to_vec();
    runtime.release(123);
    assert_eq!(runtime.render().pixels, frame);
}
