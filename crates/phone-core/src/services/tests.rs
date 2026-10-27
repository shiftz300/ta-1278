use super::*;
use crate::sms::{Mailbox, Message};
fn event(sequence: u64, event: Event) -> Envelope {
    Envelope { sequence, event }
}
fn incoming(sequence: u64) -> Envelope {
    event(
        sequence,
        Event::MessageReceived {
            from: "123".into(),
            body: "测试".into(),
            time_ms: 42,
        },
    )
}
fn alert() -> Alert {
    Alert {
        id: "weather-1".into(),
        title: "天气提醒".into(),
        body: "测试预警".into(),
        level: AlertLevel::Warning,
        issued_ms: 10,
        expires_ms: None,
    }
}
#[test]
fn durable_replay_and_order_do_not_duplicate_inbox() {
    let mut m = Mailbox::default();
    assert!(m.apply_service_event("test", &incoming(2)).is_err());
    assert!(m.inbox.is_empty());
    assert_eq!(
        m.apply_service_event("test", &incoming(1)),
        Ok(Apply::Received(0))
    );
    let saved = serde_json::to_string(&m).unwrap();
    let mut m: Mailbox = serde_json::from_str(&saved).unwrap();
    assert_eq!(
        m.apply_service_event("test", &incoming(1)),
        Ok(Apply::Replay)
    );
    assert_eq!(m.inbox.len(), 1);
    assert_eq!(m.inbox[0].time, 42);
    assert!(!m.inbox[0].read);
    assert_eq!(
        m.apply_service_event("other", &incoming(1)),
        Ok(Apply::Received(1))
    );
}
#[test]
fn invalid_input_leaves_cursor_and_mailbox_unchanged() {
    let mut m = Mailbox::default();
    let before = serde_json::to_vec(&m).unwrap();
    assert!(
        m.apply_service_event(
            "test",
            &event(
                1,
                Event::MessageReceived {
                    from: "".into(),
                    body: "test".into(),
                    time_ms: 0,
                }
            )
        )
        .is_err()
    );
    assert_eq!(before, serde_json::to_vec(&m).unwrap());
    assert!(
        m.apply_service_event(
            "test",
            &event(
                1,
                Event::Call(CallEvent::Incoming {
                    call_id: "c1".into(),
                    from: "123".into(),
                })
            )
        )
        .is_err()
    );
    assert!(m.services.cursors.is_empty());
}
#[test]
fn explicit_send_ids_and_receipts_survive_reload_and_never_regress() {
    let mut m: Mailbox =
        serde_json::from_str(r#"{"drafts":[],"outbox":[{"to":"123","body":"old"}]}"#).unwrap();
    let msg = Message {
        to: "123".into(),
        body: "new".into(),
        ..Default::default()
    };
    m.queue(None, msg.clone()).unwrap();
    assert!(m.outbox.iter().all(|m| m.submission.is_none()));
    let receipt = m
        .stage_service_queue(None, msg.clone(), Some("test"))
        .unwrap();
    m.rollback_queue(receipt).unwrap();
    m.stage_service_queue(None, msg, Some("test")).unwrap();
    let id = m
        .outbox
        .last()
        .unwrap()
        .submission
        .as_ref()
        .unwrap()
        .request_id;
    assert_eq!(id, 2); // Failed stage cannot accidentally correlate with a later request.
    for (i, state) in [
        SubmissionState::Accepted,
        SubmissionState::Sent,
        SubmissionState::Delivered,
        SubmissionState::Accepted,
        SubmissionState::Failed {
            reason: "late".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        m.apply_service_event(
            "test",
            &event(
                i as u64 + 1,
                Event::MessageStatus {
                    request_id: id,
                    state,
                },
            ),
        )
        .unwrap();
    }
    let saved = serde_json::to_string(&m).unwrap();
    let mut m: Mailbox = serde_json::from_str(&saved).unwrap();
    let last = m.outbox.last().unwrap();
    assert!(last.sent);
    assert_eq!(
        last.submission.as_ref().unwrap().state,
        SubmissionState::Delivered
    );
    m.outbox.pop();
    m.apply_service_event(
        "test",
        &event(
            6,
            Event::MessageStatus {
                request_id: id,
                state: SubmissionState::Delivered,
            },
        ),
    )
    .unwrap();
    assert_eq!(m.outbox.len(), 2);
}
#[test]
fn alerts_update_withdraw_and_remain_separate_from_sms() {
    let mut m = Mailbox::default();
    m.apply_service_event("test", &event(1, Event::AlertPublished { alert: alert() }))
        .unwrap();
    m.services.alerts[0].dismissed = true;
    let saved = serde_json::to_string(&m).unwrap();
    let mut m: Mailbox = serde_json::from_str(&saved).unwrap();
    assert!(m.services.alerts[0].dismissed);
    let mut update = alert();
    update.body = "更新".into();
    m.apply_service_event("test", &event(2, Event::AlertPublished { alert: update }))
        .unwrap();
    assert_eq!(m.services.alerts.len(), 1);
    assert!(!m.services.alerts[0].dismissed);
    m.apply_service_event(
        "test",
        &event(
            3,
            Event::AlertWithdrawn {
                id: "weather-1".into(),
            },
        ),
    )
    .unwrap();
    assert!(m.services.alerts.is_empty());
    assert!(m.inbox.is_empty());
}
