use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use icalendar::DatePerhapsTime;

use crate::models::{event::EventTrait, todo::TodoTrait};

pub(crate) mod component_props;
pub(crate) mod date_parser;
pub(crate) mod event_creator;
pub(crate) mod event_status;
pub(crate) mod event_type;
pub(crate) mod rrule_parser;

pub(crate) enum CalendarItem<E: EventTrait, T: TodoTrait> {
    Event(E),
    Todo(T),
}

impl<E: EventTrait, T: TodoTrait> ToInput for CalendarItem<E, T> {
    fn to_input(&self, date_of_input: DateTime<chrono_tz::Tz>) -> String {
        let timezone = date_of_input.timezone();
        match self {
            CalendarItem::Event(event) => {
                let start = event.get_start().with_timezone(&timezone);
                let end = event.get_end().with_timezone(&timezone);
                let date_string = if end - start < Duration::days(1) {
                    format!(
                        "at {} {}-{}",
                        start.format("%d/%m/%y"),
                        start.format("%H:%M"),
                        end.format("%H:%M")
                    )
                } else {
                    format!(
                        "at {}-{}",
                        start.format("%d/%m/%y %H:%M"),
                        end.format("%d/%m/%y %H:%M"),
                    )
                };
                format!(
                    "{} {} {} {}",
                    event.get_type(),
                    event.get_status(),
                    event.get_summary(),
                    date_string
                )
            }
            CalendarItem::Todo(todo) => format!(
                "{} {} {}",
                todo.get_type(),
                todo.get_status(),
                todo.get_summary()
            ),
        }
    }
}

trait ExtractableFromInput {
    fn extract_from_input(
        date_of_input: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<(Self, String), String>
    where
        Self: Sized;
}

trait ToInput {
    fn to_input(&self, date_of_input: DateTime<chrono_tz::Tz>) -> String;
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
