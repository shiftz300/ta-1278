use super::*;
pub(super) fn render(c: &mut Canvas, p: &Phone, kind: MenuKind, index: usize) {
    let labels = p.menu_labels(kind);
    let cols = if kind == MenuKind::Smileys { 4 } else { 7 };
    let width = 238 / cols as i32;
    let height = if cols == 4 { 60 } else { 34 };
    chrome::title(c, if cols == 4 { "表情符号" } else { "符号" });
    for (i, label) in labels.iter().enumerate() {
        let x = (i % cols) as i32 * width;
        let y = 24 + (i / cols) as i32 * height;
        if i == index {
            c.rect(x, y, width, height, c.accent(3));
        }
        let shown = if label == "\n" { "↵" } else { label };
        let size = if cols == 4 { 20 } else { 16 };
        let w = c.width(shown, size);
        c.text(
            x + (width - w) / 2,
            y + (height - i32::from(size)) / 2 - 4,
            shown,
            size,
            WHITE,
        );
    }
    Softkeys::back("选择").draw(c);
}
