use super::*;
use crate::phone::snake::SnakeView;
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    c.text(6, 1, "贪吃蛇", 14, WHITE);
    match p.snake_view {
        SnakeView::Game => {
            let s = &p.snake;
            c.text(8, 27, &format!("得分 {}", s.score), 18, WHITE);
            c.rect(0, 52, 240, 240, [18, 30, 18, 255]);
            for y in 0..24 {
                for x in 0..24 {
                    if s.wall(x, y) {
                        c.rect(x * 10, 52 + y * 10, 10, 10, MUTED);
                    }
                }
            }
            for (i, &(x, y)) in s.body.iter().enumerate() {
                c.rect(
                    x * 10 + 1,
                    53 + y * 10,
                    8,
                    8,
                    if i == 0 { WHITE } else { c.accent(1) },
                );
            }
            c.rect(
                s.food.0 * 10 + 2,
                54 + s.food.1 * 10,
                6,
                6,
                [239, 65, 65, 255],
            );
            footer(c, "", "暂停");
        }
        SnakeView::Level => {
            c.text(20, 95, "等级", 20, WHITE);
            for i in 0..10 {
                c.rect(
                    20 + i * 20,
                    160,
                    16,
                    32,
                    if i <= p.snake.level as i32 {
                        c.accent(2)
                    } else {
                        MUTED
                    },
                );
            }
            footer(c, "", "←");
            c.text(98, 296, "选择", 20, WHITE);
        }
        _ => {
            super::tools::native_list(c, &p.snake_items(), p.index);
            footer(c, "", "←");
            c.text(98, 296, "选择", 20, WHITE);
        }
    }
}
