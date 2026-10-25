//! Native accent table. The handset's saved preference is deliberately not read.
use crate::canvas::Color;
use serde::Deserialize;
#[derive(Deserialize)]
pub struct Theme {
    pub accents_rgb565: [[u16; 7]; 6],
    pub startup_fallback_accent: usize,
    pub candidate_text_rgb565: u16,
}
impl Theme {
    pub fn load() -> Self {
        serde_json::from_str(include_str!("../../../assets/ui/native-theme.json"))
            .expect("verified native theme data")
    }
    pub fn color(&self, accent: usize, shade: usize) -> Color {
        rgb565(self.accents_rgb565[accent % 6][shade.min(6)])
    }
}
pub const fn rgb565(v: u16) -> Color {
    [
        (((v >> 11) & 31) as u32 * 255 / 31) as u8,
        (((v >> 5) & 63) as u32 * 255 / 63) as u8,
        ((v & 31) as u32 * 255 / 31) as u8,
        255,
    ]
}
