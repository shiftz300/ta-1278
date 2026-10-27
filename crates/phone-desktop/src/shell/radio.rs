//! FM option declarations and effects, separate from shared menu interaction.
use super::*;
use crate::ui::{
    list::Marker,
    menu::{self, Event, Item},
};
use tools::ToolPage;
#[derive(Clone, Copy)]
pub enum RadioAction {
    Favorites,
    AddFavorite,
    Speaker,
    Search,
    FineTune,
    Frequency,
    Close,
}
impl Shell {
    pub fn radio_menu(&self) -> [Item<'static, RadioAction>; 7] {
        use RadioAction::*;
        [
            Item::new(Favorites, "收藏夹"),
            Item::new(AddFavorite, "添加至收藏夹"),
            Item {
                enabled: false,
                marker: Marker::Switch(self.tools.radio_speaker),
                ..Item::new(Speaker, "扬声器")
            },
            Item::new(Search, "自动搜索"),
            Item::new(FineTune, "微调"),
            Item::new(Frequency, "设定频率"),
            Item::new(Close, "关闭收音机"),
        ]
    }
    pub(super) fn radio_options_key(&mut self, key: Key) {
        use RadioAction::*;
        match menu::event(&self.radio_menu(), &mut self.index, key) {
            Event::Back => {
                self.tools.page = ToolPage::Radio;
                self.index = 0;
            }
            Event::Activate(FineTune) => self.tools.page = ToolPage::Radio,
            Event::Activate(Favorites) => {
                self.tools.page = ToolPage::RadioFavorites;
                self.index = 0;
            }
            Event::Activate(AddFavorite) => {
                if !self.tools.stations.contains(&self.tools.frequency) {
                    self.tools.stations.push(self.tools.frequency);
                }
                self.tools.page = ToolPage::Radio;
            }
            Event::Activate(Search) => self.notice("未连接收音机硬件。"),
            Event::Activate(Frequency) => self.open_local_text(
                TextTarget::Tool(2),
                format!("{:.1}", self.tools.frequency as f32 / 10.),
                false,
            ),
            Event::Activate(Close) => {
                self.tools.radio_on = false;
                self.leave_app();
            }
            Event::Activate(Speaker) | Event::Focus | Event::Ignored => {}
        }
    }
}
