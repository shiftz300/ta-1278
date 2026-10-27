//! Native page chrome with a bounded HTML text adapter, not the original layout engine.
pub(super) mod chrome;
pub(super) mod layout;
use super::*;
pub(super) const PAPER: Color = [246, 252, 246, 255];
pub(super) const INK: Color = [0, 8, 0, 255];
pub(super) const FOCUS: Color = [239, 154, 0, 255];
pub(super) const LINK: Color = [0, 0, 238, 255];
pub(super) fn render(c: &mut Canvas, p: &Shell) {
    if p.browsing.view == BrowserView::Details {
        crate::ui::chrome::title(c, "页面详情");
        c.text(10, 32, "书签名称", 20, WHITE);
        c.fit(
            10,
            74,
            if p.browsing.model.page.title.is_empty() {
                "无标题"
            } else {
                &p.browsing.model.page.title
            },
            20,
            WHITE,
            224,
        );
        c.text(10, 114, "URL:", 16, WHITE);
        let lines = c.wrap(&p.browsing.model.page.url, 16, 224);
        let index = p.index.min(lines.len().saturating_sub(5));
        c.layout.index = Some(index);
        for (i, line) in lines.iter().skip(index).take(5).enumerate() {
            c.text(10, 154 + i as i32 * 23, &line.text, 16, WHITE);
        }
        Softkeys::back("").draw(c);
        return;
    }
    if matches!(
        p.browsing.view,
        BrowserView::Settings | BrowserView::HomeType | BrowserView::FontSize | BrowserView::Clear
    ) {
        chrome::settings(c, p);
        return;
    }
    if p.browsing.view != BrowserView::Page {
        let name = match p.browsing.view {
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
        crate::ui::chrome::title(c, name);
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
            if p.browsing.view == BrowserView::Start && i == 0 {
                c.rect(0, 24, 240, 38, PAPER);
                c.rect(2, 27, 235, 28, if i == p.index { FOCUS } else { PAPER });
                c.rect(3, 28, 233, 26, PAPER);
                c.icon(20, 3, 29, 24);
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
    let font = [12, 14][p.browsing.data.settings.font_size.min(1)];
    let layout = layout::layout(c, &p.browsing.model.page, font);
    let top = layout.height;
    let scroll = p.browsing.model.scroll.min(top.saturating_sub(216));
    c.layout.browser = Some(crate::ui::layout::Browser {
        height: top,
        bounds: layout.bounds.clone(),
        scroll,
    });
    c.rect(0, 64, 240, 216, PAPER);
    layout::paint(c, p, &layout, scroll);
    // Clip content by repainting the fixed chrome, including glyphs above/below the viewport.
    c.rect(0, 0, 240, 64, [0, 0, 0, 255]);
    c.rect(0, 24, 240, 40, PAPER);
    crate::ui::chrome::title(c, "互联网");
    status(c);
    let address = if p.browsing.model.loading && !p.browsing.requested.is_empty() {
        &p.browsing.requested
    } else {
        &p.browsing.model.page.url
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
        if p.browsing.model.loading {
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
            64 + (scroll * 200 / top) as i32,
            3,
            16,
            [110, 116, 110, 255],
        );
    }
    if let Some(error) = &p.browsing.model.error {
        c.rect(0, 240, 234, 40, [0, 0, 0, 255]);
        c.fit(5, 244, error, 14, WHITE, 224);
    }
}
