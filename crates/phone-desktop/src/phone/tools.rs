//! Utility routes sampled on the device. No microphone, radio, storage-device or LED access.
use super::*;
use phone_core::utilities::{Calculator, Stopwatch, Timer, format_number};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolPage {
    Other,
    Calculator,
    Stopwatch,
    Timer,
    TimerRunning,
    Converter,
    Conversion(usize),
    Flashlight,
    Radio,
    RadioOptions,
    RadioFavorites,
    Recorder,
    Recordings,
    Files,
    FileDetails,
    Counter,
    Video,
    Gallery,
    Calls,
}
pub struct Tools {
    pub page: ToolPage,
    pub calculator: Calculator,
    pub stopwatch: Stopwatch,
    pub timer: Timer,
    pub timer_seconds: u64,
    pub timer_name: String,
    pub torch: bool,
    pub frequency: u16,
    pub radio_on: bool,
    pub radio_speaker: bool,
    pub stations: Vec<u16>,
    pub recording: bool,
    pub record_ms: u64,
    pub recordings: Vec<u64>,
    pub conversion_input: String,
    pub conversion_reverse: bool,
}
impl Default for Tools {
    fn default() -> Self {
        Self {
            page: ToolPage::Other,
            calculator: Default::default(),
            stopwatch: Default::default(),
            timer: Default::default(),
            timer_seconds: 60,
            timer_name: String::new(),
            torch: false,
            frequency: 875,
            radio_on: false,
            radio_speaker: false,
            stations: vec![],
            recording: false,
            record_ms: 0,
            recordings: vec![],
            conversion_input: "0".into(),
            conversion_reverse: false,
        }
    }
}
impl Phone {
    pub fn tools_title(&self) -> &'static str {
        use ToolPage::*;
        match self.tools.page {
            Other => "其他",
            Calculator => "计算器",
            Stopwatch => "秒表",
            Timer | TimerRunning => "定时器",
            Converter | Conversion(_) => "单位换算器",
            Flashlight => "手电筒",
            Radio => "收音机",
            RadioOptions => "选项",
            RadioFavorites => "收藏夹",
            Recorder => "录音",
            Recordings => "录音列表",
            Files => "文件",
            FileDetails => "详情",
            Counter => "计数器",
            Video => "视频",
            Gallery => "多媒体资料",
            Calls => "通话历史记录",
        }
    }
    pub fn tools_items(&self) -> Vec<String> {
        use ToolPage::*;
        match self.tools.page {
            Other => vec![
                "定时器".into(),
                "秒表".into(),
                format!("手电筒  {}", if self.tools.torch { "开" } else { "关" }),
                "单位换算器".into(),
            ],
            Timer => vec![
                format!(
                    "时间  {}",
                    phone_core::utilities::clock(self.tools.timer_seconds * 1000)
                ),
                format!("名称  {}", self.tools.timer_name),
            ],
            Converter => ["温度", "重量", "长度", "面积", "体积", "货币"]
                .map(str::to_owned)
                .to_vec(),
            RadioOptions => [
                "收藏夹",
                "添加收藏",
                "扬声器",
                "自动搜索",
                "微调",
                "设置频率",
                "关闭收音机",
            ]
            .map(str::to_owned)
            .to_vec(),
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
            Files => vec!["手机存储".into()],
            _ => vec![],
        }
    }
    pub fn conversion(&self) -> (String, String, String, String) {
        let category = if let ToolPage::Conversion(n) = self.tools.page {
            n
        } else {
            0
        };
        let pairs = [
            ("华氏度", "摄氏度", 5. / 9.),
            ("千克", "磅", 2.2046226218),
            ("米", "英尺", 3.280839895),
            ("平方米", "平方英尺", 10.763910417),
            ("升", "美制加仑", 0.2641720524),
        ];
        let (a, b, factor) = pairs[category.min(4)];
        let v = self.tools.conversion_input.parse::<f64>().unwrap_or(0.);
        let result = if category == 0 {
            if self.tools.conversion_reverse {
                v * 9. / 5. + 32.
            } else {
                (v - 32.) * 5. / 9.
            }
        } else if self.tools.conversion_reverse {
            v / factor
        } else {
            v * factor
        };
        let (a, b) = if self.tools.conversion_reverse {
            (b, a)
        } else {
            (a, b)
        };
        (
            a.into(),
            b.into(),
            self.tools.conversion_input.clone(),
            format_number(result),
        )
    }
    pub(super) fn tools_key(&mut self, key: Key) {
        use ToolPage::*;
        match self.tools.page {
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
                    Key::SoftRight | Key::Delete => c.backspace(),
                    Key::SoftLeft => self.leave_app(),
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
            TimerRunning => {
                match key {
                    Key::Center => self.tools.timer.running = !self.tools.timer.running,
                    Key::SoftLeft => {
                        self.tools.timer = Default::default();
                        self.tools.page = Timer;
                    }
                    Key::SoftRight => {
                        self.tools.page = Other;
                        self.index = 0;
                    }
                    _ => {}
                }
                return;
            }
            Conversion(_) => {
                match key {
                    Key::Digit(d) => {
                        let s = &mut self.tools.conversion_input;
                        if s == "0" {
                            s.clear();
                        }
                        if s.len() < 16 {
                            s.push(char::from(b'0' + d));
                        }
                    }
                    Key::Hash => {
                        if !self.tools.conversion_input.contains('.') {
                            self.tools.conversion_input.push('.');
                        }
                    }
                    Key::Star => {
                        let s = &mut self.tools.conversion_input;
                        if s.starts_with('-') {
                            s.remove(0);
                        } else {
                            s.insert(0, '-');
                        }
                    }
                    Key::Delete => {
                        self.tools.conversion_input.pop();
                    }
                    Key::Up | Key::Down => {
                        self.tools.conversion_reverse = !self.tools.conversion_reverse
                    }
                    Key::SoftLeft | Key::SoftRight => {
                        self.tools.page = Converter;
                        self.index = 0;
                    }
                    _ => {}
                }
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
            Key::Up if n > 0 => self.index = (self.index + n - 1) % n,
            Key::Down if n > 0 => self.index = (self.index + 1) % n,
            Key::SoftRight => match self.tools.page {
                Timer | Converter => {
                    self.tools.page = Other;
                    self.index = 0;
                }
                RadioOptions | RadioFavorites => {
                    self.tools.page = Radio;
                    self.index = 0;
                }
                Recordings => {
                    self.tools.page = Recorder;
                    self.index = 0;
                }
                FileDetails => {
                    self.tools.page = Files;
                    self.index = 0;
                }
                _ => self.leave_app(),
            },
            Key::SoftLeft if self.tools.page == Timer => {
                self.tools.timer.start(self.tools.timer_seconds);
                self.tools.page = TimerRunning;
            }
            Key::Center if n > 0 => match self.tools.page {
                Other => {
                    self.tools.page = [Timer, Stopwatch, Flashlight, Converter][self.index];
                    self.index = 0;
                }
                Timer => self.open_local_text(
                    TextTarget::Tool(self.index),
                    if self.index == 0 {
                        self.tools.timer_seconds.to_string()
                    } else {
                        self.tools.timer_name.clone()
                    },
                    self.index == 0,
                ),
                Converter if self.index < 5 => {
                    self.tools.page = Conversion(self.index);
                    self.tools.conversion_input = "0".into();
                    self.tools.conversion_reverse = false;
                }
                Converter => self.notice("没有汇率数据。"),
                RadioOptions => match self.index {
                    0 => {
                        self.tools.page = RadioFavorites;
                        self.index = 0;
                    }
                    1 => {
                        if !self.tools.stations.contains(&self.tools.frequency) {
                            self.tools.stations.push(self.tools.frequency);
                        }
                        self.tools.page = Radio;
                    }
                    2 => {
                        self.tools.radio_speaker = !self.tools.radio_speaker;
                        self.tools.page = Radio;
                    }
                    3 => self.notice("未连接收音机硬件。"),
                    4 => self.tools.page = Radio,
                    5 => self.open_local_text(
                        TextTarget::Tool(2),
                        format!("{:.1}", self.tools.frequency as f32 / 10.),
                        false,
                    ),
                    _ => {
                        self.tools.radio_on = false;
                        self.leave_app();
                    }
                },
                RadioFavorites => {
                    self.tools.frequency = self.tools.stations[self.index];
                    self.tools.radio_on = true;
                    self.tools.page = Radio;
                }
                Files => {
                    self.tools.page = FileDetails;
                    self.index = 0;
                }
                Recordings => self.notice("仅保存了计时记录，没有采集音频。"),
                _ => {}
            },
            _ => {}
        }
    }
    pub(super) fn tool_text(&mut self, field: usize, value: String) {
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
