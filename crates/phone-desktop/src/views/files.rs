use super::*;
use crate::phone::files::FileView;
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    if matches!(
        p.files.view,
        FileView::DeleteSelect | FileView::DeleteOptions | FileView::DeleteConfirm
    ) {
        deletion(c, p);
        return;
    }
    if matches!(
        p.files.view,
        FileView::Storage | FileView::StorageOptions | FileView::StorageDetails
    ) {
        storage(c, p);
        return;
    }
    let title = match p.files.view {
        FileView::Options => p
            .files
            .path
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("手机存储"),
        FileView::DeleteSelect | FileView::DeleteOptions | FileView::DeleteConfirm => {
            unreachable!()
        }
        FileView::Details => "详情",
        FileView::Storage | FileView::StorageOptions | FileView::StorageDetails => unreachable!(),
        FileView::Browse => p
            .files
            .path
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("手机存储"),
    };
    let title = if let Some(t) = &p.files.transfer {
        if t.moving { "移动到" } else { "复制到" }
    } else {
        title
    };
    c.fit(6, 1, title, 14, WHITE, 228);
    match p.files.view {
        FileView::Details => {
            if let Some(e) = p.files.entries.get(p.files.selected) {
                c.fit(8, 64, &e.name, 20, WHITE, 224);

                c.text(
                    8,
                    120,
                    if e.directory { "文件夹" } else { "文件" },
                    18,
                    MUTED,
                );
                if !e.directory {
                    c.fit(8, 158, &format!("{} 字节", e.len), 18, WHITE, 224);
                }
                Softkeys::back("").draw(c);
            }
        }
        _ => {
            browse(
                c,
                p,
                if p.files.view == FileView::Options {
                    p.files.selected
                } else {
                    p.index
                },
            );
            if p.files.view == FileView::Options {
                c.dim();
                let labels = p.file_labels();
                list::popup(c, labels.len(), p.index, |i| list::Row {
                    label: &labels[i],
                    enabled: p.file_action_enabled(i),
                    marker: list::Marker::None,
                });
                Softkeys::back("选择").draw(c);
            } else if p.files.transfer.is_some() {
                Softkeys::new(
                    Button::Transfer,
                    Button::Text(if p.files.entries.is_empty() {
                        ""
                    } else {
                        "打开"
                    }),
                    Button::Back,
                )
                .draw(c);
            } else {
                Softkeys::menu(if p.files.entries.is_empty() {
                    ""
                } else {
                    "打开"
                })
                .draw(c);
            }
            if p.files.busy {
                c.rect(0, 120, 240, 42, [0, 0, 0, 255]);
                c.text(8, 130, "正在复制文件…", 20, WHITE);
            }
        }
    }
}

fn deletion(c: &mut Canvas, p: &Phone) {
    chrome::title(c, "删除");
    let entries = p.delete_entries();
    let focus = if p.files.view == FileView::DeleteSelect {
        p.index
    } else {
        p.files.delete_index
    };
    list::draw(
        c,
        entries.len(),
        focus,
        list::ListStyle {
            inset: 72,
            text_y: 10,
            font: 16,
            text_width: 160,
            ..list::ListStyle::STANDARD
        },
        |i| list::Row {
            label: &entries[i].name,
            enabled: true,
            marker: list::Marker::Check(p.files.marked.contains(&entries[i].path)),
        },
    );
    for (i, e) in entries
        .iter()
        .enumerate()
        .skip(focus.saturating_sub(5))
        .take(6)
    {
        crate::ui::art::utility(
            c,
            if e.directory {
                crate::ui::art::UtilityIcon::Folder
            } else {
                crate::ui::art::UtilityIcon::File
            },
            42,
            33 + (i - focus.saturating_sub(5)) as i32 * 42,
        );
    }
    match p.files.view {
        FileView::DeleteOptions => {
            c.dim();
            let all =
                !entries.is_empty() && entries.iter().all(|e| p.files.marked.contains(&e.path));
            list::popup(c, 2, p.index, |i| list::Row {
                label: if i == 0 {
                    "删除"
                } else if all {
                    "取消全选"
                } else {
                    "全选"
                },
                enabled: i != 0 || !p.files.marked.is_empty(),
                marker: list::Marker::None,
            });
            Softkeys::back("选择").draw(c);
        }
        FileView::DeleteConfirm => {
            c.dim();
            c.rect(0, 184, 240, 100, [0, 0, 0, 255]);
            c.fit(
                6,
                205,
                &format!("删除选定的 {} 项？", p.files.marked.len()),
                20,
                WHITE,
                228,
            );
            c.text(6, 240, "非空文件夹将保留。", 16, MUTED);
            Softkeys::new(Button::None, Button::Text("确认"), Button::Close).draw(c);
        }
        _ => Softkeys::new(
            Button::Menu,
            Button::Text(
                if entries
                    .get(focus)
                    .is_some_and(|e| p.files.marked.contains(&e.path))
                {
                    "取消选择"
                } else {
                    "选择"
                },
            ),
            Button::Back,
        )
        .draw(c),
    }
    if p.files.busy {
        c.rect(0, 120, 240, 42, [0, 0, 0, 255]);
        c.text(8, 130, "正在删除…", 20, WHITE);
    }
}

fn size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}GB", bytes as f64 / (1024. * 1024. * 1024.))
    } else {
        format!("{:.1}MB", bytes as f64 / (1024. * 1024.))
    }
}
fn storage(c: &mut Canvas, p: &Phone) {
    if p.files.view == FileView::StorageDetails {
        chrome::title(c, "详情");
        let values = if let Some((total, free)) = p.files.capacity {
            [
                "手机存储".into(),
                size(total),
                size(free),
                size(total - free),
            ]
        } else {
            ["手机存储".into(), "—".into(), "—".into(), "—".into()]
        };
        for (i, (label, value)) in ["名称", "大小", "可用存储", "已使用的存储"]
            .iter()
            .zip(values)
            .enumerate()
        {
            let y = 33 + i as i32 * 64;
            c.text(10, y, label, 16, MUTED);
            c.text(10, y + 31, &value, 18, WHITE);
        }
        Softkeys::back("").draw(c);
        return;
    }
    chrome::title(
        c,
        if let Some(t) = &p.files.transfer {
            if t.moving { "移动到" } else { "复制到" }
        } else {
            "文件"
        },
    );
    c.rect(0, 24, 240, 64, c.accent(3));
    crate::ui::art::storage(c, 6, 34);
    c.text(36, 34, "手机存储", 20, WHITE);
    if let Some((total, free)) = p.files.capacity {
        c.text(
            36,
            63,
            &format!("{}/{}", size(total - free), size(total)),
            14,
            WHITE,
        );
    }
    if p.files.view == FileView::StorageOptions {
        c.dim();
        list::popup(c, 2, p.index, |i| list::Row {
            label: ["详情", "格式化"][i],
            enabled: i == 0,
            marker: list::Marker::None,
        });
        Softkeys::back(if p.index == 0 { "选择" } else { "" }).draw(c);
    } else if p.files.transfer.is_some() {
        Softkeys::new(Button::Transfer, Button::Text("打开"), Button::Back).draw(c);
    } else {
        Softkeys::menu("打开").draw(c);
    }
}

fn browse(c: &mut Canvas, p: &Phone, index: usize) {
    let labels: Vec<_> = p.files.entries.iter().map(|e| e.name.as_str()).collect();
    if labels.is_empty() {
        c.text(8, 120, "这里没有可显示的内容。", 17, WHITE);
    }
    let style = list::ListStyle {
        inset: 36,
        font: 18,
        text_width: 198,
        ..list::ListStyle::STANDARD
    };
    list::draw(c, labels.len(), index, style, |i| {
        list::Row::text(labels[i])
    });
    let start = index.saturating_sub(5);
    for (i, e) in p.files.entries.iter().enumerate().skip(start).take(6) {
        let y = 33 + (i - start) as i32 * 42;
        use crate::ui::art::{UtilityIcon, utility};
        utility(
            c,
            if e.directory {
                UtilityIcon::Folder
            } else {
                UtilityIcon::File
            },
            6,
            y,
        );
    }
}
