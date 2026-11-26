//! Local wall-clock time: what the reference's `new Date()` reads (the
//! machine's time zone, not UTC). Shared by components and demos that show
//! "now" or "today": toast captions (`toLocaleTimeString()`), Calendar's
//! today.
use chrono::{Datelike as _, Local, Timelike as _};

/// A local date and time, to the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalDateTime {
    /// The year, e.g. 2026.
    pub year: i32,
    /// The month, 1–12.
    pub month: u32,
    /// The day of the month, 1–31.
    pub day: u32,
    /// The hour, 0–23.
    pub hour: u32,
    /// The minute, 0–59.
    pub minute: u32,
    /// The second, 0–59.
    pub second: u32,
}

/// Now, in the machine's local time zone.
pub fn now() -> LocalDateTime {
    let t = Local::now();
    LocalDateTime {
        year: t.year(),
        month: t.month(),
        day: t.day(),
        hour: t.hour(),
        minute: t.minute(),
        second: t.second().min(59),
    }
}

impl LocalDateTime {
    /// `toLocaleTimeString()` in the reference's `en-US` locale:
    /// `h:mm:ss AM|PM`, e.g. `"2:05:09 PM"`, `"12:00:00 AM"`.
    pub fn time_string(self) -> String {
        let (hour, meridiem) = match self.hour {
            0 => (12, "AM"),
            h @ 1..=11 => (h, "AM"),
            12 => (12, "PM"),
            h => (h - 12, "PM"),
        };
        format!("{hour}:{:02}:{:02} {meridiem}", self.minute, self.second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(hour: u32, minute: u32, second: u32) -> LocalDateTime {
        LocalDateTime {
            year: 2026,
            month: 1,
            day: 1,
            hour,
            minute,
            second,
        }
    }

    #[test]
    fn time_string_matches_to_locale_time_string() {
        assert_eq!(at(14, 5, 9).time_string(), "2:05:09 PM");
        assert_eq!(at(0, 0, 0).time_string(), "12:00:00 AM");
        assert_eq!(at(12, 30, 0).time_string(), "12:30:00 PM");
        assert_eq!(at(9, 41, 7).time_string(), "9:41:07 AM");
    }

    #[test]
    fn now_is_local_time() {
        let n = now();
        let local = Local::now();
        assert_eq!(
            (n.year, n.month, n.day),
            (local.year(), local.month(), local.day())
        );
        assert!(n.hour < 24 && n.minute < 60 && n.second < 60);
    }
}
