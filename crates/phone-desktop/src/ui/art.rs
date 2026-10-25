//! Original control artwork; loaded once and drawn without scaling.
use crate::canvas::Canvas;
use std::sync::OnceLock;
fn images() -> &'static Vec<image::RgbaImage> {
    static IMAGES: OnceLock<Vec<image::RgbaImage>> = OnceLock::new();
    IMAGES.get_or_init(|| {
        [
            include_bytes!(
                "../../../../nokia-ta1278-research/analysis/extracted/png/02516-icon-00ababf4.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../../nokia-ta1278-research/analysis/extracted/png/02521-icon-00abc0bc.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../../nokia-ta1278-research/analysis/extracted/png/02520-icon-00abbc94.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../../nokia-ta1278-research/analysis/extracted/png/02522-icon-00abc4e4.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../../nokia-ta1278-research/analysis/extracted/png/02519-icon-00abb86c.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../../nokia-ta1278-research/analysis/extracted/png/02518-icon-00abb444.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../../nokia-ta1278-research/analysis/extracted/png/02517-icon-00abb01c.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../../nokia-ta1278-research/analysis/extracted/png/02509-icon-00aba4bc.png"
            )
            .as_slice(),
        ]
        .into_iter()
        .map(|bytes| {
            image::load_from_memory(bytes)
                .expect("verified control image")
                .to_rgba8()
        })
        .collect()
    })
}
pub fn power(c: &mut Canvas, on: bool) {
    c.image(&images()[if on { 1 + c.accent % 6 } else { 0 }], 94, 100);
}
pub fn phone(c: &mut Canvas, x: i32) {
    c.image(&images()[7], x, 287);
}
