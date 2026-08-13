//! `Date` — a calendar date parsed **wide, then range-checked**
//! (`[R:parse-wide-then-range-check]`). The parser reads each component into
//! `i64` — wide enough to *hold* an out-of-range value like month 13 or day 40
//! — and only then narrows to the calendar type. That is what lets an
//! impossible date report as `MonthOutOfRange(13)` rather than collapsing into
//! a syntax error, keeping the out-of-range variants reachable for the very
//! values they name.

use std::fmt;

/// A valid calendar date. Fields are private and the only constructor is
/// [`Date::parse`], so a `Date` in hand is always a real day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

/// Why a string is not a valid [`Date`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DateError {
    /// The input was empty or whitespace only.
    #[error("date is empty")]
    Empty,
    /// The input was not three `-`-separated integers (`YYYY-MM-DD`).
    #[error("date must be YYYY-MM-DD (got {0:?})")]
    Syntax(String),
    /// The year did not fit the calendar's range. Carries the parsed value.
    #[error("year {0} is out of range")]
    YearOutOfRange(i64),
    /// The month was well-formed but outside `1..=12`. Carries the parsed
    /// value, which the wide parse kept visible.
    #[error("month {0} is out of range 1..=12")]
    MonthOutOfRange(i64),
    /// The day was well-formed but outside `1..=(days in that month)`. Carries
    /// the parsed value.
    #[error("day {0} is out of range for that month")]
    DayOutOfRange(i64),
}

impl Date {
    /// Parse `YYYY-MM-DD`, wide then range-checked.
    pub fn parse(s: &str) -> Result<Self, DateError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(DateError::Empty);
        }
        let parts: Vec<&str> = s.split('-').collect();
        let [ys, ms, ds] = parts.as_slice() else {
            return Err(DateError::Syntax(s.to_owned()));
        };
        // Parse WIDE: `i64` can represent 13, 40, 99999 — the very values the
        // range check below must be able to see in order to name them.
        let y: i64 = ys.parse().map_err(|_| DateError::Syntax(s.to_owned()))?;
        let m: i64 = ms.parse().map_err(|_| DateError::Syntax(s.to_owned()))?;
        let d: i64 = ds.parse().map_err(|_| DateError::Syntax(s.to_owned()))?;

        // Range-check DOWN to the calendar type; each variant names its value.
        let year = i32::try_from(y).map_err(|_| DateError::YearOutOfRange(y))?;
        if !(1..=12).contains(&m) {
            return Err(DateError::MonthOutOfRange(m));
        }
        let month = m as u8; // 1..=12 fits u8
        let max_day = i64::from(days_in_month(year, month));
        if !(1..=max_day).contains(&d) {
            return Err(DateError::DayOutOfRange(d));
        }
        let day = d as u8; // 1..=31 fits u8
        Ok(Date { year, month, day })
    }

    /// The year.
    #[must_use]
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, `1..=12`.
    #[must_use]
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day, `1..=31`.
    #[must_use]
    pub fn day(self) -> u8 {
        self.day
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        // Unreachable once `month` is range-checked to 1..=12, but the function
        // stays total without a panic.
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_a_real_date() {
        let d = Date::parse("2026-08-13").expect("valid date");
        assert_eq!((d.year(), d.month(), d.day()), (2026, 8, 13));
    }

    #[test]
    fn empty_is_empty_not_syntax() {
        assert_eq!(Date::parse("   "), Err(DateError::Empty));
    }

    #[test]
    fn non_numeric_is_syntax() {
        assert!(matches!(
            Date::parse("2026-aug-13"),
            Err(DateError::Syntax(_))
        ));
    }

    #[test]
    fn wrong_shape_is_syntax() {
        assert!(matches!(Date::parse("2026-08"), Err(DateError::Syntax(_))));
    }

    // THE dogfood: a well-formed but impossible month reports as out-of-range,
    // carrying the offending value — not as a syntax error, not silently.
    #[test]
    fn month_thirteen_is_range_error_carrying_the_value() {
        assert_eq!(
            Date::parse("2026-13-01"),
            Err(DateError::MonthOutOfRange(13))
        );
    }

    #[test]
    fn day_forty_is_range_error_carrying_the_value() {
        assert_eq!(Date::parse("2026-01-40"), Err(DateError::DayOutOfRange(40)));
    }

    #[test]
    fn leap_day_valid_only_in_leap_years() {
        assert!(Date::parse("2024-02-29").is_ok());
        assert_eq!(Date::parse("2026-02-29"), Err(DateError::DayOutOfRange(29)));
    }

    #[test]
    fn zero_month_and_zero_day_are_range_errors() {
        assert_eq!(
            Date::parse("2026-00-10"),
            Err(DateError::MonthOutOfRange(0))
        );
        assert_eq!(Date::parse("2026-01-00"), Err(DateError::DayOutOfRange(0)));
    }

    proptest! {
        /// Every real date round-trips through `Display` -> `parse`.
        #[test]
        fn valid_dates_roundtrip(year in 1i32..=9999, month in 1u8..=12) {
            let day = days_in_month(year, month) / 2 + 1; // an in-range day
            let s = format!("{year:04}-{month:02}-{day:02}");
            let d = Date::parse(&s).expect("constructed date is valid");
            prop_assert_eq!(d.to_string(), s);
        }

        /// THE property: any month strictly above 12 is a `MonthOutOfRange`
        /// carrying that exact value — the wide parse made the value visible.
        #[test]
        fn out_of_range_month_is_named_not_syntax(month in 13i64..=9999) {
            let s = format!("2026-{month}-01");
            prop_assert_eq!(Date::parse(&s), Err(DateError::MonthOutOfRange(month)));
        }
    }
}
