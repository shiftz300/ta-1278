//! Shared popup declarations: labels, typed actions, markers and availability.
use super::*;
use crate::ui::menu::Item;
use browser::Action as B;
use music::Action as M;

#[derive(Clone, Copy)]
pub enum Action {
    Open(MenuKind),
    DialContacts,
    DialBack,
    SelectRecipient,
    NoGroups,
    Attachment,
    RemoveAttachment,
    ClearText,
    SaveDraft,
    Insert(&'static str),
    DiscardDraft,
    KeepEditing,
    Browser(B),
    Music(M),
    RefreshMusic,
}
const fn insert(value: &'static str) -> Item<'static, Action> {
    Item::new(Action::Insert(value), value)
}
impl MenuKind {
    pub fn items(self) -> &'static [Item<'static, Action>] {
        use Action::*;
        match self {
            Self::Dial => {
                const {
                    &[
                        Item::new(DialContacts, "通讯录"),
                        Item::new(DialBack, "返回"),
                    ]
                }
            }
            Self::Sms => {
                const {
                    &[
                        Item::new(Open(MenuKind::SmsInsert), "插入选项").child(),
                        Item::new(Attachment, "添加附件").child(),
                        Item::new(RemoveAttachment, "移除附件"),
                        Item::new(ClearText, "清除文字"),
                        Item::new(SaveDraft, "另存为草稿"),
                    ]
                }
            }
            Self::SmsRecipient => {
                const {
                    &[
                        Item::new(SelectRecipient, "通讯录"),
                        Item::new(NoGroups, "群组"),
                    ]
                }
            }
            Self::SmsInsert => {
                const {
                    &[
                        Item::new(Open(MenuKind::Smileys), "插入表情符号"),
                        Item::new(Open(MenuKind::Symbols), "插入符号"),
                        Item::new(Open(MenuKind::Templates), "插入模板"),
                    ]
                }
            }
            Self::Smileys => {
                const {
                    &[
                        insert(":)"),
                        insert(";)"),
                        insert(":D"),
                        insert(":P"),
                        insert(":*"),
                        insert(":o"),
                        insert(":|"),
                        insert(":/"),
                        insert("8)"),
                        insert(":("),
                        insert(">:("),
                        insert(">:o"),
                    ]
                }
            }
            Self::Symbols => {
                const {
                    &[
                        insert("."),
                        insert(","),
                        insert("'"),
                        insert("?"),
                        insert("!"),
                        insert("\""),
                        insert("-"),
                        insert("("),
                        insert(")"),
                        insert("@"),
                        insert("/"),
                        insert(":"),
                        insert("_"),
                        insert(";"),
                        insert("+"),
                        insert("&"),
                        insert("%"),
                        insert("*"),
                        insert("="),
                        insert("<"),
                        insert(">"),
                        insert("£"),
                        insert("€"),
                        insert("$"),
                        insert("¥"),
                        insert("₹"),
                        insert("¤"),
                        insert("["),
                        insert("]"),
                        insert("{"),
                        insert("}"),
                        insert("\\"),
                        insert("~"),
                        insert("^"),
                        insert("`"),
                        insert("¡"),
                        insert("¿"),
                        insert("§"),
                        insert("#"),
                        insert("|"),
                        insert(""),
                        insert("\n"),
                    ]
                }
            }
            Self::Templates => {
                const {
                    &[
                        insert("现在无法通话。我将稍后呼叫。"),
                        insert("我的电池电量低。请给我发送信息。"),
                        insert("请给我办公室拨电话。"),
                        insert("我在开会，晚些给你打电话。"),
                        insert("我要迟到了！到达时间为"),
                        insert("预计到达时间为"),
                        insert("老地方见。"),
                    ]
                }
            }
            Self::ExitSms => {
                const {
                    &[
                        Item::new(SaveDraft, "保存并关闭"),
                        Item::new(DiscardDraft, "放弃修改"),
                        Item::new(KeepEditing, "继续编辑"),
                    ]
                }
            }
            Self::Browser => {
                const {
                    &[
                        Item::new(Open(MenuKind::BrowserOpen), "打开"),
                        Item::new(Open(MenuKind::BrowserNavigation), "导航"),
                        Item::new(Open(MenuKind::BrowserSave), "保存"),
                        Item::new(Browser(B::Downloads), "下载管理"),
                        Item::new(Browser(B::Share), "分享链接"),
                        Item::new(Browser(B::Details), "页面详情"),
                        Item::new(Browser(B::Settings), "设置"),
                        Item::new(Browser(B::ClosePage), "关闭"),
                    ]
                }
            }
            Self::BrowserRoot => {
                const {
                    &[
                        Item::new(Browser(B::Home), "主页"),
                        Item::new(Browser(B::Settings), "设置"),
                        Item::new(Browser(B::Close), "关闭"),
                    ]
                }
            }
            Self::BrowserOpen => {
                const {
                    &[
                        Item::new(Browser(B::Home), "主页"),
                        Item::new(Browser(B::Address), "输入URL"),
                        Item::new(Browser(B::List(BrowserView::Bookmarks)), "书签"),
                        Item::new(Browser(B::List(BrowserView::History)), "历史"),
                        Item::new(Browser(B::List(BrowserView::Offline)), "离线页面"),
                    ]
                }
            }
            Self::BrowserNavigation => {
                const {
                    &[
                        Item::new(Browser(B::Refresh), "刷新"),
                        Item::new(Browser(B::Stop), "停止"),
                        Item::new(Browser(B::Forward), "前进"),
                        Item::new(Browser(B::Back), "返回"),
                    ]
                }
            }
            Self::BrowserSave => {
                const {
                    &[
                        Item::new(Browser(B::SetHome), "作为主页"),
                        Item::new(Browser(B::Bookmark), "书签"),
                        Item::new(Browser(B::SavePage), "网页"),
                        Item::new(Browser(B::SaveImage), "图像"),
                    ]
                }
            }
            Self::Music => {
                const {
                    &[
                        Item::new(Music(M::Repeat), "重复"),
                        Item::new(Music(M::Shuffle), "随机播放"),
                        Item::new(Music(M::Queue), "播放队列"),
                        Item::new(Music(M::Playlist), "添加到播放列表"),
                        Item::new(Music(M::Ringtone), "设为铃声"),
                        Item::new(Music(M::Share), "分享"),
                    ]
                }
            }
            Self::MusicLibrary => const { &[Item::new(RefreshMusic, "手动更新")] },
        }
    }
    pub fn columns(self) -> usize {
        match self {
            Self::Symbols => 7,
            Self::Smileys => 4,
            _ => 1,
        }
    }
}
impl Shell {
    pub fn menu_item(&self, kind: MenuKind, index: usize) -> Item<'static, Action> {
        let item = &kind.items()[index];
        let enabled = match item.action {
            Action::RemoveAttachment | Action::Insert("") => false,
            Action::ClearText | Action::SaveDraft if kind == MenuKind::Sms => {
                !self.messaging.compose.body.units.is_empty()
            }
            Action::Browser(B::SaveImage) => false,
            Action::Browser(B::SetHome | B::Bookmark | B::SavePage) => {
                !self.browsing.model.page.url.is_empty()
            }
            Action::Browser(B::Stop) => self.browsing.model.loading,
            Action::Browser(B::Forward) => self.browsing.model.can_forward(),
            Action::Browser(B::Back) => self.browsing.model.can_back(),
            _ => true,
        };
        Item {
            action: item.action,
            label: item.label,
            enabled,
            marker: item.marker,
        }
    }
    #[cfg(test)]
    pub fn menu_labels(&self, kind: MenuKind) -> Vec<&'static str> {
        kind.items().iter().map(|item| item.label).collect()
    }
    #[cfg(test)]
    pub fn menu_enabled(&self, kind: MenuKind, index: usize) -> bool {
        self.menu_item(kind, index).enabled
    }
}
