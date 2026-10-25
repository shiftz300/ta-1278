use super::*;
use crate::phone::organizer::OrganizerPage;
pub(super) fn render(c: &mut Canvas, p: &mut Phone) {
    use OrganizerPage::*;
    if crate::phone::alarm::is_alarm(p.organizer.page) {
        super::alarm::render(c, p);
        return;
    }
    let page = p.organizer.page;
    if crate::phone::contacts::is_contact(page) {
        super::contacts::render(c, p);
        return;
    }
    chrome::title(c, p.organizer_title());
    let mut left = Button::None;
    let mut center = "选择";
    if page == Calendar {
        let date = p.organizer.date;
        c.text(
            52,
            37,
            &format!("{}年{}月", date.year, date.month),
            24,
            WHITE,
        );
        for (i, s) in ["一", "二", "三", "四", "五", "六", "日"]
            .iter()
            .enumerate()
        {
            c.text(7 + i as i32 * 34, 78, s, 18, MUTED);
        }
        let first = phone_core::calendar::Date { day: 1, ..date }.weekday();
        for d in 1..=date.month_days() {
            let cell = first + d as usize - 1;
            let x = cell % 7;
            let y = cell / 7;
            let px = x as i32 * 34;
            let py = 108 + y as i32 * 28;
            if d == date.day {
                c.rect(px, py, 33, 28, c.accent(3));
            }
            c.text(px + 6, py + 4, &d.to_string(), 18, WHITE);
            if p.local_data
                .events
                .iter()
                .any(|e| e.date.year == date.year && e.date.month == date.month && e.date.day == d)
            {
                c.rect(px + 14, py + 25, 4, 2, WHITE);
            }
        }
        c.text(8, 276, &date.iso(), 12, MUTED);
        left = Button::Menu;
    } else if matches!(page, DeleteContact | DeleteEvent | DeleteAlarm | DeletePast) {
        c.text(
            16,
            180,
            if page == DeletePast {
                "删除今天之前的项目？"
            } else {
                "删除此项目？"
            },
            18,
            WHITE,
        );
        center = "确认";
    } else {
        let items = p.organizer_items();
        list::plain(c, &items, p.index);
        left = match page {
            Contacts | Alarms | Events => Button::Text("新增"),
            EventForm | AlarmForm | ContactForm => Button::Text("保存"),
            ContactDetail => Button::Menu,
            _ => Button::None,
        };
    }
    Softkeys::new(left, Button::Text(center), Button::Back).draw(c);
}
