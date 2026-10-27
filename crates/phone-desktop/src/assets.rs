//! Embedded handset PNGs. Names retain their extraction IDs for traceability.
macro_rules! png {
    ($name:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/icons/",
            $name
        ))
        .as_slice()
    };
}
pub(crate) use png;

pub fn decode(bytes: &[u8]) -> Result<image::RgbaImage, String> {
    image::load_from_memory(bytes)
        .map(|image| image.to_rgba8())
        .map_err(|error| error.to_string())
}
