use std::{str::FromStr, time::Instant};

use crate::{
    calendar_items::{
        component_props::{get_string_property, ComponentProps, GeneralComponentProps},
        date_from_calendar_to_utc,
        event_status::EventStatus,
        event_type::EventType,
        input_traits::ToInput,
        rrule_parser::EventRecurrence,
    },
    impl_ical_parseable,
    schema::*,
};
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use icalendar::{Component, DatePerhapsTime, EventLike, Property};
use libdav::FetchedResource;
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
    pub rrule_str: Option<String>,
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
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
    pub rrule_str: Option<String>,
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

impl_ical_parseable!(Event);
impl_ical_parseable!(NewEvent);

pub trait EventTrait: IcalParseableTrait {
    /// Get [`Event::starts_at`]
    fn get_start(&self) -> DateTime<Utc>;
    /// Get [`Event::ends_at`]
    fn get_end(&self) -> DateTime<Utc>;
    fn get_rrule_str(&self) -> Option<String>;
    fn get_start_end_for_date<Tz: TimeZone>(
        &self,
        base_date: DateTime<Tz>,
    ) -> (DateTime<Tz>, DateTime<Tz>) {
        let tz = base_date.timezone();
        let val = self
            .get_next_recurrence_from_date(base_date)
            .map(|d| d.with_timezone(&tz));
        let duration = self.get_end() - self.get_start();
        let start = match val {
            Some(date) => date,
            None => self.get_start().with_timezone(&tz),
        };
        (start.clone(), start + duration)
    }

    /// Parsed the recurrence of the event using the [`Event::ical_data`]
    fn get_rrule_from_ical(&self) -> Option<RRuleSet> {
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

    /// Parsed the recurrence of the event using the [`Event::ical_data`]
    fn get_rrule(&self) -> Option<RRuleSet> {
        let rrule_str = self.get_rrule_str()?;
        let rrule: Result<RRuleSet, RRuleError> = rrule_str.parse();
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

    fn to_input(&self, date_of_input: DateTime<chrono_tz::Tz>) -> String {
        let timezone = date_of_input.timezone();
        let (start, end) = self.get_start_end_for_date(date_of_input);
        let start = start.with_timezone(&timezone);
        let end = end.with_timezone(&timezone);
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
        let base = format!(
            "{} {} {} {}",
            self.get_type().to_input(date_of_input),
            self.get_status().to_input(date_of_input),
            self.get_summary(),
            date_string
        );
        let recurrence_str = self
            .get_rrule()
            .and_then(|rrule| EventRecurrence::to_natural_language(&rrule).ok());
        if let Some(recurrence_str) = recurrence_str {
            format!("{} {}", base, recurrence_str)
        } else {
            base
        }
    }
}

macro_rules! impl_event_trait {
    ($t: ty) => {
        impl EventTrait for $t {
            fn get_start(&self) -> DateTime<Utc> {
                self.starts_at
            }

            fn get_end(&self) -> DateTime<Utc> {
                self.ends_at
            }
            fn get_rrule_str(&self) -> Option<String> {
                self.rrule_str.clone()
            }
        }
    };
}

impl_event_trait!(Event);
impl_event_trait!(NewEvent);

impl TryFrom<NewEvent> for icalendar::Event {
    type Error = String;
    fn try_from(new_event: NewEvent) -> Result<Self, String> {
        if new_event.event_type == EventType::Task {
            return Err("Event can't be tasks".to_string());
        }
        let mut vevent = icalendar::Event::new();
        vevent.summary(&new_event.summary);
        vevent.starts(new_event.starts_at);
        vevent.ends(new_event.ends_at);
        if let Some(description) = new_event.description.clone() {
            vevent.description(&description);
        }
        if let Some(rrule) = new_event.get_rrule() {
            let props = format!("{}", rrule)
                .split("\n")
                .filter_map(|line| icalendar::Property::from_str(line).ok())
                .collect::<Vec<Property>>();
            for p in props {
                vevent.append_property(p);
            }
        }
        vevent.append_property(icalendar::Property::from(new_event.status));
        vevent.append_property(icalendar::Property::from(new_event.event_type));
        Ok(vevent)
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
            rrule_str: None,
            status,
            original_text,
            tag,
            event_type,
            importance,
            load,
            urgency,
            postponed,
        };
        let rrule_str = new_event.get_rrule_from_ical().map(|r| r.to_string());
        Ok(Some(NewEvent {
            has_rrule: rrule_str.is_some(),
            rrule_str,
            ..new_event
        }))
    }
}

#[cfg(test)]
mod tests {

    use std::{fs, path::PathBuf};

    use chrono::{NaiveDate, TimeZone};
    use rrule::Tz;

    use crate::calendar_items::CalendarItem;

    use super::*;

    macro_rules! assert_property {
        ($vevent:ident, $prop:expr, $expected:expr) => {
            let value = $vevent.property_value($prop).unwrap();
            assert_eq!(value, $expected);
        };
    }

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
        let (start, end) = event.get_start_end_for_date(date_of_input);
        assert_eq!(
            start,
            chrono_tz::Tz::UTC
                .with_ymd_and_hms(2025, 3, 17, 13, 0, 0)
                .unwrap()
        );

        assert_eq!(
            event.to_input(date_of_input),
            "@block %todo Work at 17/03/25 13:00-16:00 every weekday"
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
                .with_ymd_and_hms(2025, 4, 10, 8, 0, 0)
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

    #[test]
    fn test_to_vevent() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/with_timezone.ics");
        let ics = fs::read_to_string(d).expect("To Load file");
        let event = NewEvent::new_from_ical_data(1, "/cal".into(), ics);

        assert!(event.is_ok());
        let event = event.unwrap();
        assert!(event.is_some());
        let event = event.unwrap();

        println!("{:#?}", event);
        let vevent = icalendar::Event::try_from(event.clone()).unwrap();

        assert_eq!(vevent.get_summary().unwrap(), event.summary);
        assert_eq!(
            vevent.get_description().unwrap(),
            event.description.unwrap()
        );
        let start = date_from_calendar_to_utc(vevent.get_start().unwrap(), chrono_tz::UTC).unwrap();
        assert_eq!(start, event.starts_at);
        let end = date_from_calendar_to_utc(vevent.get_end().unwrap(), chrono_tz::UTC).unwrap();
        assert_eq!(end, event.ends_at);
        assert_property!(
            vevent,
            ComponentProps::RRule.as_ref(),
            "FREQ=WEEKLY;BYHOUR=10;BYMINUTE=30;BYSECOND=0;BYDAY=TH"
        );
        assert_property!(
            vevent,
            ComponentProps::Status.as_ref(),
            event.status.as_ref()
        );
        assert_property!(
            vevent,
            ComponentProps::Type.as_ref(),
            event.event_type.as_ref()
        );
    }
}
