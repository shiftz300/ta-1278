use super::*;
pub(super) fn render(c: &mut Canvas, p: &Shell) {
    if p.music_refresh_visible() {
        chrome::title(c, "音乐");
        chrome::waiting(c, "正在刷新您的音乐收藏...", p.music_wait_time());
        Softkeys::new(Button::None, Button::None, Button::Close).draw(c);
        return;
    }
    if p.playback.view != MusicView::Player {
        c.text(
            6,
            1,
            match p.playback.view {
                MusicView::Songs => "歌曲",
                MusicView::Playlists
                | MusicView::AddPlaylist
                | MusicView::Playlist
                | MusicView::DeletePlaylist => "播放列表",
                MusicView::PlaylistMenu => "选项",
                MusicView::Repeat => "重复",
                MusicView::Queue => "播放队列",
                _ => "音乐",
            },
            14,
            WHITE,
        );
        if matches!(
            p.playback.view,
            MusicView::Songs | MusicView::Queue | MusicView::Playlist
        ) {
            let order = match p.playback.view {
                MusicView::Queue => p.playback.model.order(),
                MusicView::Playlist => p.playlist_tracks(),
                _ => vec![],
            };
            let count = if p.playback.view == MusicView::Songs {
                p.playback.model.tracks.len()
            } else {
                order.len()
            };
            if count == 0 {
                c.text(8, 130, "这里没有可显示的内容。", 18, MUTED);
            }
            list::draw(c, count, p.index, list::ListStyle::STANDARD, |i| {
                list::Row::text(
                    &p.playback.model.tracks[if p.playback.view == MusicView::Songs {
                        i
                    } else {
                        order[i]
                    }]
                    .0,
                )
            });
            Softkeys::menu("选择").draw(c);
            return;
        }
        let labels = p.music_items();
        if labels.is_empty() {
            c.text(8, 130, "这里没有可显示的内容。", 18, MUTED);
        }
        let root = p.playback.view == MusicView::Root;
        if root {
            // Native library Home has an 84px first row, then standard rows.
            for (i, label) in labels.iter().take(6).enumerate() {
                let y = if i == 0 {
                    24
                } else {
                    108 + (i - 1) as i32 * 42
                };
                if i == p.index {
                    c.rect(0, y, 240, if i == 0 { 84 } else { 42 }, c.accent(3));
                }
                c.fit(
                    6,
                    if i == 0 { 54 } else { y + 10 },
                    label,
                    20,
                    WHITE,
                    if i == 0 { 196 } else { 228 },
                );
                if i == 0 {
                    for row in 0_i32..22 {
                        let width = 19 * (11 - (row - 10).abs()) / 11;
                        c.rect(214, 55 + row, width.max(1), 1, WHITE);
                    }
                }
            }
        } else {
            list::plain(c, &labels, p.index);
        }
        Softkeys::menu(if root && p.index == 0 {
            "播放"
        } else {
            "选择"
        })
        .draw(c);
        return;
    }

    chrome::title(c, "音乐");
    c.rect(0, 24, 240, 216, [66, 66, 66, 255]);
    c.ring(116, 143, 25, [28, 32, 28, 255]);
    c.rect(139, 59, 4, 85, [28, 32, 28, 255]);
    let duration = p.playback.model.duration();
    c.rect(0, 236, 240, 4, [38, 44, 38, 255]);
    c.rect(
        0,
        236,
        (240. * p.playback.model.position / duration.max(1.)).clamp(0., 240.) as i32,
        4,
        c.accent(1),
    );
    let track = p
        .playback
        .model
        .tracks
        .get(p.playback.model.index)
        .map_or("", |t| t.0.as_str());
    c.fit(44, 245, track, 18, WHITE, 151);
    c.text(
        1,
        247,
        &format!(
            "{}:{:02}",
            p.playback.model.position as u32 / 60,
            p.playback.model.position as u32 % 60
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
    softkeys::transport(c, p.playback.model.playing);
}
