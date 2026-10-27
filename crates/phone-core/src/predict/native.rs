//! Bounded translations from firmware 17.00.17.02. Not a candidate engine.
//! Byte/call-site checks: scripts/recover-t9-core.py; docs/t9-recovery.md.
/// 0x880d929c..0x880d9314: record span from a tag byte. The caller supplies
/// the circular-buffer helper's next uint16 only for tag 0xff. This helper
/// is used on runtime records; it must not be assumed to decode an LDB trie.
pub fn record_span(tag: u8, extended: Option<u16>) -> Option<usize> {
    let n = usize::from(tag & 63) + 1;
    Some(match tag >> 6 {
        0 => n + 3,
        1 => n * 2 + n.div_ceil(2) + 3,
        2 => n.div_ceil(2) + 5,
        _ if n <= 63 => n,
        _ => usize::from(extended?.wrapping_add(1)),
    })
}
/// Valid depth store at 0x880d7020..2c, after its context guard succeeds.
/// Other guard/rebuild calls are not reconstructed here.
pub fn context_depth(depth: u32) -> Option<u16> {
    (depth <= 8).then_some(depth as u16)
}
/// Post-helper flag updates in AWClrLdbCompletion / AWClrUdbCustWordCompletion.
/// Does not emulate their field validation or helper side effects.
pub fn clear_completion_flags(flags: u32) -> u32 {
    flags & !(0x10000 | 0x800)
}

/// A payload-only circular record buffer. 0x880e6984 reads high byte first;
/// host bounds checks reject an incomplete/zero-sized record rather than looping.
pub fn record_span_at(payload: &[u8], offset: usize) -> Option<usize> {
    let tag = *payload.get(offset)?;
    let extended = if tag == 255 {
        if payload.len() < 3 {
            return None;
        }
        Some(u16::from_be_bytes([
            payload[(offset + 1) % payload.len()],
            payload[(offset + 2) % payload.len()],
        ]))
    } else {
        None
    };
    record_span(tag, extended).filter(|&n| n > 0 && n <= payload.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn circular_records_use_big_endian_length_and_reject_invalid_spans() {
        assert_eq!(record_span_at(&[], 0), None);
        assert_eq!(record_span_at(&[255, 0], 0), None);
        assert_eq!(record_span_at(&[255, 0, 3, 0], 0), Some(4));
        assert_eq!(record_span_at(&[3, 0, 255, 0], 2), Some(4));
        assert_eq!(record_span_at(&[255, 3, 0, 0], 0), None);
        assert_eq!(record_span_at(&[255, 255, 255], 0), None);
        assert_eq!(record_span_at(&[0, 0, 0, 0], 4), None);
    }
    #[test]
    fn record_span_handles_tag_classes_and_extended_wrap() {
        for tag in 0u8..=254 {
            let low = usize::from(tag & 63);
            let expected = match tag >> 6 {
                0 => low + 4,
                1 => 2 * (low + 1) + ((low + 2) >> 1) + 3,
                2 => ((low + 2) >> 1) + 5,
                _ => low + 1,
            };
            assert_eq!(record_span(tag, None), Some(expected));
        }
        assert_eq!(record_span(255, None), None);
        for value in 0u16..=u16::MAX {
            assert_eq!(
                record_span(255, Some(value)),
                Some((usize::from(value) + 1) & 65535)
            );
            assert_eq!(context_depth(u32::from(value)).is_some(), value <= 8);
        }
        assert_eq!(clear_completion_flags(u32::MAX), 0xfffef7ff);
    }
}
