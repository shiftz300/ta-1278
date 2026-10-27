//! Decode only build-time embedded resources, with their exact output-size bound.
pub mod soundfont;
pub fn decode((bytes, expected): (&[u8], usize)) -> Result<Vec<u8>, String> {
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(bytes, expected)
        .map_err(|e| format!("Cannot decode embedded resource: {e:?}"))?;
    if raw.len() != expected {
        return Err("Embedded resource length mismatch".into());
    }
    Ok(raw)
}
