//! Shared handset controls. Coordinates and hit testing use the same composed frame.
mod paint;
#[cfg(test)]
mod tests;
use phone_core::input::Key;
use std::collections::HashMap;
pub const WIDTH: u32 = 240;
pub const HEIGHT: u32 = 576;
const FIRST_SOURCE: u32 = 0x20000;

#[derive(Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub lcd_height: Option<f32>,
}
impl Viewport {
    pub fn fit(width: u32, height: u32, keypad: bool) -> Self {
        let h = if keypad { HEIGHT } else { 320 } as f32;
        let mut scale = (width as f32 / WIDTH as f32).min(height as f32 / h);
        if !keypad && scale >= 1. {
            scale = scale.floor();
        }
        let (w, h) = (WIDTH as f32 * scale, h * scale);
        Self {
            x: (width as f32 - w) / 2.,
            y: (height as f32 - h) / 2.,
            w,
            h,
            lcd_height: None,
        }
    }
    /// Keep the LCD aspect ratio; let physical controls use the remaining window.
    pub fn fullscreen(width: u32, height: u32) -> Self {
        Self {
            x: 0.,
            y: 0.,
            w: width as f32,
            h: height as f32,
            lcd_height: Some((width as f32 * 320. / WIDTH as f32).min(height as f32 * 0.7)),
        }
    }
    pub fn point(self, x: f64, y: f64) -> Option<(f32, f32)> {
        let (x, y) = (x as f32 - self.x, y as f32 - self.y);
        (self.w > 0. && self.h > 0. && x >= 0. && y >= 0. && x < self.w && y < self.h).then(|| {
            let y = match self.lcd_height {
                Some(lcd) if y >= lcd => 320. + (y - lcd) / (self.h - lcd) * (HEIGHT - 320) as f32,
                Some(lcd) => y / lcd * 320.,
                None => y / self.h * HEIGHT as f32,
            };
            (x / self.w * WIDTH as f32, y)
        })
    }
}
const PAD: [i32; 4] = [87, 349, 66, 72];
/// Four equal-area sectors in pad-local coordinates, plus an independent center.
/// Painting and hit testing use the same map, including the rounded corners.
fn pad_key(x: f32, y: f32) -> Option<Key> {
    let [left, top, width, height] = PAD;
    let dx = (x - left as f32) * 2. / width as f32 - 1.;
    let dy = (y - top as f32) * 2. / height as f32 - 1.;
    let (ax, ay) = (dx.abs(), dy.abs());
    if ax >= 1. || ay >= 1. {
        return None;
    }
    let (rx, ry) = ((ax - 0.75).max(0.), (ay - 0.75).max(0.));
    if rx * rx + ry * ry > 0.25 * 0.25 {
        return None;
    }
    Some(if ax.max(ay) < 0.3 {
        Key::Center
    } else if ax > ay {
        if dx < 0. { Key::Left } else { Key::Right }
    } else if dy < 0. {
        Key::Up
    } else {
        Key::Down
    })
}
#[derive(Clone, Copy)]
struct Button {
    key: Key,
    rect: [i32; 4],
    label: &'static str,
    letters: &'static str,
}
impl Button {
    fn contains(self, x: f32, y: f32) -> bool {
        let [left, top, w, h] = self.rect;
        x >= left as f32 && y >= top as f32 && x < (left + w) as f32 && y < (top + h) as f32
    }
}
const BUTTONS: [Button; 16] = [
    Button {
        key: Key::SoftLeft,
        rect: [12, 348, 64, 30],
        label: "",
        letters: "",
    },
    Button {
        key: Key::SoftRight,
        rect: [164, 348, 64, 30],
        label: "",
        letters: "",
    },
    Button {
        key: Key::Call,
        rect: [12, 388, 64, 30],
        label: "",
        letters: "",
    },
    Button {
        key: Key::End,
        rect: [164, 388, 64, 30],
        label: "",
        letters: "",
    },
    Button {
        key: Key::Digit(1),
        rect: [12, 430, 66, 30],
        label: "1",
        letters: "",
    },
    Button {
        key: Key::Digit(2),
        rect: [87, 430, 66, 30],
        label: "2",
        letters: "abc",
    },
    Button {
        key: Key::Digit(3),
        rect: [162, 430, 66, 30],
        label: "3",
        letters: "def",
    },
    Button {
        key: Key::Digit(4),
        rect: [12, 464, 66, 30],
        label: "4",
        letters: "ghi",
    },
    Button {
        key: Key::Digit(5),
        rect: [87, 464, 66, 30],
        label: "5",
        letters: "jkl",
    },
    Button {
        key: Key::Digit(6),
        rect: [162, 464, 66, 30],
        label: "6",
        letters: "mno",
    },
    Button {
        key: Key::Digit(7),
        rect: [12, 498, 66, 30],
        label: "7",
        letters: "pqrs",
    },
    Button {
        key: Key::Digit(8),
        rect: [87, 498, 66, 30],
        label: "8",
        letters: "tuv",
    },
    Button {
        key: Key::Digit(9),
        rect: [162, 498, 66, 30],
        label: "9",
        letters: "wxyz",
    },
    Button {
        key: Key::Star,
        rect: [12, 532, 66, 30],
        label: "*",
        letters: "+",
    },
    Button {
        key: Key::Digit(0),
        rect: [87, 532, 66, 30],
        label: "0",
        letters: "",
    },
    Button {
        key: Key::Hash,
        rect: [162, 532, 66, 30],
        label: "#",
        letters: "",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pointer {
    Mouse,
    Touch(u64),
}
struct Held {
    key: Key,
    source: u32,
}
pub struct Keypad {
    held: HashMap<Pointer, Held>,
    painter: paint::Painter,
    pixels: Vec<u8>,
    dirty: bool,
}
impl Keypad {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            dirty: true,
            held: HashMap::new(),
            painter: paint::Painter::new()?,
            pixels: vec![0; (WIDTH * HEIGHT * 4) as usize],
        })
    }
    pub fn hit(&self, viewport: Viewport, x: f64, y: f64) -> Option<Key> {
        let (x, y) = viewport.point(x, y)?;
        pad_key(x, y).or_else(|| BUTTONS.iter().find(|b| b.contains(x, y)).map(|b| b.key))
    }
    pub fn press(&mut self, id: Pointer, key: Key) -> Option<u32> {
        if self.held.contains_key(&id) || self.held.len() >= 16 {
            return None;
        }
        let source = (FIRST_SOURCE..FIRST_SOURCE + 16)
            .find(|s| self.held.values().all(|h| h.source != *s))?;
        self.held.insert(id, Held { key, source });
        self.dirty = true;
        Some(source)
    }
    pub fn key(&self, id: Pointer) -> Option<Key> {
        self.held.get(&id).map(|h| h.key)
    }
    pub fn release(&mut self, id: Pointer) -> Option<u32> {
        self.held.remove(&id).map(|h| {
            self.dirty = true;
            h.source
        })
    }
    pub fn clear(&mut self) {
        self.held.clear();
        self.dirty = true;
    }
    pub fn compose(&mut self, screen: &[u8]) -> &[u8] {
        self.pixels[..screen.len()].copy_from_slice(screen);
        if self.dirty {
            self.painter.paint(&mut self.pixels, |key| {
                self.held.values().any(|h| h.key == key)
            });
            self.dirty = false;
        }
        &self.pixels
    }
}
