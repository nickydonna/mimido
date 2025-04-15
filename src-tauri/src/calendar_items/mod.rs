use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use icalendar::DatePerhapsTime;
use input_traits::ToInput;

use crate::models::{
    event::{Event, EventTrait, NewEvent},
    todo::{NewTodo, Todo, TodoTrait},
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

impl ToInput for CalendarItem {
    fn to_input(&self, date_of_input: DateTime<chrono_tz::Tz>) -> String {
        match self {
            CalendarItem::Event(event) => event.to_input(date_of_input),
            CalendarItem::NewEvent(event) => event.to_input(date_of_input),
            CalendarItem::Todo(todo) => todo.to_input(date_of_input),
            CalendarItem::NewTodo(todo) => todo.to_input(date_of_input),
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
