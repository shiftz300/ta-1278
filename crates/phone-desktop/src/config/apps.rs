//! Optional application policy. Core phone routes cannot be removed by configuration.
use crate::phone::{Screen, alarm, contacts, organizer::OrganizerPage, tools::ToolPage};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Apps {
    pub browser: bool,
    pub music: bool,
    pub calendar: bool,
    pub alarms: bool,
    pub snake: bool,
    pub radio: bool,
    pub flashlight: bool,
    pub recorder: bool,
    pub calculator: bool,
    /// Other: timer, stopwatch and converter.
    pub utilities: bool,
    pub counter: bool,
    pub pictures: bool,
    pub video: bool,
}
impl Default for Apps {
    fn default() -> Self {
        Self {
            browser: true,
            music: true,
            calendar: true,
            alarms: true,
            snake: true,
            radio: true,
            flashlight: true,
            recorder: true,
            calculator: true,
            utilities: true,
            counter: true,
            pictures: true,
            video: true,
        }
    }
}
impl Apps {
    pub fn allows(self, screen: Screen) -> bool {
        match screen {
            Screen::Browser => self.browser,
            Screen::Music => self.music,
            Screen::Snake => self.snake,
            Screen::Organizer(page)
                if alarm::is_alarm(page) || page == OrganizerPage::DeleteAlarm =>
            {
                self.alarms
            }
            Screen::Organizer(page)
                if contacts::is_contact(page) || page == OrganizerPage::DeleteContact =>
            {
                true
            }
            Screen::Organizer(_) => self.calendar,
            Screen::Tools(page) => self.tool(page),
            _ => true,
        }
    }
    pub fn tool(self, page: ToolPage) -> bool {
        use ToolPage::*;
        match page {
            Radio | RadioOptions | RadioFavorites => self.radio,
            Flashlight => self.flashlight,
            Recorder | Recordings => self.recorder,
            Calculator => self.calculator,
            Other | Timer | TimerMenu | Stopwatch | Converter | ConversionPairs(_)
            | Conversion(_) => self.utilities,
            Counter => self.counter,
            Gallery => self.pictures,
            Video => self.video,
            Files | Calls => true, // Calls keep their explicitly disabled grey entry.
        }
    }
}
