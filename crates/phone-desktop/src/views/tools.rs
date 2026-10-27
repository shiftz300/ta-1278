use super::*;
use crate::phone::tools::ToolPage;
use phone_core::utilities::clock;
pub(super) fn render(c: &mut Canvas, p: &mut Phone) {
    use ToolPage::*;
    if p.tools.page == Files {
        super::files::render(c, p);
        return;
    }
    chrome::title(c, p.tools_title());
    let t = &p.tools;
    let mut left = Button::None;
    let mut center = "选择";
    match t.page {
        Calculator => {
            let calc = &t.calculator;
            let expression = calc.expression.replace(' ', "").replace('×', "x");
            right(c, 234, 86, &expression, 28, MUTED);
            right(c, 234, 133, calc.display(), 48, WHITE);
            c.rect(0, 206, 240, 114, c.accent(3));
            c.rect(7, 212, 56, 28, c.accent(4));
            c.rect(176, 212, 56, 28, c.accent(4));
            use crate::ui::art::{UtilityIcon, utility};
            c.text(13, 214, "* ±", 20, WHITE);
            c.text(182, 214, "# .", 20, WHITE);
            Softkeys::new(
                Button::Percent,
                Button::None,
                if calc.entry.is_empty() && calc.expression.is_empty() {
                    Button::Back
                } else {
                    Button::Delete
                },
            )
            .accent_panel()
            .draw(c);
            // The original pad continues into the footer. Draw its backdrop and
            // glyphs after the shared bar, rather than clipping its lower end.
            rounded(c, [69, 212, 102, 102], 12, c.accent(4));
            for (icon, x, y) in [
                (UtilityIcon::Plus, 106, 212),
                (UtilityIcon::Minus, 106, 287),
                (UtilityIcon::Divide, 70, 249),
                (UtilityIcon::Equals, 106, 249),
                (UtilityIcon::Multiply, 143, 249),
            ] {
                utility(c, icon, x, y);
            }
            return;
        }
        Stopwatch => {
            let time = stopwatch_time(t.stopwatch.elapsed_ms);
            c.fit(7, 33, &time, 44, c.accent(3), 228);
            for (i, lap) in t.stopwatch.laps.iter().take(6).enumerate() {
                let y = 98 + i as i32 * 29;
                c.text(
                    6,
                    y,
                    &format!("{:02}", t.stopwatch.laps.len() - i),
                    16,
                    WHITE,
                );
                right(c, 234, y, &stopwatch_time(*lap), 16, WHITE);
            }
            left = if t.stopwatch.running {
                Button::Lap
            } else if t.stopwatch.elapsed_ms > 0 {
                Button::Reset
            } else {
                Button::None
            };
            center = if t.stopwatch.running {
                "停止"
            } else if t.stopwatch.elapsed_ms > 0 {
                "恢复"
            } else {
                "开始"
            };
        }
        Timer => {
            let edit = &t.timer_edit;
            let time = clock(edit.seconds * 1000);
            c.text(
                7,
                31,
                "设置时间",
                16,
                if p.index == 0 { c.accent(3) } else { MUTED },
            );
            c.text(7, 59, &time, 16, WHITE);
            if p.index == 0 {
                let at = [0, 1, 3, 4, 6, 7][edit.digit];
                let x = 7 + c.width(&time[..at], 16);
                let w = c.width(&time[at..at + 1], 16);
                c.rect(x, 59, w, 26, WHITE);
                c.text(x, 59, &time[at..at + 1], 16, [0, 0, 0, 255]);
            }
            c.rect(
                7,
                84,
                228,
                3,
                if p.index == 0 { c.accent(3) } else { MUTED },
            );
            c.text(
                7,
                103,
                "定时器名称",
                16,
                if p.index == 1 { c.accent(3) } else { MUTED },
            );
            if p.index == 1 {
                p.editor_lines = Edit::new([7, 125, 227, 27])
                    .sized(20, 26)
                    .draw(c, &edit.name);
                c.text(203, 103, mode(&edit.name), 16, MUTED);
            } else {
                c.text(7, 125, &edit.name.text(), 20, WHITE);
            }
            if edit.name.units.is_empty() && !edit.name.prediction.active() {
                c.text(7, 125, "添加名称", 20, MUTED);
            }
            c.rect(
                7,
                149,
                228,
                3,
                if p.index == 1 { c.accent(3) } else { MUTED },
            );
            Softkeys::new(
                Button::None,
                if edit.seconds > 0
                    && (!edit.name.units.is_empty() || edit.name.prediction.active())
                {
                    Button::Text("开始")
                } else {
                    Button::Disabled("开始")
                },
                Button::Back,
            )
            .draw(c);
            return;
        }
        Other | TimerMenu => {
            let remaining = if t.timer.running {
                clock(t.timer.remaining_ms.saturating_add(999))
            } else {
                "关".into()
            };
            let rows = [
                ("定时器", remaining.as_str()),
                ("秒表", if t.stopwatch.running { "开" } else { "关" }),
                ("手电筒", if t.torch { "开" } else { "关" }),
                ("单位换算", ""),
            ];
            chrome::title(c, "其他");
            list::detail_rows(
                c,
                rows.len(),
                if t.page == TimerMenu { 0 } else { p.index },
                |i| rows[i],
            );
            if t.page == TimerMenu {
                c.dim();
                let style = list::ListStyle {
                    top: 103,
                    ..list::ListStyle::popup(2)
                };
                c.rect(0, 103, 240, 76, [0, 0, 0, 255]);
                list::draw(c, 2, p.index, style, |i| {
                    list::Row::text(["编辑定时器", "停止计时"][i])
                });
                Softkeys::back("选择").draw(c);
                return;
            }
            center = "打开";
        }
        Conversion(_) => {
            let (a, b, value, result) = p.conversion();
            for (i, (unit, number)) in [(a, value), (b, result)].iter().enumerate() {
                let y = 40 + i as i32 * 108;
                if usize::from(t.conversion_reverse) == i {
                    c.rect(0, y, 240, 84, c.accent(3));
                }
                c.text(8, y + 8, unit, 18, WHITE);
                c.fit(8, y + 38, number, 24, WHITE, 224);
            }
            left = Button::Back;
            center = "";
        }
        Flashlight => {
            crate::ui::art::power(c, t.torch);
            Softkeys::new(
                Button::Phone,
                Button::Text(if t.torch { "关闭" } else { "打开" }),
                Button::Back,
            )
            .accent_panel()
            .draw(c);
            return;
        }
        Radio => {
            c.rect(0, 24, 240, 90, c.accent(3));
            let frequency = format!("{:.1}", t.frequency as f32 / 10.);
            let frequency_x = (240 - c.width(&frequency, 18)) / 2;
            c.text(frequency_x, 38, &frequency, 18, WHITE);
            c.text(107, 65, "MHz", 12, WHITE);
            c.text(3, 80, "87.5", 12, MUTED);
            right(c, 237, 80, "108.0", 12, MUTED);
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
            rounded(c, [46, 132, 148, 148], 29, [0, 0, 0, 255]);
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
            return;
        }
        Recorder => {
            let time = clock(t.record_ms);
            let time_x = (240 - c.width(&time, 22)) / 2;
            c.text(time_x, 32, &time, 22, c.accent(3));
            for radius in (10..=100).step_by(10) {
                c.ring(120, 165, radius, c.accent(6));
            }
            Softkeys::new(
                Button::Recordings,
                if t.recording {
                    Button::Text("停止")
                } else {
                    Button::Record
                },
                Button::Back,
            )
            .draw(c);
            return;
        }
        Counter | Video | Gallery | Calls => {
            c.text(8, 120, "这里没有可显示的内容。", 17, WHITE);
            center = "";
        }
        _ => {
            let labels = p.tools_items();
            list::plain(c, &labels, p.index);
            if t.page == Timer {
                left = Button::Text("开始");
            }
        }
    }
    Softkeys::new(
        left,
        Button::Text(center),
        if t.page == Calculator {
            Button::Text("清除")
        } else {
            Button::Back
        },
    )
    .draw(c);
}

fn right(c: &mut Canvas, edge: i32, y: i32, text: &str, size: u16, color: Color) {
    let width = c.width(text, size);
    c.fit((edge - width).max(6), y, text, size, color, edge - 6);
}
fn stopwatch_time(ms: u64) -> String {
    format!(
        "{}:{:02}:{:02}.{:02}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms / 10 % 100
    )
}
fn rounded(c: &mut Canvas, [x, y, w, h]: [i32; 4], radius: i32, color: Color) {
    for dy in 0..h {
        for dx in 0..w {
            let cx = dx.clamp(radius, w - radius - 1);
            let cy = dy.clamp(radius, h - radius - 1);
            if (dx - cx).pow(2) + (dy - cy).pow(2) <= radius.pow(2) {
                c.rect(x + dx, y + dy, 1, 1, color);
            }
        }
    }
}

pub(super) fn timer_elapsed(c: &mut Canvas, p: &Phone) {
    c.rect(0, 96, 240, 190, [0, 0, 0, 255]);
    crate::ui::art::hourglass(c, 94, 132);
    let name = &p.tools.timer_name;
    let x = (240 - c.width(name, 16)) / 2;
    c.text(x, 198, name, 16, WHITE);
    let duration = clock(p.tools.timer_seconds * 1000);
    let x = (240 - c.width(&duration, 14)) / 2;
    c.text(x, 229, &duration, 14, WHITE);
    Softkeys::new(Button::None, Button::Text("重计时"), Button::Text("停止")).draw(c);
}
