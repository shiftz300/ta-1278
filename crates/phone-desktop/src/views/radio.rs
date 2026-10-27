//! Radio content and its observed overlay, sharing system popup/softkey controls.
use super::*;
use crate::shell::tools::ToolPage;
pub(super) fn render(c: &mut Canvas, p: &Shell) {
    chrome::title(c, "收音机");
    let t = &p.tools;
    c.rect(0, 24, 240, 90, c.accent(3));
    let frequency = format!("{:.1}", t.frequency as f32 / 10.);
    let frequency_x = (240 - c.width(&frequency, 18)) / 2;
    c.text(frequency_x, 38, &frequency, 18, WHITE);
    c.text(107, 65, "MHz", 12, WHITE);
    c.text(3, 80, "87.5", 12, MUTED);
    let maximum_x = 237 - c.width("108.0", 12);
    c.text(maximum_x, 80, "108.0", 12, MUTED);
    for i in 0..59 {
        c.rect(
            3 + i * 4,
            109 - if i % 5 == 0 { 8 } else { 5 },
            1,
            if i % 5 == 0 { 8 } else { 5 },
            MUTED,
        );
    }
    c.rect(0, 114, 240, 206, [31, 29, 32, 255]);
    c.rounded([46, 132, 148, 148], 29, [0, 0, 0, 255]);
    use crate::ui::art::{UtilityIcon, utility};
    utility(c, UtilityIcon::Plus, 105, 132);
    utility(c, UtilityIcon::Minus, 105, 256);
    let label = if t.radio_on { "静音" } else { "取消静音" };
    let label_x = (240 - c.width(label, 18)) / 2;
    c.text(label_x, 200, label, 18, WHITE);
    for d in 0..12 {
        c.rect(50 + d, 207 - d, 1, 1, WHITE);
        c.rect(50 + d, 207 + d, 1, 1, WHITE);
        c.rect(188 - d, 207 - d, 1, 1, WHITE);
        c.rect(188 - d, 207 + d, 1, 1, WHITE);
    }
    Softkeys::menu("").draw(c);
    if t.page == ToolPage::RadioOptions {
        let items = p.radio_menu();
        PopupMenu::default().draw(c, items.len(), p.index, |i| items[i].row());
    }
}
