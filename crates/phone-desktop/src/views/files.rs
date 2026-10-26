use super::*;
use crate::phone::files::FileView;
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    let title = match p.files.view {
        FileView::Options => "选项",
        FileView::Delete => "删除",
        FileView::Details => "详情",
        FileView::Browse => p
            .files
            .path
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("文件"),
    };
    let title = match title {
        "phone" => "手机存储",
        "card" => "存储卡",
        _ => title,
    };
    c.fit(6, 1, title, 14, WHITE, 228);
    match p.files.view {
        FileView::Delete | FileView::Details => {
            if let Some(e) = p.files.entries.get(p.files.selected) {
                c.fit(8, 64, &e.name, 20, WHITE, 224);
                if p.files.view == FileView::Delete {
                    c.text(8, 120, "确认删除？", 20, WHITE);
                    Softkeys::new(Button::Text("删除"), Button::None, Button::Text("取消")).draw(c);
                } else {
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
        }
        _ => {
            let labels = p.file_labels();
            if p.files.view == FileView::Options {
                list::draw(c, labels.len(), p.index, list::ListStyle::STANDARD, |i| {
                    list::Row {
                        label: &labels[i],
                        enabled: p.file_action_enabled(i),
                        marker: list::Marker::None,
                    }
                });
            } else {
                list::plain(c, &labels, p.index);
            }
            if p.files.busy {
                c.rect(0, 120, 240, 42, [0, 0, 0, 255]);
                c.text(8, 130, "正在复制文件…", 20, WHITE);
            }
            Softkeys::menu("选择").draw(c);
        }
    }
}
