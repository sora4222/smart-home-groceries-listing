//! The time buckets of the "Over time" view: weeks, months and quarters.
//!
//! A week starts on Monday; a quarter on 1 January, April, July or October.
//! Days are the household's own calendar days ([`super::range::local_date`]).
//! Pure: no I/O.

use chrono::{Datelike, Days, Months, NaiveDate};
use serde::Deserialize;

/// How the "Over time" view groups spending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    Week,
    #[default]
    Month,
    Quarter,
}

impl Period {
    /// The first day of the bucket `day` falls in.
    pub fn start_of(self, day: NaiveDate) -> NaiveDate {
        match self {
            Period::Week => day - Days::new(u64::from(day.weekday().num_days_from_monday())),
            Period::Month => first_of_month(day.year(), day.month()),
            Period::Quarter => first_of_month(day.year(), (day.month0() / 3) * 3 + 1),
        }
    }

    /// The first day of the bucket after the one starting on `start`.
    pub fn next(self, start: NaiveDate) -> NaiveDate {
        match self {
            Period::Week => start + Days::new(7),
            Period::Month => start + Months::new(1),
            Period::Quarter => start + Months::new(3),
        }
    }
}

/// 1 `month` of `year`. Months come from a real date, so this cannot fail.
fn first_of_month(year: i32, month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn a_week_starts_on_monday() {
        // 2 October 2026 is a Friday.
        assert_eq!(Period::Week.start_of(day("2026-10-02")), day("2026-09-28"));
        assert_eq!(Period::Week.start_of(day("2026-09-28")), day("2026-09-28"));
        assert_eq!(Period::Week.next(day("2026-09-28")), day("2026-10-05"));
    }

    #[test]
    fn a_month_starts_on_the_first() {
        assert_eq!(Period::Month.start_of(day("2026-10-31")), day("2026-10-01"));
        assert_eq!(Period::Month.next(day("2026-12-01")), day("2027-01-01"));
    }

    #[test]
    fn quarters_start_in_january_april_july_and_october() {
        assert_eq!(
            Period::Quarter.start_of(day("2026-03-31")),
            day("2026-01-01")
        );
        assert_eq!(
            Period::Quarter.start_of(day("2026-05-15")),
            day("2026-04-01")
        );
        assert_eq!(
            Period::Quarter.start_of(day("2026-12-25")),
            day("2026-10-01")
        );
        assert_eq!(Period::Quarter.next(day("2026-10-01")), day("2027-01-01"));
    }

    #[test]
    fn month_is_the_default() {
        assert_eq!(Period::default(), Period::Month);
    }
}
