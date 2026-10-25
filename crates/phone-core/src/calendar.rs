//! Gregorian calendar arithmetic, independent of host timezone and UI.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}
impl Date {
    /// Editable digits in the fixed YYYY-MM-DD display format.
    pub const DIGIT_OFFSETS: [usize; 8] = [0, 1, 2, 3, 5, 6, 8, 9];
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        let d = Self { year, month, day };
        if (1900..=9999).contains(&year)
            && (1..=12).contains(&month)
            && day >= 1
            && day <= d.month_days()
        {
            Some(d)
        } else {
            None
        }
    }
    pub fn month_days(self) -> u32 {
        match self.month {
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.year % 4 == 0 && (self.year % 100 != 0 || self.year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 31,
        }
    }
    pub fn next(self, delta: i32) -> Self {
        let mut d = self;
        for _ in 0..delta.unsigned_abs().min(36600) {
            if delta > 0 {
                if d.day < d.month_days() {
                    d.day += 1;
                } else {
                    d.day = 1;
                    if d.month == 12 {
                        d.month = 1;
                        d.year += 1;
                    } else {
                        d.month += 1;
                    }
                }
            } else if d.day > 1 {
                d.day -= 1;
            } else {
                if d.month == 1 {
                    d.month = 12;
                    d.year -= 1;
                } else {
                    d.month -= 1;
                }
                d.day = d.month_days();
            }
        }
        d
    }
    pub fn month_shift(self, delta: i32) -> Self {
        let m = self.year * 12 + self.month as i32 - 1 + delta;
        let mut d = Self {
            year: m.div_euclid(12),
            month: m.rem_euclid(12) as u32 + 1,
            day: self.day,
        };
        d.day = d.day.min(d.month_days());
        d
    }
    pub fn weekday(self) -> usize {
        // Monday = 0.
        let offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = self.year - if self.month < 3 { 1 } else { 0 };
        ((y + y / 4 - y / 100 + y / 400 + offsets[(self.month - 1) as usize] + self.day as i32 + 6)
            .rem_euclid(7)) as usize
    }
    pub fn parse(s: &str) -> Option<Self> {
        let v = s
            .split('-')
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        if v.len() != 3 {
            return None;
        }
        Self::new(v[0] as i32, v[1], v[2])
    }
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
    pub fn from_unix(seconds: u64, offset: i32) -> Self {
        let mut days = (seconds as i64 + offset as i64).div_euclid(86400);
        let mut year = 1970;
        loop {
            let count = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                366
            } else {
                365
            };
            if days < count {
                break;
            }
            days -= count;
            year += 1;
        }
        let mut d = Self {
            year,
            month: 1,
            day: 1,
        };
        while days >= d.month_days() as i64 {
            days -= d.month_days() as i64;
            d.month += 1;
        }
        d.day = days as u32 + 1;
        d
    }
}
impl Default for Date {
    fn default() -> Self {
        let sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self::from_unix(sec, 8 * 3600)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leap_and_month_edges() {
        let d = Date::new(2024, 2, 28).unwrap();
        assert_eq!(d.next(2).iso(), "2024-03-01");
        assert_eq!(d.next(2).next(-2), d);
        assert!(Date::new(2100, 2, 29).is_none());
        assert_eq!(Date::new(2026, 10, 25).unwrap().weekday(), 6);
        assert_eq!(Date::new(2024, 1, 31).unwrap().month_shift(1).day, 29);
        assert_eq!(Date::from_unix(0, 0).iso(), "1970-01-01");
    }
}
