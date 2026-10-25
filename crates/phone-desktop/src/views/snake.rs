use super::*;
use crate::{phone::snake::SnakeView, snake_art::Sprite};
use phone_core::snake::{Direction, Snake};
fn game(c: &mut Canvas, s: &Snake) {
    c.sprite(Sprite::Background, 0, 0, [0, 0, 240, 320]);
    for y in 0..Snake::H {
        for x in 0..Snake::W {
            if s.wall(x, y) {
                c.sprite(Sprite::Wall, x * 20, y * 20, [0, 0, 20, 20]);
            }
        }
    }
    c.sprite(Sprite::Fruit, s.food.0 * 20, s.food.1 * 20, [0, 0, 20, 20]);
    if let Some(b) = s.bonus {
        c.sprite(
            Sprite::Bonus,
            b.position.0 * 20,
            b.position.1 * 20,
            [0, 0, 20, 20],
        );
        c.sprite(
            Sprite::Countdown,
            135,
            13,
            [b.countdown as u32 * 9, 0, 9, 12],
        );
    }
    for (i, &(x, y)) in s.body.iter().enumerate() {
        let d = s.segment_direction(i);
        if i == 0 {
            let (sx, dx, dy) = match d {
                Direction::Right => (0, 0, -3),
                Direction::Down => (26, -3, -3),
                Direction::Left => (52, -6, -3),
                Direction::Up => (79, -2, -4),
            };
            c.sprite(Sprite::Head, x * 20 + dx, y * 20 + dy, [sx, 0, 26, 26]);
        } else {
            let frame = match d {
                Direction::Right => 0,
                Direction::Down => 1,
                Direction::Left => 2,
                Direction::Up => 3,
            };
            c.sprite(Sprite::Body, x * 20, y * 20, [frame * 20, 0, 20, 20]);
        }
    }
    for (i, d) in s.score.to_string().bytes().rev().enumerate() {
        c.sprite(
            Sprite::Score,
            105 - i as i32 * 16,
            3,
            [(d - b'0') as u32 * 16, 0, 16, 23],
        );
    }
}
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    if p.snake_view == SnakeView::Game {
        game(c, &p.snake);
        return;
    }
    c.text(
        6,
        1,
        match p.snake_view {
            SnakeView::Level => "关卡",
            SnakeView::Types => "游戏类型",
            SnakeView::Scores => "最高得分",
            SnakeView::Sound => "声音",
            _ => "贪吃蛇",
        },
        14,
        WHITE,
    );
    match p.snake_view {
        SnakeView::Level => {
            for i in 0..8 {
                c.rect(
                    24 + i * 24,
                    160,
                    21,
                    41,
                    crate::theme::rgb565(if i < p.snake_level_preview as i32 {
                        0x07e0
                    } else {
                        0xad55
                    }),
                );
            }
        }
        SnakeView::Scores => {
            for (i, score) in p.snake_data.scores[p.snake.kind.min(5)].iter().enumerate() {
                c.text(
                    10,
                    30 + i as i32 * 24,
                    &format!("{}.{}", i + 1, score),
                    16,
                    WHITE,
                );
            }
        }
        _ => {
            super::tools::native_list(c, &p.snake_items(), p.index);
            if p.snake_view == SnakeView::Sound {
                for i in 0..2 {
                    c.ring(225, 45 + i * 42, 7, c.accent(2));
                    if i == i32::from(p.snake_data.sound) {
                        c.rect(222, 42 + i * 42, 6, 6, c.accent(2));
                    }
                }
            }
        }
    }
    footer(c, "", "←");
    if p.snake_view != SnakeView::Scores {
        c.text(98, 296, "选择", 20, WHITE);
    }
}
