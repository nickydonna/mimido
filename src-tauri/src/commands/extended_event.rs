use chrono::{DateTime, TimeZone, Utc};
use now::DateTimeNow;

use crate::{
    calendar_items::input_traits::ToInput,
    models::vevent::{VEvent, VEventTrait},
};

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct ExtendedEvent {
    /// Date when the extended event was calculated
    pub query_date: DateTime<Utc>,
    pub event: VEvent,
    /// The start date of the event, if recurrent the value for the current query
    pub starts_at: DateTime<Utc>,
    /// The end date of the event, if recurrent the value for the current query
    pub ends_at: DateTime<Utc>,
    pub natural_recurrence: Option<String>,
    pub natural_string: String,
}

impl ExtendedEvent {
    pub fn on_day<Tz: TimeZone>(event: &VEvent, query_date: &DateTime<Tz>) -> Option<Self> {
        if !event.has_rrule && event.starts_at.date_naive() != query_date.date_naive() {
            log::warn!(
                "Event {} does not have a recurrence rule and is not on the requested date",
                event.uid
            );
            return None;
        }
        let base = query_date.beginning_of_day();
        let (starts_at, ends_at) = event.get_start_end_for_date(&base);
        if starts_at > base && starts_at < query_date.end_of_day() {
            Some(Self {
                query_date: query_date.to_utc(),
                event: event.clone(),
                starts_at: starts_at.to_utc(),
                ends_at: ends_at.to_utc(),
                natural_recurrence: None,
                natural_string: event.to_input(query_date),
            })
        } else {
            None
        }
    }
}
