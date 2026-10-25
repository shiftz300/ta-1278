//! Local mailbox views; native conversation geometry remains unverified.
use super::*;
use crate::phone::mail::MailPage;
const BLACK: Color = [0, 0, 0, 255];
pub(super) fn folder(c: &mut Canvas, p: &Phone) {
    let drafts = p.screen == Screen::Drafts;
    c.text(6, 1, if drafts { "草稿" } else { "发件箱" }, 14, WHITE);
    let messages = if drafts {
        &p.mailbox.drafts
    } else {
        &p.mailbox.outbox
    };
    if messages.is_empty() {
        c.text(70, 132, "没有信息", 20, MUTED);
    }
    let start = p.index.saturating_sub(3);
    for (i, m) in messages.iter().enumerate().skip(start).take(4) {
        let y = 24 + (i - start) as i32 * 64;
        if i == p.index {
            c.rect(0, y, 240, 64, c.accent(3));
        }
        c.fit(
            7,
            y + 8,
            if m.to.is_empty() {
                "无收件人"
            } else {
                &m.to
            },
            20,
            WHITE,
            226,
        );
        c.fit(
            7,
            y + 36,
            &m.body.replace(['\n', '\r'], " "),
            18,
            MUTED,
            226,
        );
    }
    c.rect(0, 282, 240, 38, BLACK);
    super::softkeys::browser(
        c,
        false,
        if messages.is_empty() { "" } else { "打开" },
        false,
    );
}
fn read(c: &mut Canvas, p: &mut Phone) {
    let Some(m) = p.mail_message().cloned() else {
        return;
    };
    c.text(6, 1, "未发送", 14, WHITE);
    c.fit(7, 29, &format!("至：{}", m.to), 18, MUTED, 225);
    let lines = c.wrap(&m.body, 20, 216);
    p.mail_ui.lines = lines.len();
    p.mail_ui.scroll = p.mail_ui.scroll.min(lines.len().saturating_sub(7));
    for (i, line) in lines.iter().skip(p.mail_ui.scroll).take(7).enumerate() {
        c.text(7, 64 + i as i32 * 29, &line.text, 20, WHITE);
    }
    if lines.len() > 7 {
        c.rect(235, 64, 3, 203, [42, 47, 42, 255]);
        let height = (203 * 7 / lines.len()).max(8) as i32;
        let y = 64 + (203 - height) * p.mail_ui.scroll as i32 / (lines.len() - 7) as i32;
        c.rect(235, y, 3, height, MUTED);
    }
}
pub(super) fn render(c: &mut Canvas, p: &mut Phone, page: MailPage) {
    use MailPage::*;
    match page {
        Read => {
            read(c, p);
            super::softkeys::browser(c, true, "", false);
        }
        Options => {
            read(c, p);
            c.dim();
            let labels = ["重新发送", "编辑为新信息", "转发", "删除"];
            for (i, label) in labels.iter().enumerate() {
                let y = 124 + i as i32 * 38;
                c.rect(
                    0,
                    y,
                    240,
                    38,
                    if i == p.index { c.accent(3) } else { BLACK },
                );
                c.text(7, y + 7, label, 20, WHITE);
            }
            c.rect(0, 282, 240, 38, BLACK);
            super::softkeys::browser(c, false, "选择", false);
        }
        Delete => {
            c.text(6, 1, "信息", 14, WHITE);
            c.text(44, 190, "删除此信息？", 20, WHITE);
            super::softkeys::browser(c, false, "确认", false);
        }
        Result => {
            c.text(6, 1, "信息", 14, WHITE);
            c.text(84, 91, "未发送", 24, WHITE);
            c.text(22, 141, "短信服务未连接", 20, WHITE);
            c.text(22, 172, "信息已保存在本机", 18, MUTED);
            super::softkeys::browser(c, false, "查看", false);
            c.text(7, 290, "重试", 20, WHITE);
        }
    }
}
