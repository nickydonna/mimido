use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use icalendar::{CalendarComponent, Component, DatePerhapsTime, EventLike};
use log::warn;

use crate::calendar_items::{
    component_props::ComponentProps, event_creator::EventUpsertInfo, event_status::EventStatus,
    event_type::EventType, rrule_parser::EventRecurrence,
};

pub(crate) mod component_props;
pub(crate) mod date_parser;
pub(crate) mod event_creator;
pub(crate) mod event_status;
pub(crate) mod event_type;
pub(crate) mod input_traits;
pub(crate) mod rrule_parser;

impl From<EventUpsertInfo> for CalendarComponent {
    fn from(value: EventUpsertInfo) -> Self {
        match value.date_info.0 {
            Some(date_info) => {
                let mut event = icalendar::Event::new()
                    .summary(&value.summary)
                    .starts(date_info.start)
                    .ends(date_info.end)
                    .add_property(ComponentProps::Type, value.event_type)
                    .add_property(ComponentProps::Status, value.status)
                    .add_property(ComponentProps::Load, value.load.to_string())
                    .add_property(ComponentProps::Urgency, value.urgency.to_string())
                    .add_property(ComponentProps::Importance, value.importance.to_string())
                    .done();

                if let Some(recurrence) = date_info.get_recurrence_as_cal_property() {
                    event.add_property(ComponentProps::RRule, recurrence);
                }

                event.into()
            }
            None => {
                let todo = icalendar::Todo::new().summary(&value.summary).done();
                todo.into()
            }
        }
    }
}

/// Simplified version of a [`EventUpsertInfo`] for showing to the user while creating
#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct DisplayUpsertInfo {
    pub summary: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub recurrence: Option<String>,
    pub status: EventStatus,
    pub event_type: EventType,
    pub postponed: i32,
    pub urgency: i32,
    pub load: i32,
    pub importance: i32,
}

impl From<EventUpsertInfo> for DisplayUpsertInfo {
    fn from(value: EventUpsertInfo) -> Self {
        let (starts_at, ends_at, recurrence) = value
            .date_info
            .0
            .map(|info| (Some(info.start), Some(info.end), info.recurrence))
            .unwrap_or((None, None, EventRecurrence(None)));

        Self {
            summary: value.summary,
            starts_at,
            ends_at,
            recurrence: recurrence.to_natural_language().ok(),
            status: value.status,
            event_type: value.event_type,
            postponed: value.postponed,
            urgency: value.urgency,
            load: value.load,
            importance: value.importance,
        }
    }
}

pub fn date_from_calendar_to_utc(
    original: DatePerhapsTime,
    timezone: chrono_tz::Tz,
) -> Option<DateTime<Utc>> {
    match original {
        DatePerhapsTime::DateTime(calendar_date_time) => match calendar_date_time {
            icalendar::CalendarDateTime::Floating(floating) => floating
                .and_local_timezone(timezone)
                .earliest()
                .map(|d| d.to_utc()),
            icalendar::CalendarDateTime::Utc(date_time) => Some(date_time),
            icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
                let tz: chrono_tz::Tz = tzid.parse().ok()?;
                tz.from_local_datetime(&date_time)
                    .earliest()
                    .map(|d| d.to_utc())
            }
        },
        DatePerhapsTime::Date(naive_date) => naive_date
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_local_timezone(timezone)
            .earliest()
            .map(|d| d.to_utc()),
    }
}
