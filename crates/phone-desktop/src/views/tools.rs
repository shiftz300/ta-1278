use super::*;
use crate::phone::tools::ToolPage;
use phone_core::utilities::clock;
pub(super) fn render(c: &mut Canvas, p: &Phone) {
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
            c.fit(8, 50, &t.calculator.expression, 16, MUTED, 224);
            c.fit(8, 86, t.calculator.display(), 32, WHITE, 224);
            for (x, y, label) in [
                (110, 157, "+"),
                (70, 197, "÷"),
                (110, 197, "="),
                (150, 197, "×"),
                (110, 237, "−"),
            ] {
                c.text(x, y, label, 24, WHITE);
            }
            c.text(8, 270, "* ±", 16, MUTED);
            c.text(185, 270, "# .", 16, MUTED);
            left = Button::Back;
            center = "";
        }
        Stopwatch => {
            c.text(20, 65, &clock(t.stopwatch.elapsed_ms), 32, WHITE);
            c.text(
                180,
                107,
                &format!(".{:02}", t.stopwatch.elapsed_ms / 10 % 100),
                18,
                MUTED,
            );
            for (i, lap) in t.stopwatch.laps.iter().take(5).enumerate() {
                c.text(
                    16,
                    140 + i as i32 * 27,
                    &format!(
                        "{}   {}.{:02}",
                        t.stopwatch.laps.len() - i,
                        clock(*lap),
                        lap / 10 % 100
                    ),
                    18,
                    WHITE,
                );
            }
            left = Button::Text(if t.stopwatch.running {
                "计次"
            } else {
                "重置"
            });
            center = if t.stopwatch.running {
                "停止"
            } else if t.stopwatch.elapsed_ms > 0 {
                "继续"
            } else {
                "开始"
            };
        }
        TimerRunning => {
            c.fit(10, 62, &t.timer_name, 20, WHITE, 220);
            c.text(
                20,
                124,
                &clock(t.timer.remaining_ms.saturating_add(999)),
                32,
                WHITE,
            );
            left = Button::Text("重置");
            center = if t.timer.running { "暂停" } else { "继续" };
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
            c.text(
                35,
                100,
                &format!("{:.1}", t.frequency as f32 / 10.),
                40,
                WHITE,
            );
            c.text(170, 119, "MHz", 16, MUTED);
            c.rect(14, 190, 212, 1, WHITE);
            for i in 0..22 {
                c.rect(15 + i * 10, 185, 1, if i % 5 == 0 { 16 } else { 9 }, WHITE);
            }
            c.rect(
                15 + (t.frequency as i32 - 875) * 210 / 205,
                180,
                2,
                26,
                c.accent(2),
            );
            c.text(8, 238, "87.5", 14, MUTED);
            c.text(185, 238, "108.0", 14, MUTED);
            left = Button::Menu;
            center = if t.radio_on { "停止" } else { "开始" };
        }
        Recorder => {
            c.icon(14, 82, 58, 76);
            c.text(20, 166, &clock(t.record_ms), 32, WHITE);
            left = Button::Text("列表");
            center = if t.recording { "停止" } else { "录音" };
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
