use super::*;
use crate::shell::media::Purpose;
pub(super) fn render(c: &mut Canvas, p: &Shell) {
    let m = &p.media.ui;
    chrome::title(
        c,
        match m.purpose {
            Purpose::Videos => "视频",
            Purpose::Pictures => "多媒体资料",
            Purpose::ContactRingtone
            | Purpose::Ringtone
            | Purpose::EventRingtone
            | Purpose::AlarmRingtone => "铃声",
            _ => "图片",
        },
    );
    if m.viewing {
        if let Some(frame) = &m.frame {
            let scale =
                (240. / frame.width() as f32).min(220. / frame.height() as f32) * m.zoom as f32;
            let w = (frame.width() as f32 * scale).round().max(1.) as u32;
            let h = (frame.height() as f32 * scale).round().max(1.) as u32;
            let pan_x = m.pan.0.clamp(
                -((w as i32 - 240) / 2).max(0),
                ((w as i32 - 240) / 2).max(0),
            );
            let pan_y = m.pan.1.clamp(
                -((h as i32 - 220) / 2).max(0),
                ((h as i32 - 220) / 2).max(0),
            );
            c.image_rect(
                frame,
                [
                    (240 - w as i32) / 2 + pan_x,
                    28 + (220 - h as i32) / 2 + pan_y,
                    w as i32,
                    h as i32,
                ],
                [0, 28, 240, 220],
            );
        } else {
            c.text(8, 130, "正在打开…", 20, WHITE);
        }
        if m.purpose == Purpose::Videos {
            c.fit(
                6,
                253,
                &format!(
                    "{} / {}",
                    phone_core::utilities::clock((m.position * 1000.) as u64),
                    phone_core::utilities::clock((m.duration * 1000.) as u64)
                ),
                16,
                WHITE,
                180,
            );
            c.text(195, 253, &format!("{}", m.volume), 16, MUTED);
        }
        Softkeys::back(if m.purpose.picker() {
            if m.frame.is_some() { "选择" } else { "" }
        } else if m.purpose == Purpose::Videos {
            if m.playing { "暂停" } else { "播放" }
        } else {
            "缩放"
        })
        .draw(c);
    } else {
        let labels: Vec<_> = m
            .entries
            .iter()
            .map(|e| {
                if e.directory {
                    format!("{} /", e.name)
                } else {
                    e.name.clone()
                }
            })
            .collect();
        list::plain(c, &labels, p.index);
        Softkeys::back(if labels.is_empty() { "" } else { "打开" }).draw(c);
    }
}
