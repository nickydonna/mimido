use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use icalendar::DatePerhapsTime;

pub(crate) mod component_props;
pub(crate) mod date_parser;
pub(crate) mod event_creator;
pub(crate) mod event_status;
pub(crate) mod event_type;
pub(crate) mod rrule_parser;

pub struct PropertyMatch<T: Sized> {
    property: T,
    start_end: Option<(usize, usize)>,
}

impl<T: Sized> PropertyMatch<T> {
    pub fn default(property: T) -> Self {
        PropertyMatch {
            property,
            start_end: None,
        }
    }

    pub fn new(property: T, start: usize, end: usize) -> Self {
        PropertyMatch {
            property,
            start_end: Some((start, end)),
        }
    }
}

trait ExtractableFromInput {
    fn extract_from_input(
        date_of_input: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<PropertyMatch<Self>, String>
    where
        Self: Sized;
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
