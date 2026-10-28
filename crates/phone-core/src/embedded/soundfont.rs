//! Reversible SF2 sample packing. RIFF metadata and 16-bit PCM are restored exactly.
use std::ops::Range;

fn samples(bytes: &[u8]) -> Result<Range<usize>, String> {
    fn word(bytes: &[u8], at: usize) -> Result<usize, String> {
        let value = bytes.get(at..at + 4).ok_or("Truncated SF2 chunk")?;
        Ok(u32::from_le_bytes(value.try_into().unwrap()) as usize)
    }
    fn chunks(
        bytes: &[u8],
        span: Range<usize>,
        key: &[u8; 4],
    ) -> Result<Option<Range<usize>>, String> {
        let mut at = span.start;
        let mut found = None;
        while at < span.end {
            if span.end - at < 8 {
                return Err("Truncated SF2 chunk header".into());
            }
            let size = word(bytes, at + 4)?;
            let start = at + 8;
            let end = start
                .checked_add(size)
                .filter(|end| *end <= span.end)
                .ok_or("SF2 chunk exceeds its container")?;
            if &bytes[at..at + 4] == key {
                // A different LIST type may precede sdta; only select sdta here.
                if key != b"LIST" || (size >= 4 && bytes.get(start..start + 4) == Some(b"sdta")) {
                    if found.is_some() {
                        return Err("Duplicate SF2 sample chunk".into());
                    }
                    found = Some(start..end);
                }
            }
            at = end
                .checked_add(size & 1)
                .filter(|at| *at <= span.end)
                .ok_or("Missing SF2 chunk padding")?;
        }
        Ok(found)
    }
    if bytes.get(..4) != Some(b"RIFF")
        || bytes.get(8..12) != Some(b"sfbk")
        || word(bytes, 4)?.checked_add(8) != Some(bytes.len())
    {
        return Err("Invalid SF2 container".into());
    }
    let sdta = chunks(bytes, 12..bytes.len(), b"LIST")?.ok_or("Missing SF2 sample data")?;
    let smpl =
        chunks(bytes, sdta.start + 4..sdta.end, b"smpl")?.ok_or("Missing SF2 PCM samples")?;
    if smpl.len() % 2 != 0 {
        return Err("Invalid SF2 PCM length".into());
    }
    Ok(smpl)
}

/// Build-time second differences modulo 2^16, followed by low/high byte planes.
pub fn encode(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let span = samples(bytes)?;
    let count = span.len() / 2;
    let mut packed = bytes.to_vec();
    let (mut previous, mut difference) = (0u16, 0u16);
    for (i, sample) in bytes[span.clone()].as_chunks::<2>().0.iter().enumerate() {
        let sample = u16::from_le_bytes([sample[0], sample[1]]);
        let next = sample.wrapping_sub(previous);
        let value = next.wrapping_sub(difference).to_le_bytes();
        packed[span.start + i] = value[0];
        packed[span.start + count + i] = value[1];
        previous = sample;
        difference = next;
    }
    Ok(packed)
}

/// Inverse predictor. The original SF2 can then be passed to any normal SF2 loader.
pub fn restore(bytes: &mut [u8]) -> Result<(), String> {
    let span = samples(bytes)?;
    let encoded = bytes[span.clone()].to_vec();
    let count = encoded.len() / 2;
    let (mut sample, mut difference) = (0u16, 0u16);
    for (i, out) in bytes[span].as_chunks_mut::<2>().0.iter_mut().enumerate() {
        difference = difference.wrapping_add(u16::from_le_bytes([encoded[i], encoded[count + i]]));
        sample = sample.wrapping_add(difference);
        out.copy_from_slice(&sample.to_le_bytes());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_bank_survives_predictor_and_zlib_byte_identically() {
        let raw = include_bytes!("../../../../assets/audio/gm1_A21.sf2");
        let filtered = encode(raw).unwrap();
        assert_eq!(filtered.len(), raw.len());
        assert_ne!(filtered, raw);
        let z = miniz_oxide::deflate::compress_to_vec_zlib(&filtered, 9);
        assert!(z.len() < 2_100_000);
        let mut restored = super::super::decode((&z, raw.len())).unwrap();
        restore(&mut restored).unwrap();
        assert!(restored == raw, "SF2 metadata or PCM changed");
        let span = samples(raw).unwrap();
        assert_eq!(&filtered[..span.start], &raw[..span.start]);
        assert_eq!(&filtered[span.end..], &raw[span.end..]);
    }

    #[test]
    fn truncated_lengths_and_odd_pcm_are_rejected() {
        let raw = include_bytes!("../../../../assets/audio/gm1_A21.sf2");
        assert!(encode(&raw[..raw.len() - 1]).is_err());
        assert!(encode(b"RIFF").is_err());
        let mut bad = raw.to_vec();
        bad[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(restore(&mut bad).is_err());
        let span = samples(raw).unwrap();
        let mut bad = raw.to_vec();
        bad[span.start - 4..span.start].copy_from_slice(&((span.len() - 1) as u32).to_le_bytes());
        assert!(restore(&mut bad).is_err());
    }
}
