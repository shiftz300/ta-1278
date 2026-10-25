use super::*;
pub(super) fn render(c: &mut Canvas, p: &Phone, kind: MenuKind, index: usize) {
    let labels = p.menu_labels(kind);
    let cols = if kind == MenuKind::Smileys { 4 } else { 7 };
    let width = 238 / cols as i32;
    let height = if cols == 4 { 48 } else { 35 };
    chrome::title(c, if cols == 4 { "表情符号" } else { "符号" });
    for (i, label) in labels.iter().enumerate() {
        let x = (i % cols) as i32 * width;
        let y = 40 + (i / cols) as i32 * height;
        if i == index {
            c.rect(x, y, width, height, c.accent(3));
        }
        let shown = if label == "\n" { "↵" } else { label };
        let w = c.width(shown, 20);
        c.text(x + (width - w) / 2, y + (height - 24) / 2, shown, 20, WHITE);
    }
    Softkeys::back("选择").draw(c);
}
