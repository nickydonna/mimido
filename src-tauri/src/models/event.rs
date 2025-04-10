use crate::{
    calendar_items::{
        component_props::{
            get_int_property, get_property_or_default, get_string_property, ComponentProps,
        },
        date_from_calendar_to_utc,
        event_status::EventStatus,
        event_type::EventType,
        rrule_parser::RRuleParser,
    },
    schema::*,
};
use chrono::{DateTime, Days, NaiveDateTime, Utc};
use diesel::prelude::*;
use icalendar::{Component, DatePerhapsTime};
use libdav::FetchedResource;
use now::DateTimeNow;
use rrule::{RRuleError, RRuleSet};

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

pub trait EventTrait {
    fn get_ical_data(&self) -> String;
    fn parse_ical_data(&self) -> Result<icalendar::Event, String> {
        let cal: icalendar::Calendar = self.get_ical_data().parse()?;
        let events = cal
            .components
            .into_iter()
            .filter_map(|f| f.as_event().cloned())
            .collect::<Vec<icalendar::Event>>();
        events
            .first()
            .cloned()
            .ok_or("iCal was parsed correctly but not event was found".to_string())
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
            .and_then(|r| RRuleParser::to_natural_language(&r).ok())
    }

    fn get_recurrence_for_date(&self, date: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let rule_set = self.get_rrule()?;
        let start = date.beginning_of_day();
        let end = date.end_of_day();

        let r_rule = rule_set.after(date.with_timezone(&rrule::Tz::UTC) - Days::new(1));
        r_rule
            .clone()
            .all(2)
            .dates
            .into_iter()
            .find(|d| d >= &start && d <= &end)
            .map(|d| d.to_utc())
    }
}

impl EventTrait for Event {
    fn get_ical_data(&self) -> String {
        self.ical_data.clone()
    }
}

impl EventTrait for NewEvent {
    fn get_ical_data(&self) -> String {
        self.ical_data.clone()
    }
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
        NewEvent::new(cal_id, href, content.data.clone())
    }

    pub fn new(cal_id: i32, href: String, ical_data: String) -> Result<Option<Self>, String> {
        let calendar_item: icalendar::Calendar = ical_data.clone().parse()?;
        let timezone = calendar_item
            .get_timezone()
            .and_then(|tzid| {
                let tz: Option<chrono_tz::Tz> = tzid.parse().ok();
                tz
            })
            .unwrap_or(chrono_tz::UTC);

        let first_event = calendar_item
            .components
            .into_iter()
            .filter_map(|cmp| cmp.as_event().cloned())
            .collect::<Vec<icalendar::Event>>();

        let Some(first_event) = first_event.first() else {
            return Ok(None);
        };

        let uid = first_event.get_uid();
        let summary = first_event.get_summary().unwrap_or("[No Summary]");
        let description = first_event.get_description().map(|d| d.to_string());
        let last_modified = first_event
            .get_last_modified()
            .map(|modified| modified.timestamp())
            .unwrap_or(Utc::now().timestamp());
        let start = first_event
            .get_start()
            .and_then(|s| date_from_calendar_to_utc(s, timezone));
        let end = first_event
            .get_end()
            .and_then(|e| date_from_calendar_to_utc(e, timezone));
        let values = match (uid, start, end) {
            (Some(uid), Some(start), Some(end)) => Some((uid, start, end)),
            _ => None,
        };
        let event_type =
            get_property_or_default(first_event, ComponentProps::Type, EventType::Event);
        let tag = get_string_property(first_event, ComponentProps::Tag);
        let status =
            get_property_or_default(first_event, ComponentProps::Status, EventStatus::Todo);
        let original_text = get_string_property(first_event, ComponentProps::OriginalText);
        let importance = get_int_property(first_event, ComponentProps::Importance);
        let urgency = get_int_property(first_event, ComponentProps::Urgency);
        let load = get_int_property(first_event, ComponentProps::Load);
        let postponed = get_int_property(first_event, ComponentProps::Postponed);

        let Some((uid, starts_at, ends_at)) = values else {
            return Ok(None);
        };
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

    use super::*;

    #[test]
    fn gets_the_value() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/basic.ics");
        let ics = fs::read_to_string(d).expect("To Load file");
        let event = NewEvent::new(1, "/hello".into(), ics).unwrap().unwrap();

        let result = event.get_rrule();
        assert!(result.is_some());
        let rrule_set = result.unwrap();
        println!("{:#?}", rrule_set);
        assert_eq!(
            *rrule_set.get_dt_start(),
            Tz::UTC.with_ymd_and_hms(2024, 5, 20, 13, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_uses_correct_timezone_dst() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/with_timezone.ics");
        let ics = fs::read_to_string(d).expect("To Load file");

        let event = NewEvent::new(1, "/hello".into(), ics).unwrap().unwrap();
        let recurrence = event.get_recurrence_for_date(
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
        let event = NewEvent::new(1, "/cal".into(), ics);

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
