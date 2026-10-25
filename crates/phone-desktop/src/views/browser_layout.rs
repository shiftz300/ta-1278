//! Shared geometry for painting and focus scrolling. Coordinates are document-relative.
use super::browser::{FOCUS, INK, LINK, PAPER};
use super::*;
use phone_core::{browser::Page, web_flow::Flow};
#[derive(Clone, Debug)]
pub enum Draw {
    Text {
        x: i32,
        y: i32,
        text: String,
        size: u16,
        bold: bool,
        link: Option<usize>,
    },
    Control {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        index: usize,
    },
    Disabled {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        value: String,
    },
    Box {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    Legend {
        x: i32,
        y: i32,
        text: String,
    },
}
pub struct Layout {
    pub draws: Vec<Draw>,
    pub bounds: Vec<(usize, usize)>,
    pub height: usize,
}
struct Builder<'a> {
    c: &'a mut Canvas,
    draws: Vec<Draw>,
    bounds: Vec<(usize, usize)>,
    x: i32,
    y: i32,
    left: i32,
    right: i32,
    line: i32,
    gap: i32,
    font: u16,
    boxes: Vec<(usize, i32, i32)>,
}
impl Builder<'_> {
    fn br(&mut self, gap: i32) {
        if self.x > self.left {
            self.y += self.line + gap;
        } else {
            self.y += (gap - self.gap).max(0);
        }
        self.x = self.left;
        self.line = i32::from(self.font) + 1;
        self.gap = gap;
    }
    fn mark(&mut self, i: usize, y: i32, h: i32) {
        let b = &mut self.bounds[i];
        if b.1 == 0 {
            *b = (y.max(0) as usize, (y + h).max(0) as usize);
        } else {
            b.0 = b.0.min(y.max(0) as usize);
            b.1 = b.1.max((y + h).max(0) as usize);
        }
    }
    fn words(&mut self, text: &str, size: u16, bold: bool, link: Option<usize>) {
        for chunk in text.split_inclusive(' ') {
            if chunk.trim().is_empty() && self.x == self.left {
                continue;
            }
            let w = self.c.width(chunk, size);
            if self.x + w > self.right && self.x > self.left {
                self.br(0);
            }
            if w > self.right - self.left && chunk.chars().count() > 1 {
                for ch in chunk.chars() {
                    self.words(&ch.to_string(), size, bold, link);
                }
                continue;
            }
            self.draws.push(Draw::Text {
                x: self.x,
                y: self.y,
                text: chunk.into(),
                size,
                bold,
                link,
            });
            if let Some(i) = link {
                self.mark(i, self.y, i32::from(size) + 3);
            }
            self.x += w;
            self.line = self
                .line
                .max(i32::from(size) + if size > self.font { 6 } else { 1 });
            self.gap = 0;
        }
    }
    fn control(&mut self, index: usize, width: i32, height: i32) {
        let w = width.min(self.right - self.left);
        if self.x + w > self.right {
            self.br(0);
        }
        self.draws.push(Draw::Control {
            x: self.x,
            y: self.y + 2,
            w,
            h: height,
            index,
        });
        self.mark(index, self.y + 2, height);
        self.x += w;
        self.line = self.line.max(height);
        self.gap = 0;
    }
}
pub fn layout(c: &mut Canvas, page: &Page, font: u16) -> Layout {
    let mut b = Builder {
        c,
        draws: vec![],
        bounds: vec![(0, 0); page.elements.len()],
        x: 2,
        y: 0,
        left: 2,
        right: 234,
        line: i32::from(font) + 1,
        gap: 0,
        font,
        boxes: vec![],
    };
    let fallback: Vec<Flow> = page
        .elements
        .iter()
        .enumerate()
        .flat_map(|(i, e)| {
            [
                match e {
                    Element::Text(s) => Flow::Text {
                        text: s.clone(),
                        heading: false,
                        bold: false,
                    },
                    _ => Flow::Control {
                        element: i,
                        width: 102,
                        height: 16,
                    },
                },
                Flow::Break { gap: 12 },
            ]
        })
        .collect();
    let flow = if page.flow.is_empty() {
        &fallback
    } else {
        &page.flow
    };
    for item in flow {
        match item {
            Flow::Text {
                text,
                heading,
                bold,
            } => b.words(text, if *heading { font + 5 } else { font }, *bold, None),
            Flow::Break { gap } => b.br(i32::from(*gap)),
            Flow::Indent(dx) => {
                b.br(0);
                b.left = (b.left + i32::from(*dx)).clamp(2, 180);
                b.x = b.left;
            }
            Flow::Control {
                element,
                width,
                height,
            } => {
                let Some(e) = page.elements.get(*element) else {
                    continue;
                };
                match e {
                    Element::Link { label, .. } => b.words(label, font, false, Some(*element)),
                    _ => {
                        let w = if *width == 0 {
                            match e {
                                Element::Select {
                                    options, selected, ..
                                } => {
                                    let text = options
                                        .iter()
                                        .zip(selected)
                                        .find(|(_, s)| **s)
                                        .map_or("", |(o, _)| o.0.as_str());
                                    b.c.width(text, font) + 17
                                }
                                Element::Submit { label, .. } => b.c.width(label, font) + 10,
                                Element::Reset { .. } => b.c.width("Reset", font) + 10,
                                _ => 102,
                            }
                        } else {
                            i32::from(*width)
                        };
                        b.control(*element, w, i32::from(*height));
                    }
                }
            }
            Flow::Disabled {
                width,
                height,
                value,
            } => {
                let w = i32::from(*width);
                let h = i32::from(*height);
                if b.x + w > b.right {
                    b.br(0);
                }
                b.draws.push(Draw::Disabled {
                    x: b.x,
                    y: b.y,
                    w,
                    h,
                    value: value.clone(),
                });
                b.x += w;
                b.line = b.line.max(h);
                b.gap = 0;
            }
            Flow::BoxStart(legend) => {
                b.br(0);
                let id = b.draws.len();
                b.draws.push(Draw::Box {
                    x: 4,
                    y: b.y + 11,
                    w: 228,
                    h: 0,
                });
                b.draws.push(Draw::Legend {
                    x: 8,
                    y: b.y + 2,
                    text: legend.clone(),
                });
                b.boxes.push((id, b.left, b.right));
                b.left = 8;
                b.right = 228;
                b.x = 8;
                b.y += 18;
                b.gap = 0;
            }
            Flow::Bullet => {
                b.draws.push(Draw::Text {
                    x: b.left - 14,
                    y: b.y,
                    text: "•".into(),
                    size: font,
                    bold: false,
                    link: None,
                });
            }
            Flow::BoxEnd => {
                b.br(0);
                if let Some((id, left, right)) = b.boxes.pop() {
                    if let Draw::Box { y, h, .. } = &mut b.draws[id] {
                        *h = (b.y - *y + 2).max(20);
                    }
                    b.left = left;
                    b.right = right;
                    b.x = left;
                    b.y += 4;
                    b.gap = 0;
                }
            }
        }
    }
    b.br(0);
    Layout {
        draws: b.draws,
        bounds: b.bounds,
        height: b.y.max(0) as usize,
    }
}
pub fn paint(c: &mut Canvas, p: &Phone, layout: &Layout) {
    let shift = 64 - p.browser.scroll as i32;
    let font = [12, 14][p.browser_data.settings.font_size.min(1)];
    for item in &layout.draws {
        match item {
            Draw::Text {
                x,
                y,
                text,
                size,
                bold,
                link,
            } => {
                let yy = y + shift;
                if yy + 24 < 64 || yy >= 280 {
                    continue;
                }
                let focused = link.is_some_and(|i| i == p.browser.focus);
                let color = if link.is_some() {
                    if focused { WHITE } else { LINK }
                } else {
                    INK
                };
                let w = c.width(text, *size);
                if focused {
                    c.rect(*x, yy, w, i32::from(*size) + 2, FOCUS);
                }
                c.text(*x, yy - i32::from(*size == font), text, *size, color);
                if *bold {
                    c.text(*x + 1, yy, text, *size, color);
                }
                if link.is_some() {
                    c.rect(*x, yy + i32::from(*size) + 1, w, 1, color);
                }
            }
            Draw::Control { x, y, w, h, index } => {
                let y = y + shift;
                if y + h < 64 || y >= 280 {
                    continue;
                }
                let focused = *index == p.browser.focus;
                let Some(e) = p.browser.page.elements.get(*index) else {
                    continue;
                };
                match e {
                    Element::Check { radio, checked, .. } => {
                        let xx = *x;
                        let size = 9;
                        if focused {
                            c.rect(xx, y, size, size, FOCUS);
                        }
                        if *radio {
                            for dy in -4..=4 {
                                for dx in -4..=4 {
                                    let r = dx * dx + dy * dy;
                                    if (9..=16).contains(&r) {
                                        c.rect(xx + 4 + dx, y + 4 + dy, 1, 1, INK);
                                    }
                                }
                            }
                            if *checked {
                                c.rect(xx + 3, y + 3, 3, 3, INK);
                            }
                        } else {
                            border(c, xx, y, 9, 9, INK);
                            if *checked {
                                c.text(xx, y - 3, "✓", 11, INK);
                            }
                        }
                    }
                    _ => {
                        let readonly = matches!(
                            e,
                            Element::Field {
                                read_only: true,
                                ..
                            }
                        );
                        let fill = if readonly {
                            [174, 174, 158, 255]
                        } else if focused {
                            FOCUS
                        } else {
                            PAPER
                        };
                        c.rect(*x, y, *w, *h, fill);
                        border(c, *x, y, *w, *h, INK);
                        let text = match e {
                            Element::Field {
                                value, password, ..
                            } => {
                                if *password {
                                    "*".repeat(value.chars().count())
                                } else {
                                    value.lines().next().unwrap_or("").into()
                                }
                            }
                            Element::Select {
                                options, selected, ..
                            } => options
                                .iter()
                                .zip(selected)
                                .filter(|(_, s)| **s)
                                .map(|(o, _)| o.0.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                            Element::Submit { label, .. } => label.clone(),
                            Element::Reset { .. } => "Reset".into(),
                            _ => String::new(),
                        };
                        c.fit(
                            *x + 2,
                            y,
                            &text,
                            font,
                            if focused && !readonly {
                                WHITE
                            } else if matches!(e, Element::Select { .. }) {
                                LINK
                            } else {
                                INK
                            },
                            *w - 4
                                - if matches!(e, Element::Select { .. }) {
                                    10
                                } else {
                                    0
                                },
                        );
                        if matches!(e, Element::Select { .. }) {
                            c.text(x + w - 10, y, "▾", font, INK);
                        }
                    }
                }
            }
            Draw::Disabled { x, y, w, h, value } => {
                let y = y + shift;
                c.rect(*x, y, *w, *h, [174, 174, 158, 255]);
                border(c, *x, y, *w, *h, INK);
                c.fit(x + 2, y, value, font, INK, w - 4);
            }
            Draw::Box { x, y, w, h } => border(c, *x, y + shift, *w, *h, [112, 119, 110, 255]),
            Draw::Legend { x, y, text } => {
                let y = y + shift;
                let w = c.width(text, font);
                c.rect(*x, y, w + 4, i32::from(font) + 1, PAPER);
                c.text(*x, y, text, font, INK);
            }
        }
    }
}
fn border(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, color: Color) {
    c.rect(x, y, w, 1, color);
    c.rect(x, y + h - 1, w, 1, color);
    c.rect(x, y, 1, h, color);
    c.rect(x + w - 1, y, 1, h, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inline_form_geometry_and_link_bounds_follow_painted_flow() {
        let page = crate::web::parse(
            "about:fixture",
            include_str!("../../../../scripts/fixtures/browser-form.html"),
        );
        let mut canvas = Canvas::new().unwrap();
        let result = layout(&mut canvas, &page, 12);
        let controls: Vec<_> = result
            .draws
            .iter()
            .filter_map(|d| match d {
                Draw::Control { x, y, w, h, index } => Some((*x, *y, *w, *h, *index)),
                _ => None,
            })
            .collect();
        // Native form: label + field on one row, 27 px paragraph spacing.
        assert_eq!(&controls[0].0, &97);
        assert_eq!(controls[0].2, 102);
        assert_eq!(controls[0].3, 15);
        assert_eq!(controls[1].1 - controls[0].1, 27);
        assert_eq!(controls[2].1 - controls[1].1, 27);
        assert_eq!(controls[4].1 - controls[3].1, 25);
        for (_, y, _, h, i) in controls {
            assert_eq!(result.bounds[i], (y as usize, (y + h) as usize));
        }
        let page = crate::web::parse(
            "about:fixture",
            "<ul><li><a href=next>A long linked sentence that must wrap across more than one row in this narrow viewport</a></li></ul>",
        );
        let result = layout(&mut canvas, &page, 12);
        let lines: Vec<_> = result
            .draws
            .iter()
            .filter_map(|d| match d {
                Draw::Text {
                    x,
                    y,
                    link: Some(i),
                    ..
                } => Some((*x, *y, *i)),
                _ => None,
            })
            .collect();
        assert_eq!(lines[0].0, 22);
        let last = lines.last().unwrap();
        assert!(last.1 > lines[0].1);
        assert_eq!(result.bounds[last.2].0, lines[0].1 as usize);
        assert!(result.bounds[last.2].1 > last.1 as usize);
    }
}
