//! Original group 0x401f; IDs and draw call sites in docs/evidence/snake-recovery.json.
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
    macro_rules! png {
        ($name:literal) => {
            include_bytes!(concat!(
                "../../../nokia-ta1278-research/analysis/extracted/png/",
                $name
            ))
            .as_slice()
        };
    }
    [
        png!("02624-icon-00ad3704.png"),
        png!("02625-icon-00ad379c.png"),
        png!("02626-icon-00ada83c.png"),
        png!("02629-icon-00adad24.png"),
        png!("02631-icon-00adb3d8.png"),
        png!("02632-icon-00adb7dc.png"),
        png!("02633-icon-00adb914.png"),
        png!("02634-icon-00adba58.png"),
    ]
    .iter()
    .map(|b| {
        image::load_from_memory(b)
            .map(|i| i.to_rgba8())
            .map_err(|e| e.to_string())
    })
    .collect()
}
