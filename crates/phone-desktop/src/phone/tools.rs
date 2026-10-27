//! Utility routes sampled on the device. No microphone, radio, storage-device or LED access.
use super::*;
use phone_core::utilities::{Calculator, Stopwatch, Timer};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolPage {
    Other,
    Calculator,
    Stopwatch,
    Timer,
    TimerMenu,
    Converter,
    ConversionPairs(usize),
    Conversion(usize),
    Flashlight,
    Radio,
    RadioOptions,
    RadioFavorites,
    Recorder,
    Recordings,
    Files,
    Counter,
    Video,
    Gallery,
    Calls,
}
pub struct Tools {
    pub dirty: bool,
    pub timer_notice: bool,
    pub page: ToolPage,
    pub calculator: Calculator,
    pub stopwatch: Stopwatch,
    pub timer: Timer,
    pub timer_seconds: u64,
    pub timer_name: String,
    pub timer_edit: super::timer::TimerEdit,
    pub torch: bool,
    pub frequency: u16,
    pub radio_on: bool,
    pub radio_speaker: bool,
    pub stations: Vec<u16>,
    pub recording: bool,
    pub record_ms: u64,
    pub recordings: Vec<u64>,
    pub conversion: super::converter::ConversionUi,
}
impl Default for Tools {
    fn default() -> Self {
        Self {
            dirty: false,
            timer_notice: false,
            page: ToolPage::Other,
            calculator: Default::default(),
            stopwatch: Default::default(),
            timer: Default::default(),
            timer_seconds: 60,
            timer_name: String::new(),
            timer_edit: Default::default(),
            torch: false,
            frequency: 875,
            radio_on: false,
            radio_speaker: false,
            stations: vec![],
            recording: false,
            record_ms: 0,
            recordings: vec![],
            conversion: Default::default(),
        }
    }
}
impl Phone {
    pub fn other_pages(&self) -> Vec<ToolPage> {
        [
            ToolPage::Timer,
            ToolPage::Stopwatch,
            ToolPage::Flashlight,
            ToolPage::Converter,
        ]
        .into_iter()
        .filter(|p| self.apps.tool(*p))
        .collect()
    }
    pub fn other_index(&self, page: ToolPage) -> usize {
        self.other_pages()
            .iter()
            .position(|p| *p == page)
            .unwrap_or(0)
    }
    pub fn tools_title(&self) -> &'static str {
        use ToolPage::*;
        match self.tools.page {
            Other => "其他",
            Calculator => "计算器",
            Stopwatch => "秒表",
            Timer | TimerMenu => "定时器",
            Converter => "单位转换",
            ConversionPairs(n) | Conversion(n) => phone_core::converter::CATEGORIES[n.min(5)],
            Flashlight => "手电筒",
            Radio => "收音机",
            RadioOptions => "选项",
            RadioFavorites => "收藏夹",
            Recorder => "录音",
            Recordings => "录音列表",
            Files => "文件",
            Counter => "计数器",
            Video => "视频",
            Gallery => "多媒体资料",
            Calls => "通话历史记录",
        }
    }
    pub fn tools_items(&self) -> Vec<String> {
        use ToolPage::*;
        match self.tools.page {
            Other => self
                .other_pages()
                .iter()
                .map(|page| {
                    match page {
                        Timer => "定时器",
                        Stopwatch => "秒表",
                        Flashlight => "手电筒",
                        _ => "单位换算",
                    }
                    .to_owned()
                })
                .collect(),
            Timer => vec![
                format!(
                    "时间  {}",
                    phone_core::utilities::clock(self.tools.timer_seconds * 1000)
                ),
                format!("名称  {}", self.tools.timer_name),
            ],
            Converter => phone_core::converter::CATEGORIES
                .iter()
                .map(|s| (*s).into())
                .collect(),
            ConversionPairs(n) => phone_core::converter::PAIRS
                .get(n)
                .map(|pairs| pairs.iter().map(|p| p.title.into()).collect())
                .unwrap_or_default(),
            RadioOptions => self
                .radio_menu()
                .iter()
                .map(|item| item.label.to_owned())
                .collect(),
            RadioFavorites => self
                .tools
                .stations
                .iter()
                .map(|f| format!("{:.1} MHz", *f as f32 / 10.))
                .collect(),
            Recordings => self
                .tools
                .recordings
                .iter()
                .enumerate()
                .map(|(i, t)| format!("录音 {}  {}", i + 1, phone_core::utilities::clock(*t)))
                .collect(),
            _ => vec![],
        }
    }
    pub(super) fn tools_key(&mut self, key: Key) {
        self.tools.dirty = true;
        use ToolPage::*;
        match self.tools.page {
            Files => {
                self.files_key(key);
                return;
            }
            Calculator => {
                let c = &mut self.tools.calculator;
                match key {
                    Key::Digit(d) => c.digit(d),
                    Key::Up => c.operation('+'),
                    Key::Down => c.operation('-'),
                    Key::Left => c.operation('÷'),
                    Key::Right => c.operation('×'),
                    Key::Star => c.sign(),
                    Key::Hash => c.decimal(),
                    Key::Center => c.evaluate(),
                    Key::SoftRight if c.entry.is_empty() && c.expression.is_empty() => {
                        self.leave_app()
                    }
                    Key::SoftRight | Key::Delete => c.backspace(),
                    Key::SoftLeft => c.percent(),
                    _ => {}
                }
                return;
            }
            Stopwatch => {
                match key {
                    Key::Center => self.tools.stopwatch.running = !self.tools.stopwatch.running,
                    Key::SoftLeft => self.tools.stopwatch.lap_or_reset(),
                    Key::SoftRight => {
                        self.tools.page = Other;
                        self.index = 1;
                    }
                    _ => {}
                }
                return;
            }
            Timer | TimerMenu => {
                self.timer_key(key);
                return;
            }
            Conversion(_) | ConversionPairs(_) => {
                self.conversion_key(key);
                return;
            }
            Flashlight => {
                match key {
                    Key::Center => self.tools.torch = !self.tools.torch,
                    Key::SoftRight => {
                        if self.screen == Screen::Tools(Other) {
                            self.tools.page = Other;
                            self.index = 2;
                        } else {
                            self.leave_app();
                        }
                    }
                    _ => {}
                }
                return;
            }
            Radio => {
                match key {
                    Key::Left => {
                        self.tools.frequency = self.tools.frequency.saturating_sub(1).max(875)
                    }
                    Key::Right => self.tools.frequency = (self.tools.frequency + 1).min(1080),
                    Key::Center => self.tools.radio_on = !self.tools.radio_on,
                    Key::SoftLeft => {
                        self.tools.page = RadioOptions;
                        self.index = 0;
                    }
                    Key::SoftRight => self.leave_app(),
                    _ => {}
                }
                return;
            }
            RadioOptions => {
                self.radio_options_key(key);
                return;
            }
            Recorder => {
                match key {
                    Key::Center => {
                        if self.tools.recording {
                            self.tools.recordings.push(self.tools.record_ms);
                            self.tools.recording = false;
                        } else {
                            self.tools.record_ms = 0;
                            self.tools.recording = true;
                        }
                    }
                    Key::SoftLeft => {
                        self.tools.page = Recordings;
                        self.index = 0;
                    }
                    Key::SoftRight => {
                        if self.tools.recording {
                            self.tools.recordings.push(self.tools.record_ms);
                            self.tools.recording = false;
                        }
                        self.leave_app();
                    }
                    _ => {}
                }
                return;
            }
            _ => {}
        }
        let n = self.tools_items().len();
        match key {
            Key::Up | Key::Down => {
                vertical(&mut self.index, n, key, Boundary::Wrap);
            }
            Key::SoftRight => match self.tools.page {
                Timer | Converter => {
                    let converter = self.tools.page == Converter;
                    self.tools.page = Other;
                    self.index = if converter {
                        self.other_index(Converter)
                    } else {
                        0
                    };
                }
                RadioOptions | RadioFavorites => {
                    self.tools.page = Radio;
                    self.index = 0;
                }
                Recordings => {
                    self.tools.page = Recorder;
                    self.index = 0;
                }
                _ => self.leave_app(),
            },
            Key::Center if n > 0 => match self.tools.page {
                Other => {
                    if self.index == 0 {
                        if self.tools.timer.running {
                            self.tools.page = TimerMenu;
                            self.index = 0;
                        } else {
                            self.open_timer();
                        }
                    } else {
                        self.tools.page = self.other_pages()[self.index];
                        self.index = 0;
                    }
                }
                Converter if self.index < 5 => {
                    self.tools.page = ConversionPairs(self.index);
                    self.index = 0;
                }
                Converter => self.notice("没有汇率数据。"),
                RadioFavorites => {
                    self.tools.frequency = self.tools.stations[self.index];
                    self.tools.radio_on = true;
                    self.tools.page = Radio;
                }
                Recordings => self.notice("仅保存了计时记录，没有采集音频。"),
                _ => {}
            },
            _ => {}
        }
    }
    pub(super) fn tool_text(&mut self, field: usize, value: String) {
        self.tools.dirty = true;
        match field {
            0 => match value.parse::<u64>() {
                Ok(seconds) if seconds <= 359999 => self.tools.timer_seconds = seconds,
                _ => self.notice("请输入 0 到 359999 秒。"),
            },
            1 => self.tools.timer_name = value,
            2 => match value.parse::<f32>() {
                Ok(f) if (87.5..=108.0).contains(&f) => {
                    self.tools.frequency = (f * 10.).round() as u16;
                    self.tools.page = ToolPage::Radio;
                }
                _ => self.notice("频率范围是 87.5–108.0 MHz。"),
            },
            _ => {}
        }
    }
}
