//! Deterministic local utilities. Key assignments come from the observed handset;
//! arithmetic precision and timer internals are replica policy, not recovered code.
#[derive(Default)]
pub struct Calculator {
    pub entry: String,
    pub expression: String,
    accumulator: Option<f64>,
    operator: Option<char>,
    result: bool,
}
impl Calculator {
    pub fn display(&self) -> &str {
        if self.entry.is_empty() {
            "0"
        } else {
            &self.entry
        }
    }
    pub fn digit(&mut self, digit: u8) {
        if digit > 9 {
            return;
        }
        if self.result {
            self.clear();
        }
        if self.entry == "0" {
            self.entry.clear();
        }
        if self.entry.len() < 16 {
            self.entry.push(char::from(b'0' + digit));
        }
    }
    pub fn decimal(&mut self) {
        if self.result {
            self.clear();
        }
        if self.entry.is_empty() {
            self.entry.push('0');
        }
        if !self.entry.contains('.') {
            self.entry.push('.');
        }
    }
    pub fn sign(&mut self) {
        if self.entry.starts_with('-') {
            self.entry.remove(0);
        } else if !self.entry.is_empty() && self.entry != "0" {
            self.entry.insert(0, '-');
        }
    }
    /// Unary percent is a local arithmetic policy; native chaining is unverified.
    pub fn percent(&mut self) {
        if let Ok(value) = self.entry.parse::<f64>() {
            self.entry = format_number(value / 100.);
        }
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn backspace(&mut self) {
        if self.result {
            self.clear();
        } else {
            self.entry.pop();
        }
    }
    pub fn operation(&mut self, op: char) {
        if !self.entry.is_empty() {
            self.evaluate();
        }
        if self.entry == "错误" {
            return;
        }
        self.accumulator = self.entry.parse().ok().or(self.accumulator).or(Some(0.));
        self.operator = Some(op);
        self.expression = format!("{} {op}", self.accumulator.unwrap_or(0.));
        self.entry.clear();
        self.result = false;
    }
    pub fn evaluate(&mut self) {
        let rhs = self.entry.parse::<f64>().unwrap_or(0.);
        if let (Some(lhs), Some(op)) = (self.accumulator.take(), self.operator.take()) {
            let value = match op {
                '+' => lhs + rhs,
                '-' => lhs - rhs,
                '×' => lhs * rhs,
                '÷' => lhs / rhs,
                _ => rhs,
            };
            self.expression = format!("{lhs} {op} {rhs} =");
            self.entry = if value.is_finite() {
                format_number(value)
            } else {
                "错误".into()
            };
        } else if self.entry.is_empty() {
            self.entry = "0".into();
        }
        self.result = true;
    }
}
pub fn format_number(value: f64) -> String {
    if value.abs() > 1e12 || (value != 0. && value.abs() < 1e-8) {
        return format!("{value:.8e}");
    }
    let s = format!("{value:.8}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stopwatch {
    pub elapsed_ms: u64,
    pub running: bool,
    pub laps: Vec<u64>,
}
impl Stopwatch {
    pub fn tick(&mut self, ms: u64) {
        if self.running {
            self.elapsed_ms = self.elapsed_ms.saturating_add(ms);
        }
    }
    pub fn lap_or_reset(&mut self) {
        if self.running {
            if self.laps.len() < 100 {
                self.laps.insert(0, self.elapsed_ms);
            }
        } else {
            *self = Self::default();
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Timer {
    pub remaining_ms: u64,
    pub running: bool,
    pub expired: bool,
}
impl Timer {
    pub fn start(&mut self, seconds: u64) {
        self.remaining_ms = seconds * 1000;
        self.running = seconds > 0;
        self.expired = false;
    }
    pub fn tick(&mut self, ms: u64) {
        if self.running {
            self.remaining_ms = self.remaining_ms.saturating_sub(ms);
            if self.remaining_ms == 0 {
                self.running = false;
                self.expired = true;
            }
        }
    }
}
pub fn clock(ms: u64) -> String {
    let sec = ms / 1000;
    format!("{:02}:{:02}:{:02}", sec / 3600, sec / 60 % 60, sec % 60)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_calculator_chain_and_division_error() {
        let mut c = Calculator::default();
        c.digit(1);
        c.operation('+');
        c.digit(2);
        c.evaluate();
        assert_eq!(c.display(), "3");
        c.operation('×');
        c.digit(4);
        c.evaluate();
        assert_eq!(c.display(), "12");
        c.backspace();
        assert_eq!(c.display(), "0");
        c.digit(2);
        c.operation('÷');
        c.digit(0);
        c.evaluate();
        assert_eq!(c.display(), "错误");
        c.digit(7);
        assert_eq!(c.display(), "7");
    }
    #[test]
    fn stopped_time_does_not_advance_and_timer_fires_once() {
        let mut s = Stopwatch::default();
        s.tick(1000);
        assert_eq!(s.elapsed_ms, 0);
        s.running = true;
        s.tick(1200);
        s.lap_or_reset();
        assert_eq!(s.laps, [1200]);
        s.running = false;
        s.tick(1000);
        assert_eq!(s.elapsed_ms, 1200);
        s.lap_or_reset();
        assert_eq!(s.elapsed_ms, 0);
        let mut t = Timer::default();
        t.start(1);
        t.tick(1200);
        assert!(t.expired && !t.running);
        t.expired = false;
        t.tick(1000);
        assert!(!t.expired);
    }
}
