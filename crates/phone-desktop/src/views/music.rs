use super::*;
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    if p.music_view != MusicView::Player {
        c.text(
            6,
            1,
            match p.music_view {
                MusicView::Songs => "歌曲",
                MusicView::Playlists => "播放列表",
                MusicView::Repeat => "重复",
                MusicView::Queue => "播放队列",
                _ => "音乐",
            },
            14,
            WHITE,
        );
        let labels = p.music_items();
        if labels.is_empty() {
            c.text(12, 120, "这里没有可显示的内容。", 17, WHITE);
        }
        let start = p.index.saturating_sub(5);
        for (i, label) in labels.iter().enumerate().skip(start).take(6) {
            let y = 24 + (i - start) as i32 * 42;
            if i == p.index {
                c.rect(0, y, 240, 42, c.accent(3));
            }
            c.fit(6, y + 10, label, 20, WHITE, 228);
        }
        footer(c, "≡", "←");
        c.text(98, 296, "选择", 20, WHITE);
        return;
    }

    c.text(6, 1, "音乐", 14, WHITE);
    c.rect(0, 24, 240, 216, [66, 66, 66, 255]);
    c.ring(116, 143, 25, [28, 32, 28, 255]);
    c.rect(139, 59, 4, 85, [28, 32, 28, 255]);
    let duration = p.music.duration();
    c.rect(0, 236, 240, 4, [38, 44, 38, 255]);
    c.rect(
        0,
        236,
        (240. * p.music.position / duration.max(1.)).clamp(0., 240.) as i32,
        4,
        c.accent(1),
    );
    let track = p
        .music
        .tracks
        .get(p.music.index)
        .map_or("", |t| t.0.as_str());
    c.fit(44, 245, track, 18, WHITE, 151);
    c.text(
        1,
        247,
        &format!(
            "{}:{:02}",
            p.music.position as u32 / 60,
            p.music.position as u32 % 60
        ),
        12,
        c.accent(1),
    );
    c.text(
        204,
        247,
        &format!("{}:{:02}", duration as u32 / 60, duration as u32 % 60),
        12,
        c.accent(1),
    );
    footer(c, "≡", "←");
    c.text(76, 290, "◀", 20, WHITE);
    c.rect(77, 296, 2, 14, WHITE);
    c.text(160, 290, "▶", 20, WHITE);
    c.rect(178, 296, 2, 14, WHITE);
    if p.music.playing {
        c.rect(113, 296, 5, 14, WHITE);
        c.rect(122, 296, 5, 14, WHITE);
    } else {
        c.text(112, 290, "▶", 20, WHITE);
    }
}
