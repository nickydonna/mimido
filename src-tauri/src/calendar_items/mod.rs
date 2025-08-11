use chrono::{DateTime, NaiveTime, TimeDelta, TimeZone, Utc};
use icalendar::{CalendarComponent, Component, DatePerhapsTime, EventLike};
use log::warn;

use crate::calendar_items::{
    component_props::ComponentProps, event_date::EventRecurrence, event_status::EventStatus,
    event_type::EventType, event_upsert::EventUpsertInfo,
};

pub(crate) mod component_props;
pub(crate) mod event_date;
pub(crate) mod event_status;
pub(crate) mod event_tags;
pub(crate) mod event_type;
pub(crate) mod event_upsert;
pub(crate) mod input_traits;

impl<Tz: TimeZone> From<EventUpsertInfo<Tz>> for CalendarComponent {
    fn from(value: EventUpsertInfo<Tz>) -> Self {
        match value.event_type {
            EventType::Event | EventType::Block => match value.date_info.0 {
                Some(date_info) => {
                    let mut event = icalendar::Event::new()
                        .summary(&value.summary)
                        .starts(date_info.start.to_utc())
                        .ends(date_info.get_end_or_default(value.event_type).to_utc())
                        .add_property(ComponentProps::Type, value.event_type)
                        .add_property(ComponentProps::XStatus, value.status)
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
                    warn!("Event is {} with no date", value.event_type);
                    let todo = icalendar::Todo::new().summary(&value.summary).done();
                    todo.into()
                }
            },
            EventType::Reminder | EventType::Task => {
                let mut todo = icalendar::Todo::new()
                    .summary(&value.summary)
                    .add_property(ComponentProps::Type, value.event_type)
                    .add_property(ComponentProps::XStatus, value.status)
                    .add_property(ComponentProps::Load, value.load.to_string())
                    .add_property(ComponentProps::Urgency, value.urgency.to_string())
                    .add_property(ComponentProps::Importance, value.importance.to_string())
                    .done();

                if let Some(date_info) = value.date_info.0 {
                    todo.starts(date_info.start.to_utc());
                    todo.due(date_info.get_end_or_default(value.event_type).to_utc());

                    if let Some(recurrence) = date_info.get_recurrence_as_cal_property() {
                        todo.add_property(ComponentProps::RRule, recurrence);
                    }
                }

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
    pub tag: Option<String>,
}

impl<Tz: TimeZone> From<EventUpsertInfo<Tz>> for DisplayUpsertInfo {
    fn from(value: EventUpsertInfo<Tz>) -> Self {
        let (starts_at, ends_at, recurrence) = value
            .date_info
            .0
            .map(|info| {
                (
                    Some(info.start.clone().to_utc()),
                    Some(info.get_end_or_default(value.event_type).clone().to_utc()),
                    info.recurrence,
                )
            })
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
            tag: value.tag.0,
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

pub fn parse_duration(duration_str: &str) -> Option<TimeDelta> {
    let dur = duration_str.parse::<iso8601::Duration>().ok()?;
    let chrono_d = match dur {
        iso8601::Duration::YMDHMS {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
        } => {
            if year > 0 || month > 0 {
                warn!("Duration had month ({month}) and year ({year})");
            }
            TimeDelta::milliseconds(millisecond as i64)
                + TimeDelta::seconds(second as i64)
                + TimeDelta::minutes(minute as i64)
                + TimeDelta::hours(hour as i64)
                + TimeDelta::days(day as i64)
        }
        iso8601::Duration::Weeks(w) => TimeDelta::weeks(w as i64),
    };
    Some(chrono_d)
}
