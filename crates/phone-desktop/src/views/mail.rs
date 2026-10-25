//! Local mailbox views; native conversation geometry remains unverified.
use super::*;
use crate::phone::mail::MailPage;
const BLACK: Color = [0, 0, 0, 255];
pub(super) fn folder(c: &mut Canvas, p: &Phone) {
    let title = match p.screen {
        Screen::Drafts => "草稿",
        Screen::Inbox => "收件箱",
        _ => "发件箱",
    };
    chrome::title(c, title);
    let count = p.mail_count(p.screen);
    if count == 0 {
        c.text(70, 132, "没有信息", 20, MUTED);
    }
    let start = p.index.saturating_sub(3);
    for i in start..(start + 4).min(count) {
        let m = p.mail_content(p.screen, i).unwrap();
        let y = 24 + (i - start) as i32 * 64;
        if i == p.index {
            c.rect(0, y, 240, 64, c.accent(3));
        }
        c.fit(
            7,
            y + 8,
            if m.peer.is_empty() {
                "无收件人"
            } else {
                m.peer
            },
            20,
            WHITE,
            if m.unread { 205 } else { 226 },
        );
        if m.unread {
            c.ring(226, y + 18, 4, WHITE);
        }
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
    Softkeys::back(if count == 0 { "" } else { "打开" }).draw(c);
}
fn read(c: &mut Canvas, p: &mut Phone) {
    let Some(m) = p.mail_message() else {
        return;
    };
    let incoming = p.mail_ui.folder == Screen::Inbox;
    chrome::title(c, if incoming { "信息" } else { "未发送" });
    c.fit(
        7,
        29,
        &format!("{}：{}", if incoming { "来自" } else { "至" }, m.peer),
        18,
        MUTED,
        225,
    );
    let lines = c.wrap(m.body, 20, 216);
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
            Softkeys::menu("").draw(c);
        }
        Options => {
            read(c, p);
            c.dim();
            let actions = p.mail_actions();
            list::popup(c, actions.len(), p.index, |i| {
                list::Row::text(actions[i].label())
            });
            c.rect(0, 282, 240, 38, BLACK);
            Softkeys::back("选择").draw(c);
        }
        Delete => {
            chrome::title(c, "信息");
            c.text(44, 190, "删除此信息？", 20, WHITE);
            Softkeys::back("确认").draw(c);
        }
        Result => {
            chrome::title(c, "信息");
            c.text(84, 91, "未发送", 24, WHITE);
            c.text(22, 141, "短信服务未连接", 20, WHITE);
            c.text(22, 172, "信息已保存在本机", 18, MUTED);
            Softkeys::new(Button::Text("重试"), Button::Text("查看"), Button::Back).draw(c);
        }
    }
}
