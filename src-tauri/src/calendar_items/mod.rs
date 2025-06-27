use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use icalendar::DatePerhapsTime;

use crate::{
    calendar_items::{
        event_creator::{EventDateInfo, EventUpsertInfo},
        event_status::EventStatus,
        event_type::EventType,
    },
    models::{
        event::{Event, NewEvent},
        todo::{NewTodo, Todo},
    },
};

pub(crate) mod component_props;
pub(crate) mod date_parser;
pub(crate) mod event_creator;
pub(crate) mod event_status;
pub(crate) mod event_type;
pub(crate) mod input_traits;
pub(crate) mod rrule_parser;

pub(crate) enum CalendarItem {
    Event(Event),
    NewEvent(NewEvent),
    Todo(Todo),
    NewTodo(NewTodo),
}

impl TryFrom<CalendarItem> for icalendar::Calendar {
    type Error = String;
    fn try_from(value: CalendarItem) -> Result<Self, String> {
        let component = match value {
            CalendarItem::Event(event) => todo!(),
            CalendarItem::NewEvent(new_event) => icalendar::Event::try_from(new_event),
            CalendarItem::Todo(todo) => todo!(),
            CalendarItem::NewTodo(new_todo) => todo!(),
        }?;
        let mut cal = icalendar::Calendar::new();
        cal.push(component);
        Ok(cal)
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
    pub priority: i32,
}

impl From<EventUpsertInfo> for DisplayUpsertInfo {
    fn from(value: EventUpsertInfo) -> Self {
        let (starts_at, ends_at) = value
            .date_info
            .0
            .map(|EventDateInfo { start, end }| (Some(start), Some(end)))
            .unwrap_or((None, None));

        Self {
            summary: value.summary,
            starts_at,
            ends_at,
            recurrence: value.recurrence.to_natural_language().ok(),
            status: value.status,
            event_type: value.event_type,
            postponed: value.postponed,
            urgency: value.urgency,
            load: value.load,
            priority: value.priority,
        }
    }
}

pub fn date_from_calendar_to_utc(
    original: DatePerhapsTime,
    timezone: chrono_tz::Tz,
) -> Option<DateTime<Utc>> {
    match original {
        DatePerhapsTime::DateTime(calendar_date_time) => match calendar_date_time {
            icalendar::CalendarDateTime::Floating(_) => None,
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
