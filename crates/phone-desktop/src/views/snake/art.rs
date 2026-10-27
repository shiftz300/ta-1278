//! Original group 0x401f; IDs and draw call sites in research archive: docs/evidence/snake-recovery.json.
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Sprite {
    Fruit,
    Background,
    Body,
    Head,
    Score,
    Countdown,
    Wall,
    Bonus,
}
pub fn load() -> Result<Vec<image::RgbaImage>, String> {
    [
        crate::assets::png!("02624-icon-00ad3704.png"),
        crate::assets::png!("02625-icon-00ad379c.png"),
        crate::assets::png!("02626-icon-00ada83c.png"),
        crate::assets::png!("02629-icon-00adad24.png"),
        crate::assets::png!("02631-icon-00adb3d8.png"),
        crate::assets::png!("02632-icon-00adb7dc.png"),
        crate::assets::png!("02633-icon-00adb914.png"),
        crate::assets::png!("02634-icon-00adba58.png"),
    ]
    .into_iter()
    .map(crate::assets::decode)
    .collect()
}
