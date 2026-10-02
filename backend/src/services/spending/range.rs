//! Turning the household's calendar days into instants, and back.
//!
//! The web app sends days ("from 1 September to 30 September") and the
//! browser's time zone. A day runs from local midnight to the next local
//! midnight, so a shop at 11pm in Sydney counts on that day, not the UTC one.
//! Pure: no I/O.

use chrono::{DateTime, Days, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

/// The instants a `from`–`to` range of days covers: `[start, end)`.
/// Either side may be open.
pub fn bounds(
    tz: Tz,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
    let start = from.map(|day| day_start(tz, day));
    let end = to.and_then(|day| {
        day.checked_add_days(Days::new(1))
            .map(|next| day_start(tz, next))
    });
    (start, end)
}

/// The household's calendar day at `instant`.
pub fn local_date(tz: Tz, instant: DateTime<Utc>) -> NaiveDate {
    instant.with_timezone(&tz).date_naive()
}

/// The first instant of `day` in `tz`. Where a clock change skips midnight,
/// the first instant that exists that day.
fn day_start(tz: Tz, day: NaiveDate) -> DateTime<Utc> {
    (0..24)
        .filter_map(|hour| day.and_hms_opt(hour, 0, 0))
        .find_map(|local| tz.from_local_datetime(&local).earliest())
        .map_or_else(
            || Utc.from_utc_datetime(&day.and_hms_opt(0, 0, 0).unwrap_or_default()),
            |start| start.with_timezone(&Utc),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYDNEY: Tz = chrono_tz::Australia::Sydney;

    fn day(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
    }

    fn utc(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn a_range_runs_from_local_midnight_to_the_midnight_after_the_last_day() {
        let (start, end) = bounds(SYDNEY, Some(day("2026-09-01")), Some(day("2026-09-30")));
        // Sydney is UTC+10 in September, UTC+11 from 4 October.
        assert_eq!(start, Some(utc("2026-08-31T14:00:00Z")));
        assert_eq!(end, Some(utc("2026-09-30T14:00:00Z")));
    }

    #[test]
    fn open_sides_stay_open() {
        assert_eq!(bounds(SYDNEY, None, None), (None, None));
    }

    #[test]
    fn a_late_evening_shop_counts_on_the_local_day() {
        assert_eq!(
            local_date(SYDNEY, utc("2026-09-30T13:30:00Z")),
            day("2026-09-30")
        );
        assert_eq!(
            local_date(SYDNEY, utc("2026-09-30T14:30:00Z")),
            day("2026-10-01")
        );
    }
}
