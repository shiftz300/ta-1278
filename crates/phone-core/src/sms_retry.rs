//! Bounded 17.00.17.02 error-handler decision; not a modem or a retry timer.
//! Tables: scripts/recover-sms-lifecycle.py, docs/evidence/sms-lifecycle.json.
#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Ignore,
    Finish { finish_flag: bool },
    Continue { retries: u8 },
}
pub fn allowed(cause: u16) -> bool {
    include_bytes!("../../../assets/ui/sms-retry-causes.bin")
        .chunks_exact(2)
        .any(|pair| u16::from_le_bytes([pair[0], pair[1]]) == cause)
}
/// cause is the error event; rl_cause is its network reason. Neither is a UI label ID.
/// Continue denotes the native continuation branch, not permission to send immediately.
pub fn decide(cause: u16, rl_cause: u16, retries: u8) -> Decision {
    if cause == 0 {
        Decision::Ignore
    } else if matches!(cause, 4 | 9) {
        Decision::Finish { finish_flag: false }
    } else if retries > 0 && allowed(rl_cause) {
        Decision::Continue {
            retries: retries - 1,
        }
    } else {
        Decision::Finish { finish_flag: true }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn firmware_cause_domain_and_error_priority() {
        let mut count = 0;
        for code in 0..=u16::MAX {
            let d = decide(1, code, 2);
            if matches!(d, Decision::Continue { .. }) {
                count += 1;
            }
            assert_eq!(decide(0, code, 255), Decision::Ignore);
            for cause in [4, 9] {
                assert_eq!(
                    decide(cause, code, 255),
                    Decision::Finish { finish_flag: false }
                );
            }
            assert_eq!(decide(1, code, 0), Decision::Finish { finish_flag: true });
        }
        assert_eq!(count, 19);
        assert_eq!(decide(1, 17, 1), Decision::Continue { retries: 0 });
        assert_eq!(decide(1, 17, 255), Decision::Continue { retries: 254 });
        assert_eq!(decide(1, 256, 2), Decision::Finish { finish_flag: true });
    }
}
