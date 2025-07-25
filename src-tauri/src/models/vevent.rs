use anyhow::anyhow;
use std::str::FromStr;

use crate::{
    caldav::Href,
    calendar_items::{
        component_props::{get_string_property, ComponentProps, GeneralComponentProps},
        date_from_calendar_to_utc,
        date_parser::start_end_to_natural,
        event_creator::EventUpsertInfo,
        event_status::EventStatus,
        event_type::EventType,
        input_traits::ToInput,
        rrule_parser::EventRecurrence,
    },
    impl_ical_parseable,
    schema::*,
};
use chrono::{DateTime, NaiveDateTime, TimeDelta, TimeZone, Utc};
use diesel::{delete, insert_into, prelude::*, update};
use icalendar::{CalendarComponent, Component, DatePerhapsTime, EventLike, Property};
use libdav::FetchedResource;
use log::warn;
use rrule::{RRuleError, RRuleSet};

use super::IcalParseableTrait;

#[derive(
    Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, serde::Serialize, specta::Type,
)]
#[diesel(table_name = vevents)]
pub struct VEvent {
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
    pub etag: String,
}

impl VEvent {
    pub fn by_href(
        conn: &mut SqliteConnection,
        vevent_href: &Href,
    ) -> anyhow::Result<Option<Self>> {
        use crate::schema::vevents::dsl as event_dsl;

        event_dsl::vevents
            .filter(event_dsl::href.eq(&vevent_href.0))
            .select(Self::as_select())
            .first::<Self>(conn)
            .optional()
            .map_err(anyhow::Error::new)
    }

    pub fn by_id(conn: &mut SqliteConnection, id: i32) -> anyhow::Result<Option<Self>> {
        use crate::schema::vevents::dsl as event_dsl;

        event_dsl::vevents
            .filter(event_dsl::id.eq(id))
            .select(Self::as_select())
            .first::<Self>(conn)
            .optional()
            .map_err(anyhow::Error::new)
    }

    pub fn delete_all(conn: &mut SqliteConnection, calendar_id: i32) -> anyhow::Result<()> {
        use crate::schema::vevents::dsl as event_dsl;
        // Clean the events from that calendar
        delete(event_dsl::vevents)
            .filter(event_dsl::calendar_id.eq(calendar_id))
            .execute(conn)?;
        Ok(())
    }

    /// will return true if the vevent has been found and deleted
    pub fn delete_by_id(conn: &mut SqliteConnection, vevent_id: i32) -> anyhow::Result<bool> {
        use crate::schema::vevents::dsl as event_dsl;
        let res = delete(event_dsl::vevents)
            .filter(event_dsl::id.eq(vevent_id))
            .execute(conn)?;
        Ok(res > 0)
    }

    /// Will try to find it, if it doesn't find it it will do nothing
    /// will return true if the vevent has been found and deleted
    pub fn try_delete_by_href(
        conn: &mut SqliteConnection,
        vevent_href: &Href,
    ) -> anyhow::Result<bool> {
        match Self::by_href(conn, vevent_href)? {
            Some(vevent) => Self::delete_by_id(conn, vevent.id),
            None => Ok(false),
        }
    }

    pub fn update_from_upsert(
        &self,
        input: &str,
        extracted: EventUpsertInfo,
    ) -> anyhow::Result<Self> {
        let mut event = self.clone();
        let date_info = extracted.date_info.0.ok_or(anyhow!("Event need dates"))?;
        event.starts_at = date_info.start.to_utc();
        event.ends_at = date_info.get_end_or_default(extracted.event_type).to_utc();
        event.event_type = extracted.event_type;
        event.status = extracted.status;
        event.postponed = extracted.postponed;
        event.urgency = extracted.urgency;
        event.load = extracted.load;
        event.importance = extracted.importance;
        event.summary = extracted.summary;
        event.original_text = Some(input.to_string());
        Ok(event)
    }
}

impl From<VEvent> for CalendarComponent {
    fn from(value: VEvent) -> Self {
        let mut event = icalendar::Event::new()
            .summary(&value.summary)
            .starts(value.starts_at)
            .ends(value.ends_at)
            .uid(&value.uid)
            .add_property(ComponentProps::Type, value.event_type)
            .add_property(ComponentProps::Status, value.status)
            .add_property(ComponentProps::Load, value.load.to_string())
            .add_property(ComponentProps::Urgency, value.urgency.to_string())
            .add_property(ComponentProps::Importance, value.importance.to_string())
            .done();

        if let Some(rule) = value
            .get_rrule()
            .and_then(|r| r.get_rrule().first().map(|f| f.to_string()))
        {
            event.add_property(ComponentProps::RRule, rule);
        }

        event.into()
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = vevents)]
pub struct NewVEvent {
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
    pub etag: String,
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

impl_ical_parseable!(VEvent);
impl_ical_parseable!(NewVEvent);

pub trait VEventTrait: IcalParseableTrait {
    fn save(&self, conn: &mut SqliteConnection) -> anyhow::Result<VEvent>;
    fn update(&self, conn: &mut SqliteConnection, id: i32) -> anyhow::Result<VEvent>;
    fn upsert_by_href(&self, conn: &mut SqliteConnection) -> anyhow::Result<VEvent>;
    /// Get [`Event::starts_at`]
    fn get_start(&self) -> DateTime<Utc>;
    /// Get [`Event::ends_at`]
    fn get_end(&self) -> DateTime<Utc>;
    fn get_rrule_str(&self) -> Option<String>;
    fn get_start_end_for_date<Tz: TimeZone>(
        &self,
        base_date: &DateTime<Tz>,
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

    fn get_recurrence_natural(&self) -> Option<String> {
        EventRecurrence(self.get_rrule()).to_natural_language().ok()
    }

    fn get_next_recurrence_from_date<Tz: TimeZone>(
        &self,
        date: &DateTime<Tz>,
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

macro_rules! impl_event_trait {
    ($t: ty) => {
        impl VEventTrait for $t {
            fn get_start(&self) -> DateTime<Utc> {
                self.starts_at
            }

            fn get_end(&self) -> DateTime<Utc> {
                self.ends_at
            }
            fn get_rrule_str(&self) -> Option<String> {
                self.rrule_str.clone()
            }

            fn save(&self, conn: &mut SqliteConnection) -> anyhow::Result<VEvent> {
                use crate::schema::vevents::dsl as events_dsl;
                let val = insert_into(events_dsl::vevents)
                    .values(self)
                    .returning(VEvent::as_returning())
                    .get_result(conn)?;
                Ok(val)
            }
            fn update(&self, conn: &mut SqliteConnection, id: i32) -> anyhow::Result<VEvent> {
                use crate::schema::vevents::dsl as events_dsl;
                let val = update(events_dsl::vevents.filter(events_dsl::id.eq(id)))
                    .set(self)
                    .returning(VEvent::as_returning())
                    .get_result(conn)?;
                Ok(val)
            }
            fn upsert_by_href(&self, conn: &mut SqliteConnection) -> anyhow::Result<VEvent> {
                let href = Href(self.href.clone());
                let vevent = VEvent::by_href(conn, &href)?;
                match vevent {
                    Some(old) => self.update(conn, old.id),
                    None => self.save(conn),
                }
            }
        }

        impl ToInput for $t {
            fn to_input<Tz: TimeZone>(&self, reference_date: &DateTime<Tz>) -> String {
                let timezone = reference_date.timezone();
                let (start, end) = self.get_start_end_for_date(reference_date);
                let start = start.with_timezone(&timezone);
                let end = end.with_timezone(&timezone);
                let date_string = start_end_to_natural(&reference_date, &start, &end);
                let base = format!(
                    "{} {} {} {}",
                    self.get_type().to_input(reference_date),
                    self.get_status().to_input(reference_date),
                    self.get_summary(),
                    date_string
                );
                let recurrence_str = self.get_recurrence_natural();
                if let Some(recurrence_str) = recurrence_str {
                    format!("{base} {recurrence_str}")
                } else {
                    base
                }
            }
        }
    };
}

impl_event_trait!(VEvent);
impl_event_trait!(NewVEvent);

impl TryFrom<NewVEvent> for icalendar::Event {
    type Error = String;
    fn try_from(new_event: NewVEvent) -> Result<Self, String> {
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
            let props = format!("{rrule}")
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
) -> anyhow::Result<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)> {
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
        .ok_or(anyhow!("No event component"))?;

    if event.get_start().is_none() {
        warn!("No start {event:?}");
    }

    let start = event
        .get_start()
        .and_then(|s| date_from_calendar_to_utc(s, timezone))
        .ok_or(anyhow!("Missing start date {calendar:?}"))?;

    let end = if let Some(end) = event.get_end() {
        date_from_calendar_to_utc(end, timezone)
    } else {
        event
            .property_value(ComponentProps::Duration.as_ref())
            .and_then(parse_duration)
            .map(|dur| start + dur)
    }
    .ok_or(anyhow!("Missing end date {calendar:?}"))?;
    Ok((start, end))
}

impl NewVEvent {
    pub fn from_resource(
        cal_id: i32,
        fetched_resource: &FetchedResource,
    ) -> anyhow::Result<Option<Self>> {
        let href = &fetched_resource.href;
        let content = fetched_resource
            .content
            .as_ref()
            .map_err(|e| anyhow!("Resource returned {e}"))?;
        NewVEvent::from_ical_data(cal_id, href, &content.data, &content.etag)
    }

    pub fn from_ical_data(
        cal_id: i32,
        href: &str,
        ical_data: &str,
        etag: &str,
    ) -> anyhow::Result<Option<Self>> {
        let calendar_item: icalendar::Calendar = ical_data
            .parse()
            .map_err(|s| anyhow!("Error parsing calendar data {s}"))?;
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

        let new_event = NewVEvent {
            calendar_id: cal_id,
            uid: uid.to_string(),
            href: href.to_string(),
            ical_data: ical_data.to_string(),
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
            etag: etag.to_string(),
        };
        let rrule_str = new_event.get_rrule_from_ical().map(|r| r.to_string());
        Ok(Some(NewVEvent {
            has_rrule: rrule_str.is_some(),
            rrule_str,
            ..new_event
        }))
    }
}

fn parse_duration(duration_str: &str) -> Option<TimeDelta> {
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

#[cfg(test)]
mod tests {

    use super::*;
    use chrono::{NaiveDate, TimeZone};
    use rrule::Tz;
    use std::{fs, path::PathBuf};

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
        let event = NewVEvent::from_ical_data(1, "/hello", ics.as_str(), "")
            .unwrap()
            .unwrap();

        let rrule_set = event.get_rrule().unwrap();
        assert_eq!(
            *rrule_set.get_dt_start(),
            Tz::UTC.with_ymd_and_hms(2024, 5, 20, 13, 0, 0).unwrap()
        );
        let date_of_input = chrono_tz::Tz::UTC
            .with_ymd_and_hms(2025, 3, 15, 12, 0, 0)
            .unwrap();
        let (start, _) = event.get_start_end_for_date(&date_of_input);
        assert_eq!(
            start,
            chrono_tz::Tz::UTC
                .with_ymd_and_hms(2025, 3, 17, 13, 0, 0)
                .unwrap()
        );

        assert_eq!(
            event.to_input(&date_of_input),
            ".block %todo Work at 17/03/25 13:00-16:00 every weekday"
        );
    }

    #[test]
    fn test_uses_correct_timezone_dst() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/with_timezone.ics");
        let ics = fs::read_to_string(d).expect("To Load file");

        let event = NewVEvent::from_ical_data(1, "/hello", ics.as_str(), "")
            .unwrap()
            .unwrap();
        let recurrence = event
            .get_next_recurrence_from_date(
                &Tz::America__Buenos_Aires
                    .with_ymd_and_hms(2025, 4, 10, 8, 0, 0)
                    .unwrap()
                    .to_utc(),
            )
            .unwrap();
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
        let event = NewVEvent::from_ical_data(1, "/cal", ics.as_str(), "");

        let event = event.unwrap().unwrap();

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
        let event = NewVEvent::from_ical_data(1, "/cal", ics.as_str(), "");

        let event = event.unwrap().unwrap();

        let vevent = icalendar::Event::try_from(event.clone()).unwrap();

        assert_eq!(vevent.get_summary().unwrap(), event.summary);
        assert_eq!(
            vevent.get_description().unwrap(),
            event.description.unwrap()
        );
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
