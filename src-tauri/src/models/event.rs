use crate::{
    calendar_items::{
        component_props::{get_string_property, ComponentProps, GeneralComponentProps},
        date_from_calendar_to_utc,
        event_status::EventStatus,
        event_type::EventType,
        rrule_parser::EventRecurrence,
    },
    schema::*,
};
use chrono::{DateTime, Days, NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use icalendar::{Component, DatePerhapsTime};
use libdav::FetchedResource;
use now::DateTimeNow;
use rrule::{RRuleError, RRuleSet};

use super::IcalParseableTrait;

#[derive(
    Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, serde::Serialize, specta::Type,
)]
#[diesel(table_name = events)]
pub struct Event {
    pub id: i32,
    pub calendar_id: i32,
    pub uid: String,
    pub href: String,
    pub ical_data: String,
    pub summary: String,
    pub description: Option<String>,
    pub starts_at: chrono::DateTime<Utc>,
    pub ends_at: chrono::DateTime<Utc>,
    pub has_rrule: bool,
    pub tag: Option<String>,
    pub status: EventStatus,
    pub event_type: EventType,
    pub original_text: Option<String>,
    pub load: i32,
    pub urgency: i32,
    pub importance: i32,
    pub postponed: i32,
    pub last_modified: i64,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = events)]
pub struct NewEvent {
    pub calendar_id: i32,
    pub uid: String,
    pub href: String,
    pub ical_data: String,
    pub summary: String,
    pub description: Option<String>,
    pub starts_at: chrono::DateTime<Utc>,
    pub ends_at: chrono::DateTime<Utc>,
    pub has_rrule: bool,
    pub tag: Option<String>,
    pub status: EventStatus,
    pub event_type: EventType,
    pub original_text: Option<String>,
    pub load: i32,
    pub urgency: i32,
    pub importance: i32,
    pub postponed: i32,
    pub last_modified: i64,
}

fn format_date_ical(date: NaiveDateTime) -> String {
    date.format("%Y%m%dT%H%M%S").to_string()
}

fn get_start_string(event: &icalendar::Event) -> Option<String> {
    let dt_start = event.get_start()?;

    match dt_start {
        DatePerhapsTime::DateTime(calendar_date_time) => match calendar_date_time {
            icalendar::CalendarDateTime::Floating(_) => None,
            icalendar::CalendarDateTime::Utc(date_time) =>
            // Add `Z` to the end of the date string since RRule assumes local otherwise
            {
                Some(format!(
                    "DTSTART:{}Z\n",
                    format_date_ical(date_time.naive_utc())
                ))
            }
            icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
                let date = format_date_ical(date_time);
                Some(format!("DTSTART;TZID={tzid}:{date}\n"))
            }
        },
        DatePerhapsTime::Date(naive_date) => Some(format!(
            "DTSTART:{}\n",
            format_date_ical(naive_date.and_hms_opt(0, 0, 0).unwrap())
        )),
    }
}

pub trait EventTrait: IcalParseableTrait {
    fn get_start(&self) -> DateTime<Utc>;
    fn get_end(&self) -> DateTime<Utc>;
    fn get_start_for_date<Tz: TimeZone>(&self, base_date: DateTime<Tz>) -> DateTime<Tz> {
        let tz = base_date.timezone();
        let val = self
            .get_next_recurrence_from_date(base_date)
            .map(|d| d.with_timezone(&tz));
        match val {
            Some(date) => date,
            None => self.get_start().with_timezone(&tz),
        }
    }
    fn get_end_for_date<Tz: TimeZone>(&self, base_date: DateTime<Tz>) -> DateTime<Tz> {
        let duration = self.get_end() - self.get_start();
        self.get_start_for_date(base_date) + duration
    }

    fn get_rrule(&self) -> Option<RRuleSet> {
        let event = self.parse_ical_data().ok()?;
        let rrule = get_string_property(&event, ComponentProps::RRule)?;
        let start_str = get_start_string(&event)?;

        let r_date = get_string_property(&event, ComponentProps::RDate);
        let ex_date = get_string_property(&event, ComponentProps::Exdate);
        let mut rule_set_string = format!(
            "{start_str}\
        RRULE:{rrule}"
        );

        if let Some(r_date) = r_date {
            rule_set_string = format!(
                "
        {rule_set_string}\n\
        RDATE:{r_date}"
            );
        }

        if let Some(ex_date) = ex_date {
            rule_set_string = format!(
                "
        {rule_set_string}\n\
        EXDATE:{ex_date}"
            );
        }
        let rrule: Result<RRuleSet, RRuleError> = rule_set_string.parse();
        rrule.ok()
    }

    fn get_occurrence_natural(&self) -> Option<String> {
        self.get_rrule()
            .and_then(|r| EventRecurrence::to_natural_language(&r).ok())
    }

    fn get_next_recurrence_from_date<Tz: TimeZone>(
        &self,
        date: DateTime<Tz>,
    ) -> Option<DateTime<Tz>> {
        let rule_set = self.get_rrule()?;

        let r_rule = rule_set.after(date.with_timezone(&rrule::Tz::UTC));
        r_rule
            .clone()
            .all(1)
            .dates
            .first()
            .map(|d| d.with_timezone(&date.timezone()))
    }
}

impl IcalParseableTrait for Event {
    fn get_ical_data(&self) -> String {
        self.ical_data.clone()
    }

    fn get_summary(&self) -> String {
        self.summary.clone()
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn get_postponed(&self) -> i32 {
        self.postponed
    }

    fn get_load(&self) -> i32 {
        self.load
    }

    fn get_urgency(&self) -> i32 {
        self.urgency
    }

    fn get_importance(&self) -> i32 {
        self.importance
    }
    fn get_status(&self) -> EventStatus {
        self.status
    }
    fn get_type(&self) -> EventType {
        self.event_type
    }
}
impl EventTrait for Event {
    fn get_start(&self) -> DateTime<Utc> {
        self.starts_at
    }

    fn get_end(&self) -> DateTime<Utc> {
        self.ends_at
    }
}

impl IcalParseableTrait for NewEvent {
    fn get_ical_data(&self) -> String {
        self.ical_data.clone()
    }
    fn get_summary(&self) -> String {
        self.summary.clone()
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn get_postponed(&self) -> i32 {
        self.postponed
    }

    fn get_load(&self) -> i32 {
        self.load
    }

    fn get_urgency(&self) -> i32 {
        self.urgency
    }

    fn get_importance(&self) -> i32 {
        self.importance
    }
    fn get_status(&self) -> EventStatus {
        self.status
    }
    fn get_type(&self) -> EventType {
        self.event_type
    }
}

impl EventTrait for NewEvent {
    fn get_start(&self) -> DateTime<Utc> {
        self.starts_at
    }

    fn get_end(&self) -> DateTime<Utc> {
        self.ends_at
    }
}

fn get_start_and_end(
    calendar: &icalendar::Calendar,
) -> Result<(chrono::DateTime<Utc>, chrono::DateTime<Utc>), String> {
    let timezone = calendar
        .get_timezone()
        .and_then(|tzid| {
            let tz: Option<chrono_tz::Tz> = tzid.parse().ok();
            tz
        })
        .unwrap_or(chrono_tz::UTC);

    let event = calendar
        .components
        .iter()
        .filter_map(|cmp| cmp.as_event().cloned())
        .next()
        .ok_or("No event component".to_string())?;

    let start = event
        .get_start()
        .and_then(|s| date_from_calendar_to_utc(s, timezone))
        .ok_or("Missing start date".to_string())?;
    let end = event
        .get_end()
        .and_then(|e| date_from_calendar_to_utc(e, timezone))
        .ok_or("Missing end date".to_string())?;
    Ok((start, end))
}

impl NewEvent {
    pub fn new_from_resource(
        cal_id: i32,
        fetched_resource: &FetchedResource,
    ) -> Result<Option<Self>, String> {
        let href = fetched_resource.href.clone();
        let content = fetched_resource
            .content
            .as_ref()
            .map_err(|e| e.to_string())?;
        NewEvent::new_from_ical_data(cal_id, href, content.data.clone())
    }

    pub fn new_from_ical_data(
        cal_id: i32,
        href: String,
        ical_data: String,
    ) -> Result<Option<Self>, String> {
        let calendar_item: icalendar::Calendar = ical_data.clone().parse()?;
        let first_event = calendar_item
            .components
            .iter()
            .filter_map(|cmp| cmp.as_event().cloned())
            .collect::<Vec<icalendar::Event>>();

        let Some(first_event) = first_event.first() else {
            return Ok(None);
        };

        let GeneralComponentProps {
            uid,
            summary,
            description,
            event_type,
            tag,
            urgency,
            status,
            original_text,
            importance,
            load,
            postponed,
            last_modified,
        } = GeneralComponentProps::try_from(first_event)?;

        let (starts_at, ends_at) = get_start_and_end(&calendar_item)?;

        let new_event = NewEvent {
            calendar_id: cal_id,
            uid: uid.to_string(),
            href,
            ical_data,
            starts_at,
            ends_at,
            last_modified,
            summary: summary.to_string(),
            description,
            has_rrule: false,
            status,
            original_text,
            tag,
            event_type,
            importance,
            load,
            urgency,
            postponed,
        };
        Ok(Some(NewEvent {
            has_rrule: new_event.get_rrule().is_some(),
            ..new_event
        }))
    }
}

#[cfg(test)]
mod tests {

    use std::{fs, path::PathBuf};

    use chrono::{NaiveDate, TimeZone};
    use rrule::Tz;

    use crate::{
        calendar_items::{input_traits::ToInput, CalendarItem},
        models::todo::NewTodo,
    };

    use super::*;

    #[test]
    fn gets_the_value() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/basic.ics");
        let ics = fs::read_to_string(d).expect("To Load file");
        let event = NewEvent::new_from_ical_data(1, "/hello".into(), ics)
            .unwrap()
            .unwrap();

        let result = event.get_rrule();
        assert!(result.is_some());
        let rrule_set = result.unwrap();
        assert_eq!(
            *rrule_set.get_dt_start(),
            Tz::UTC.with_ymd_and_hms(2024, 5, 20, 13, 0, 0).unwrap()
        );
        let date_of_input = chrono_tz::Tz::UTC
            .with_ymd_and_hms(2025, 3, 15, 12, 0, 0)
            .unwrap();
        assert_eq!(
            event.get_start_for_date(date_of_input),
            chrono_tz::Tz::UTC
                .with_ymd_and_hms(2025, 3, 17, 13, 0, 0)
                .unwrap()
        );

        assert_eq!(
            CalendarItem::<NewEvent, NewTodo>::Event(event).to_input(date_of_input),
            "@block %todo Work at 17/03/25 13:00-16:00"
        );
    }

    #[test]
    fn test_uses_correct_timezone_dst() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/with_timezone.ics");
        let ics = fs::read_to_string(d).expect("To Load file");

        let event = NewEvent::new_from_ical_data(1, "/hello".into(), ics)
            .unwrap()
            .unwrap();
        let recurrence = event.get_next_recurrence_from_date(
            Tz::America__Buenos_Aires
                .with_ymd_and_hms(2025, 4, 10, 13, 0, 0)
                .unwrap()
                .to_utc(),
        );
        assert!(recurrence.is_some());
        let recurrence = recurrence.unwrap();
        assert_eq!(
            recurrence,
            Tz::America__Buenos_Aires
                .with_ymd_and_hms(2025, 4, 10, 11, 30, 0)
                .unwrap()
                .to_utc()
        );
    }

    #[test]
    fn test_correct_timezone() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/with_timezone.ics");
        let ics = fs::read_to_string(d).expect("To Load file");
        let event = NewEvent::new_from_ical_data(1, "/cal".into(), ics);

        assert!(event.is_ok());
        let event = event.unwrap();
        assert!(event.is_some());
        let event = event.unwrap();

        assert!(event.has_rrule);
        assert_eq!(event.summary, "Nicky / Eric weekly sync");
        let starts_at = NaiveDate::from_ymd_opt(2025, 2, 13)
            .unwrap()
            .and_hms_opt(15, 30, 0)
            .unwrap()
            .and_utc();
        assert_eq!(event.starts_at, starts_at);
    }
}
