use super::*;
use crate::phone::tools::ToolPage;
use phone_core::utilities::clock;
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    use ToolPage::*;
    if p.tools.page == Files {
        super::files::render(c, p);
        return;
    }
    c.text(6, 1, p.tools_title(), 14, WHITE);
    let t = &p.tools;
    let mut left = "";
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
            left = "←";
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
            left = if t.stopwatch.running {
                "计次"
            } else {
                "重置"
            };
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
            left = "重置";
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
            left = "←";
            center = "";
        }
        Flashlight => {
            c.icon(13, 82, 60, 76);
            c.text(90, 165, if t.torch { "开启" } else { "关闭" }, 24, WHITE);
            center = if t.torch { "关闭" } else { "开启" };
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
            left = "≡";
            center = if t.radio_on { "停止" } else { "开始" };
        }
        Recorder => {
            c.icon(14, 82, 58, 76);
            c.text(20, 166, &clock(t.record_ms), 32, WHITE);
            left = "列表";
            center = if t.recording { "停止" } else { "录音" };
        }
        Counter | Video | Gallery | Calls => {
            c.text(8, 120, "这里没有可显示的内容。", 17, WHITE);
            center = "";
        }
        _ => {
            let labels = p.tools_items();
            native_list(c, &labels, p.index);
            if t.page == Timer {
                left = "开始";
            }
        }
    }
    footer(
        c,
        left,
        if t.page == Calculator {
            "清除"
        } else {
            "←"
        },
    );
    if !center.is_empty() {
        c.text(98, 296, center, 20, WHITE);
    }
}
pub(super) fn native_list(c: &mut Canvas, labels: &[String], index: usize) {
    if labels.is_empty() {
        c.text(8, 120, "这里没有可显示的内容。", 17, WHITE);
    }
    let start = index.saturating_sub(5);
    for (i, label) in labels.iter().enumerate().skip(start).take(6) {
        let y = 24 + (i - start) as i32 * 42;
        if i == index {
            c.rect(0, y, 240, 42, c.accent(3));
        }
        c.fit(6, y + 10, label, 20, WHITE, 228);
    }
}
