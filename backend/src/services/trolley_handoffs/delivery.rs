//! The delivery window a handoff asks for, and the store tab's report of the
//! window it reserved.
//!
//! The window itself is chosen on the store's website by the bookmarklet,
//! because only the household's logged-in tab can see the account's address
//! and windows. The default is the day after the store's own "today", any
//! time of day, cheapest then earliest. The household can change the window
//! on the store's website afterwards.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;

use crate::error::ApiError;
use crate::models::db::{DeliveryOutcome, DeliveryTimeOfDay};

/// The delivery time the web app asked for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeliveryRequest {
    /// `None` = the day after the store's today (the default).
    pub date: Option<NaiveDate>,
    pub time_of_day: DeliveryTimeOfDay,
}

/// How far ahead a delivery day may be asked for. Stores show about a week.
pub const MAX_DAYS_AHEAD: i64 = 14;

/// Checks the asked-for day is not in the past and not too far ahead.
///
/// `today` is the server's UTC date; one day of slack either side covers the
/// household being ahead of UTC (Melbourne is UTC+10 or +11).
pub fn check_requested_date(request: &DeliveryRequest, today: NaiveDate) -> Result<(), ApiError> {
    let Some(date) = request.date else {
        return Ok(());
    };
    let days_ahead = (date - today).num_days();
    if days_ahead < -1 {
        return Err(ApiError::UnprocessableEntity(format!(
            "The delivery day {date} is in the past."
        )));
    }
    if days_ahead > MAX_DAYS_AHEAD + 1 {
        return Err(ApiError::UnprocessableEntity(format!(
            "The delivery day {date} is more than {MAX_DAYS_AHEAD} days ahead."
        )));
    }
    Ok(())
}

/// What the store tab says it did about the delivery window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportedDelivery {
    pub outcome: DeliveryOutcome,
    pub window_label: Option<String>,
    pub window_start: Option<NaiveDateTime>,
    pub window_end: Option<NaiveDateTime>,
    pub fee: Option<Decimal>,
    pub problem: Option<String>,
}

/// Checks a delivery report makes sense: a reserved or kept window says
/// which window (label, start before end); a failure says why.
pub fn check_delivery_report(delivery: &ReportedDelivery) -> Result<(), ApiError> {
    let refuse = |why: &str| Err(ApiError::UnprocessableEntity(why.to_string()));
    match delivery.outcome {
        DeliveryOutcome::Failed if delivery.problem.is_none() => {
            refuse("A failed delivery window must say why.")
        }
        DeliveryOutcome::Failed => Ok(()),
        DeliveryOutcome::Reserved | DeliveryOutcome::Kept => {
            match (
                &delivery.window_label,
                delivery.window_start,
                delivery.window_end,
            ) {
                (Some(_), Some(start), Some(end)) if start < end => Ok(()),
                (Some(_), Some(_), Some(_)) => {
                    refuse("A delivery window must end after it starts.")
                }
                _ => refuse("A reserved delivery window needs its label, start and end."),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(hour: u32) -> Option<NaiveDateTime> {
        NaiveDate::from_ymd_opt(2026, 10, 3).and_then(|d| d.and_hms_opt(hour, 0, 0))
    }

    fn reserved(start: u32, end: u32) -> ReportedDelivery {
        ReportedDelivery {
            outcome: DeliveryOutcome::Reserved,
            window_label: Some("7:00am - 10:00am".into()),
            window_start: at(start),
            window_end: at(end),
            fee: Some(Decimal::new(15, 0)),
            problem: None,
        }
    }

    #[test]
    fn a_reserved_window_needs_a_label_and_a_start_before_its_end() {
        assert!(check_delivery_report(&reserved(7, 10)).is_ok());
        assert!(check_delivery_report(&reserved(10, 7)).is_err());
        let unlabelled = ReportedDelivery {
            window_label: None,
            ..reserved(7, 10)
        };
        assert!(check_delivery_report(&unlabelled).is_err());
    }

    #[test]
    fn the_day_must_be_from_today_to_two_weeks_ahead() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
        let on = |offset: i64| DeliveryRequest {
            date: Some(today + chrono::Duration::days(offset)),
            time_of_day: DeliveryTimeOfDay::Any,
        };
        assert!(check_requested_date(&DeliveryRequest::default(), today).is_ok());
        assert!(check_requested_date(&on(1), today).is_ok());
        assert!(
            check_requested_date(&on(-1), today).is_ok(),
            "UTC can be a day behind"
        );
        assert!(check_requested_date(&on(-2), today).is_err());
        assert!(check_requested_date(&on(15), today).is_ok());
        assert!(check_requested_date(&on(16), today).is_err());
    }

    #[test]
    fn a_failure_must_say_why() {
        let failed = ReportedDelivery {
            outcome: DeliveryOutcome::Failed,
            window_label: None,
            window_start: None,
            window_end: None,
            fee: None,
            problem: None,
        };
        assert!(check_delivery_report(&failed).is_err());
        let explained = ReportedDelivery {
            problem: Some("No windows this week".into()),
            ..failed
        };
        assert!(check_delivery_report(&explained).is_ok());
    }
}
