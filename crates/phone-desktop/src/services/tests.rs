use super::*;
use crate::shell::{Overlay, Shell};
use phone_core::services::{Alert, AlertLevel, Capabilities, SubmissionState};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
#[derive(Default)]
struct Traffic {
    events: VecDeque<Envelope>,
    commands: Vec<Command>,
    acks: Vec<u64>,
}
struct TestBackend(Rc<RefCell<Traffic>>);
impl Backend for TestBackend {
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
        Ok(self.0.borrow_mut().events.pop_front())
    }
    fn acknowledge(&mut self, sequence: u64) -> Result<(), PortError> {
        self.0.borrow_mut().acks.push(sequence);
        Ok(())
    }
}
fn fixture() -> (Services, Rc<RefCell<Traffic>>, Shell) {
    let traffic = Rc::new(RefCell::new(Traffic::default()));
    (
        Services::new(Box::new(TestBackend(traffic.clone()))),
        traffic,
        Shell::new(Default::default()),
    )
}
#[test]
fn dispatch_waits_for_disk_and_acceptance_is_not_delivery() {
    let (mut services, traffic, mut shell) = fixture();
    shell
        .messaging
        .mailbox
        .stage_service_queue(
            None,
            phone_core::sms::Message {
                to: "123".into(),
                body: "test".into(),
                ..Default::default()
            },
            Some("test"),
        )
        .unwrap();
    shell.messaging.dirty = true;
    shell.poll_services(&mut services).unwrap();
    assert!(traffic.borrow().commands.is_empty());
    shell.storage_saved();
    shell.poll_services(&mut services).unwrap();
    shell.poll_services(&mut services).unwrap();
    assert_eq!(traffic.borrow().commands.len(), 1);
    assert!(!shell.messaging.mailbox.outbox[0].sent);
    traffic.borrow_mut().events.push_back(Envelope {
        sequence: 1,
        event: Event::MessageStatus {
            request_id: 1,
            state: SubmissionState::Sent,
        },
    });
    shell.poll_services(&mut services).unwrap();
    assert!(shell.messaging.mailbox.outbox[0].sent);
    assert!(traffic.borrow().acks.is_empty());
    shell.storage_failed();
    shell.poll_services(&mut services).unwrap();
    assert!(traffic.borrow().acks.is_empty());
    shell.storage_saved();
    shell.poll_services(&mut services).unwrap();
    assert_eq!(traffic.borrow().acks, [1]);
}
#[test]
fn incoming_is_saved_before_ack_and_does_not_replace_composition() {
    let (mut services, traffic, mut shell) = fixture();
    shell.screen = crate::shell::Screen::Compose;
    shell.messaging.compose.body = phone_core::editor::Editor::new("草稿", 4096);
    traffic.borrow_mut().events.push_back(Envelope {
        sequence: 1,
        event: Event::MessageReceived {
            from: "123".into(),
            body: "hello".into(),
            time_ms: 42,
        },
    });
    shell.poll_services(&mut services).unwrap();
    assert_eq!(shell.messaging.compose.body.text(), "草稿");
    assert_eq!(shell.messaging.mailbox.inbox[0].time, 42);
    assert!(traffic.borrow().acks.is_empty());
    shell.storage_saved();
    shell.poll_services(&mut services).unwrap();
    assert_eq!(traffic.borrow().acks, [1]);
}
#[test]
fn alert_waits_for_editor_and_dismissal_is_durable() {
    let (mut services, traffic, mut shell) = fixture();
    shell.screen = crate::shell::Screen::Compose;
    traffic.borrow_mut().events.push_back(Envelope {
        sequence: 1,
        event: Event::AlertPublished {
            alert: Alert {
                id: "weather".into(),
                title: "天气提醒".into(),
                body: "测试".into(),
                level: AlertLevel::Warning,
                issued_ms: 1,
                expires_ms: None,
            },
        },
    });
    shell.poll_services(&mut services).unwrap();
    shell.storage_saved();
    shell.poll_services(&mut services).unwrap();
    assert!(shell.overlay.is_none());
    shell.screen = crate::shell::Screen::Home;
    shell.poll_services(&mut services).unwrap();
    assert!(matches!(shell.overlay, Some(Overlay::ServiceAlert { .. })));
    shell.handle(
        phone_core::input::Event {
            key: phone_core::input::Key::Center,
            phase: phone_core::input::Phase::Down,
        },
        10,
    );
    assert!(shell.messaging.mailbox.services.alerts[0].dismissed);
    assert!(shell.messaging.dirty);
    assert!(shell.overlay.is_none());
}

#[test]
fn crash_replay_uses_saved_cursor_and_send_keeps_its_id() {
    let (mut services, traffic, mut shell) = fixture();
    traffic.borrow_mut().events.push_back(Envelope {
        sequence: 1,
        event: Event::MessageReceived {
            from: "123".into(),
            body: "once".into(),
            time_ms: 4,
        },
    });
    shell.poll_services(&mut services).unwrap();
    let dir = std::env::temp_dir().join(format!("ta1278-service-replay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    crate::storage::save(&dir, &shell.messaging.mailbox).unwrap();
    // Process dies after atomic write but before transport ack.
    let mut shell = Shell::new(crate::storage::load(&dir).unwrap());
    let mut services = Services::new(Box::new(TestBackend(traffic.clone())));
    traffic.borrow_mut().events.push_back(Envelope {
        sequence: 1,
        event: Event::MessageReceived {
            from: "123".into(),
            body: "once".into(),
            time_ms: 4,
        },
    });
    shell.poll_services(&mut services).unwrap();
    shell.poll_services(&mut services).unwrap();
    assert_eq!(shell.messaging.mailbox.inbox.len(), 1);
    assert_eq!(traffic.borrow().acks, [1]);
    let receipt = shell
        .messaging
        .mailbox
        .stage_service_queue(
            None,
            phone_core::sms::Message {
                to: "123".into(),
                body: "out".into(),
                ..Default::default()
            },
            Some("test"),
        )
        .unwrap();
    assert_eq!(receipt.index, 0);
    crate::storage::save(&dir, &shell.messaging.mailbox).unwrap();
    shell.poll_services(&mut services).unwrap();
    // Adapter must deduplicate this same ID after restart, not issue another send.
    let mut shell = Shell::new(crate::storage::load(&dir).unwrap());
    shell
        .poll_services(&mut Services::new(Box::new(TestBackend(traffic.clone()))))
        .unwrap();
    assert_eq!(traffic.borrow().commands.len(), 2);
    assert_eq!(traffic.borrow().commands[0], traffic.borrow().commands[1]);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn withdrawn_or_expired_alert_closes_without_losing_underlying_screen() {
    let (mut services, traffic, mut shell) = fixture();
    for (sequence, event) in [
        (
            1,
            Event::AlertPublished {
                alert: Alert {
                    id: "weather".into(),
                    title: "提醒".into(),
                    body: "内容".into(),
                    level: AlertLevel::Warning,
                    issued_ms: 1,
                    expires_ms: None,
                },
            },
        ),
        (
            2,
            Event::AlertWithdrawn {
                id: "weather".into(),
            },
        ),
    ] {
        traffic
            .borrow_mut()
            .events
            .push_back(Envelope { sequence, event });
        shell.poll_services(&mut services).unwrap();
        shell.storage_saved();
        shell.poll_services(&mut services).unwrap();
        assert_eq!(shell.overlay.is_some(), sequence == 1);
    }
    assert_eq!(shell.screen, crate::shell::Screen::Home);
    traffic.borrow_mut().events.push_back(Envelope {
        sequence: 3,
        event: Event::AlertPublished {
            alert: Alert {
                id: "expired".into(),
                title: "提醒".into(),
                body: "内容".into(),
                level: AlertLevel::Information,
                issued_ms: 1,
                expires_ms: Some(2),
            },
        },
    });
    shell.poll_services(&mut services).unwrap();
    shell.storage_saved();
    shell.poll_services(&mut services).unwrap();
    assert!(shell.messaging.mailbox.services.alerts.is_empty());
    assert!(shell.overlay.is_none());
}
