//! Native page chrome with a bounded HTML text adapter, not the original layout engine.
use super::*;
pub(super) const PAPER: Color = [246, 252, 246, 255];
pub(super) const INK: Color = [0, 8, 0, 255];
pub(super) const FOCUS: Color = [239, 154, 0, 255];
pub(super) const LINK: Color = [0, 0, 238, 255];
pub(super) fn render(c: &mut Canvas, p: &mut Phone) {
    if matches!(
        p.browser_view,
        BrowserView::Settings | BrowserView::HomeType | BrowserView::FontSize | BrowserView::Clear
    ) {
        super::browser_chrome::settings(c, p);
        return;
    }
    if p.browser_view != BrowserView::Page {
        let name = match p.browser_view {
            BrowserView::Settings => "设置",
            BrowserView::HomeType => "主页类型",
            BrowserView::FontSize => "字体大小",
            BrowserView::Clear => "清除",
            BrowserView::Restore => "恢复预设设置？",
            BrowserView::Bookmarks => "书签",
            BrowserView::History => "历史",
            BrowserView::Offline => "离线页面",
            _ => "互联网",
        };
        chrome::title(c, name);
        let labels = p.browser_items();
        if labels.is_empty() {
            c.text(8, 120, "这里没有可显示的内容。", 17, WHITE);
        }
        for (i, label) in labels
            .iter()
            .enumerate()
            .skip(p.index.saturating_sub(5))
            .take(6)
        {
            let y = 24 + (i - p.index.saturating_sub(5)) as i32 * 42;
            if p.browser_view == BrowserView::Start && i == 0 {
                c.rect(0, 24, 240, 38, PAPER);
                c.rect(2, 27, 235, 28, if i == p.index { FOCUS } else { PAPER });
                c.rect(3, 28, 233, 26, PAPER);
                c.icon(22, 3, 29, 24);
                c.text(30, 31, label, 14, INK);
            } else {
                if i == p.index {
                    c.rect(0, y, 240, 42, c.accent(3));
                }
                c.fit(6, y + 8, label, 20, WHITE, 228);
            }
        }
        Softkeys::menu("选择").draw(c);
        return;
    }
    let font = [12, 14][p.browser_data.settings.font_size.min(1)];
    let layout = super::browser_layout::layout(c, &p.browser.page, font);
    let top = layout.height;
    p.browser.content_height = top;
    p.browser.bounds = layout.bounds.clone();
    p.browser.scroll = p.browser.scroll.min(top.saturating_sub(216));
    c.rect(0, 64, 240, 216, PAPER);
    super::browser_layout::paint(c, p, &layout);
    // Clip content by repainting the fixed chrome, including glyphs above/below the viewport.
    c.rect(0, 0, 240, 64, [0, 0, 0, 255]);
    c.rect(0, 24, 240, 40, PAPER);
    chrome::title(c, "互联网");
    status(c);
    let address = if p.browser.loading && !p.requested_url.is_empty() {
        &p.requested_url
    } else {
        &p.browser.page.url
    };
    c.ring(12, 42, 10, [174, 184, 174, 255]);
    c.ring(12, 42, 6, [211, 218, 211, 255]);
    c.rect(24, 30, 213, 26, FOCUS);
    c.rect(25, 31, 211, 24, PAPER);
    c.fit(27, 34, address, 14, INK, 205);
    c.rect(0, 280, 240, 40, [0, 0, 0, 255]);
    Softkeys::new(
        Button::Menu,
        Button::Text("选择"),
        if p.browser.loading {
            Button::Close
        } else {
            Button::Back
        },
    )
    .draw(c);
    if top > 216 {
        c.rect(236, 64, 3, 216, [190, 196, 190, 255]);
        c.rect(
            236,
            64 + (p.browser.scroll * 200 / top) as i32,
            3,
            16,
            [110, 116, 110, 255],
        );
    }
    if let Some(error) = &p.browser.error {
        c.rect(0, 240, 234, 40, [0, 0, 0, 255]);
        c.fit(5, 244, error, 14, WHITE, 224);
    }
}
