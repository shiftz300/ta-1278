//! Original control artwork; loaded once and drawn without scaling.
use crate::canvas::Canvas;
use std::sync::OnceLock;
fn images() -> &'static Vec<image::RgbaImage> {
    static IMAGES: OnceLock<Vec<image::RgbaImage>> = OnceLock::new();
    IMAGES.get_or_init(|| {
        [
            crate::assets::png!("02516-icon-00ababf4.png"),
            crate::assets::png!("02521-icon-00abc0bc.png"),
            crate::assets::png!("02520-icon-00abbc94.png"),
            crate::assets::png!("02522-icon-00abc4e4.png"),
            crate::assets::png!("02519-icon-00abb86c.png"),
            crate::assets::png!("02518-icon-00abb444.png"),
            crate::assets::png!("02517-icon-00abb01c.png"),
            crate::assets::png!("02509-icon-00aba4bc.png"),
            crate::assets::png!("02658-icon-00ae03e0.png"),
            crate::assets::png!("02523-icon-00abc90c.png"),
            crate::assets::png!("02656-icon-00ae01b4.png"),
            crate::assets::png!("02657-icon-00ae028c.png"),
            crate::assets::png!("02484-icon-00ab93e4.png"),
            crate::assets::png!("02486-icon-00ab9454.png"),
            crate::assets::png!("02488-icon-00ab9500.png"),
            crate::assets::png!("02490-icon-00ab9658.png"),
            crate::assets::png!("02492-icon-00ab9714.png"),
            crate::assets::png!("02493-icon-00ab9748.png"),
            crate::assets::png!("02461-icon-00ab4cf8.png"),
            crate::assets::png!("02474-icon-00ab7dbc.png"),
            crate::assets::png!("02524-icon-00abcbc8.png"),
            crate::assets::png!("02525-icon-00abcd24.png"),
        ]
        .into_iter()
        .map(|bytes| crate::assets::decode(bytes).expect("verified control image"))
        .collect()
    })
}
pub fn power(c: &mut Canvas, on: bool) {
    c.image(&images()[if on { 1 + c.accent % 6 } else { 0 }], 94, 100);
}
pub fn shell(c: &mut Canvas, x: i32) {
    c.image(&images()[7], x, 287);
}

pub fn storage(c: &mut Canvas, x: i32, y: i32) {
    c.image(&images()[8], x, y);
}
pub fn hourglass(c: &mut Canvas, x: i32, y: i32) {
    c.image(&images()[9], x, y);
}

#[derive(Clone, Copy)]
pub enum UtilityIcon {
    Folder = 10,
    File,
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,
    Percent,
    Recordings,
    Record,
    Reset,
    Lap,
}
pub fn utility(c: &mut Canvas, icon: UtilityIcon, x: i32, y: i32) {
    c.image(&images()[icon as usize], x, y);
}
