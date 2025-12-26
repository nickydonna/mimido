use anyhow::anyhow;
use now::DateTimeNow;
use std::str::FromStr;
use tauri::async_runtime::spawn_blocking;

use crate::{
    calendar_items::{
        component_props::{ComponentProps, GeneralComponentProps},
        date_from_calendar_to_utc,
        event_date::EventDateInfo,
        event_status::EventStatus,
        event_tags::EventTags,
        event_type::EventType,
        event_upsert::EventUpsertInfo,
        input_traits::ToUserInput,
        parse_duration,
    },
    db_conn::DbConn,
    impl_ical_parseable,
    models::{
        FromResource,
        model_traits::{
            ByHref, ById, CalendarAndSyncStatus, DeleteAllByCalendar, DeleteById,
            ListForDayOrRecurring, SetSyncedAt,
        },
    },
    schema::*,
    util::{Href, remove_multiple_spaces},
};
use chrono::{DateTime, TimeZone, Utc};
use diesel::{delete, insert_into, prelude::*, update};
use icalendar::{CalendarComponent, Component, EventLike, Property};
use libdav::FetchedResource;
use log::warn;

use super::IcalParseableTrait;

#[derive(
    Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, serde::Serialize, specta::Type,
)]
#[diesel(table_name = vevents)]
pub struct VEvent {
    pub id: i32,
    pub calendar_id: i32,
    pub uid: String,
    pub href: Option<String>,
    pub ical_data: Option<String>,
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
    pub last_modified: Option<chrono::DateTime<Utc>>,
    pub etag: Option<String>,
    pub synced_at: Option<chrono::DateTime<Utc>>,
}

impl ById for VEvent {
    async fn by_id(conn: DbConn, id: i32) -> anyhow::Result<Option<Self>> {
        use crate::schema::vevents::dsl as event_dsl;

        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            event_dsl::vevents
                .filter(event_dsl::id.eq(id))
                .select(Self::as_select())
                .first::<Self>(conn)
                .optional()
                .map_err(anyhow::Error::new)
        })
        .await?
    }
}

impl ByHref for VEvent {
    async fn by_href(conn: DbConn, href: &Href) -> anyhow::Result<Option<Self>> {
        use crate::schema::vevents::dsl as event_dsl;
        let href_str = href.to_string();

        spawn_blocking(move || {
            let c = &mut *conn.0.lock().unwrap();
            event_dsl::vevents
                .filter(event_dsl::href.eq(href_str))
                .select(Self::as_select())
                .first::<Self>(c)
                .optional()
                .map_err(anyhow::Error::new)
        })
        .await?
    }
}

impl DeleteById for VEvent {
    async fn delete_by_id(conn: DbConn, vevent_id: i32) -> anyhow::Result<bool> {
        use crate::schema::vevents::dsl as event_dsl;

        let res = spawn_blocking(move || {
            let c = &mut *conn.0.lock().unwrap();
            delete(event_dsl::vevents)
                .filter(event_dsl::id.eq(vevent_id))
                .execute(&mut *c)
        })
        .await??;
        Ok(res > 0)
    }
}

impl DeleteAllByCalendar for VEvent {
    async fn delete_all_by_calendar(conn: DbConn, calendar_id: i32) -> anyhow::Result<()> {
        use crate::schema::vevents::dsl as event_dsl;
        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            // Clean the events from that calendar
            delete(event_dsl::vevents)
                .filter(event_dsl::calendar_id.eq(calendar_id))
                .execute(conn)
        })
        .await??;
        Ok(())
    }
}

impl ListForDayOrRecurring for VEvent {
    async fn list_for_day_or_recurring(
        conn: DbConn,
        date: DateTime<chrono::FixedOffset>,
    ) -> anyhow::Result<Vec<Self>> {
        use crate::schema::vevents::dsl as event_dsl;
        let start = date.beginning_of_day();
        let end = date.end_of_day();
        let events = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            event_dsl::vevents
                .filter(
                    event_dsl::has_rrule.eq(true).or(event_dsl::starts_at
                        .ge(start)
                        .and(event_dsl::ends_at.le(end))),
                )
                .select(VEvent::as_select())
                .load(conn)
        })
        .await??;
        Ok(events)
    }
}

impl CalendarAndSyncStatus for VEvent {
    async fn by_calendar_id_and_not_sync(
        conn: DbConn,
        calendar_id: i32,
    ) -> anyhow::Result<Vec<Self>> {
        use crate::schema::vevents::dsl as event_dsl;

        let todos = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            event_dsl::vevents
                .filter(
                    event_dsl::calendar_id
                        .eq(calendar_id)
                        .and(event_dsl::synced_at.is_null()),
                )
                .select(VEvent::as_select())
                .load(conn)
        })
        .await??;
        Ok(todos)
    }

    async fn by_calendar_id_and_modified_after(
        conn: DbConn,
        calendar_id: i32,
        synced_at: DateTime<Utc>,
    ) -> anyhow::Result<Vec<Self>> {
        use crate::schema::vevents::dsl as event_dsl;

        let events = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            event_dsl::vevents
                .filter(
                    event_dsl::calendar_id
                        .eq(calendar_id)
                        .and(event_dsl::last_modified.gt(synced_at)),
                )
                .select(VEvent::as_select())
                .load(conn)
        })
        .await??;
        Ok(events)
    }
}

impl SetSyncedAt for VEvent {
    async fn set_synced_at(&self, conn: DbConn, synced_at: DateTime<Utc>) -> anyhow::Result<()> {
        use crate::schema::vevents::dsl as vevent_dsl;

        let id = self.id;
        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();
            update(vevent_dsl::vevents)
                .filter(vevent_dsl::id.eq(id))
                .set(vevent_dsl::synced_at.eq(synced_at))
                .execute(conn)
        })
        .await??;
        Ok(())
    }
}

impl VEvent {
    /// Will try to find it, if it doesn't find it it will do nothing
    /// will return true if the vevent has been found and deleted
    pub async fn try_delete_by_href(conn: DbConn, vevent_href: &Href) -> anyhow::Result<bool> {
        match Self::by_href(conn.clone(), vevent_href).await? {
            Some(vevent) => Self::delete_by_id(conn, vevent.id).await,
            None => Ok(false),
        }
    }

    pub async fn update_status_by_id(
        conn: DbConn,
        vevent_id: i32,
        status: EventStatus,
    ) -> anyhow::Result<Option<VEvent>> {
        use crate::schema::vevents::dsl as vevents_dsl;
        let conn_2 = conn.clone();
        spawn_blocking(move || {
            let conn = &mut *conn_2.0.lock().unwrap();

            diesel::update(vevents_dsl::vevents)
                .filter(vevents_dsl::id.eq(vevent_id))
                .set(vevents_dsl::status.eq(status))
                .execute(conn)
        })
        .await??;
        Self::by_id(conn, vevent_id).await
    }

    pub fn update_from_upsert<Tz: TimeZone>(
        &self,
        input: &str,
        extracted: EventUpsertInfo<Tz>,
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
        event.tag = extracted.tag.0;
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
            .add_property(ComponentProps::XStatus, value.status)
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

        if let Some(tag) = value.tag {
            event.add_property(ComponentProps::Tag, tag.to_lowercase());
            event.add_property(ComponentProps::Categories, tag.to_uppercase());
        }

        event.into()
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = vevents)]
pub struct NewVEvent {
    pub calendar_id: i32,
    pub uid: String,
    pub href: Option<String>,
    pub ical_data: Option<String>,
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
    pub last_modified: Option<chrono::DateTime<Utc>>,
    pub etag: Option<String>,
    pub synced_at: Option<chrono::DateTime<Utc>>,
}

impl_ical_parseable!(VEvent, icalendar::Event, |f| f.as_event());
impl_ical_parseable!(NewVEvent, icalendar::Event, |f| f.as_event());

pub trait VEventTrait: IcalParseableTrait<icalendar::Event> {
    async fn create(&self, conn: DbConn) -> anyhow::Result<VEvent>;
    async fn update(&self, conn: DbConn, id: i32) -> anyhow::Result<VEvent>;
    async fn upsert_by_href(&self, conn: DbConn) -> anyhow::Result<VEvent>;
    /// Get [`Event::starts_at`]
    fn get_start(&self) -> &DateTime<Utc>;
    /// Get [`Event::ends_at`]
    fn get_end(&self) -> &DateTime<Utc>;
    fn get_start_end_for_date<Tz: TimeZone>(
        &self,
        base_date: &DateTime<Tz>,
    ) -> (DateTime<Tz>, DateTime<Tz>) {
        let start = self.get_start();
        let end = self.get_end();

        let tz = base_date.timezone();
        let val = self
            .get_next_recurrence_from_date(base_date)
            .map(|d| d.with_timezone(&tz));
        let duration = *end - *start;
        let start = match val {
            Some(date) => date,
            None => self.get_start().with_timezone(&tz),
        };
        (start.clone(), start + duration)
    }
}

macro_rules! impl_event_trait {
    ($t: ty) => {
        impl VEventTrait for $t {
            fn get_start(&self) -> &DateTime<Utc> {
                &self.starts_at
            }

            fn get_end(&self) -> &DateTime<Utc> {
                &self.ends_at
            }

            async fn create(&self, conn: DbConn) -> anyhow::Result<VEvent> {
                use crate::schema::vevents::dsl as events_dsl;
                let event = self.clone();
                let val = spawn_blocking(move || {
                    let conn = &mut *conn.0.lock().unwrap();
                    insert_into(events_dsl::vevents)
                        .values(event)
                        .returning(VEvent::as_returning())
                        .get_result(conn)
                })
                .await??;
                Ok(val)
            }
            async fn update(&self, conn: DbConn, id: i32) -> anyhow::Result<VEvent> {
                use crate::schema::vevents::dsl as events_dsl;
                let event = self.clone();
                let val = spawn_blocking(move || {
                    let conn = &mut *conn.0.lock().unwrap();

                    update(events_dsl::vevents.filter(events_dsl::id.eq(id)))
                        .set(event)
                        .returning(VEvent::as_returning())
                        .get_result(conn)
                })
                .await??;
                Ok(val)
            }
            async fn upsert_by_href(&self, conn: DbConn) -> anyhow::Result<VEvent> {
                let href = Href(
                    self.href
                        .clone()
                        .ok_or(anyhow!("$ty must have href to be update"))?,
                );
                let vevent = VEvent::by_href(conn.clone(), &href).await?;
                match vevent {
                    Some(old) => self.update(conn.clone(), old.id).await,
                    None => self.create(conn).await,
                }
            }
        }

        impl<Tz: TimeZone> ToUserInput<Tz> for $t {
            fn to_input(&self, reference_date: &DateTime<Tz>) -> String {
                let timezone = reference_date.timezone();
                let start = self.starts_at.with_timezone(&timezone);
                let end = self.ends_at.with_timezone(&timezone);
                let date_string =
                    EventDateInfo::new(start, end, self.get_rrule()).to_input(reference_date);
                let value = format!(
                    "{} {} {} {} {}",
                    self.get_type().to_input(reference_date),
                    self.get_status().to_input(reference_date),
                    self.get_summary(),
                    date_string,
                    EventTags(self.tag.clone()).to_input(reference_date),
                );
                remove_multiple_spaces(&value)
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
        if let Some(tag) = new_event.tag {
            vevent.append_property(icalendar::Property::new(ComponentProps::Tag, tag));
        }

        Ok(vevent)
    }
}

fn parse_event_start_and_end(
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

impl FromResource for NewVEvent {
    fn from_resource(
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

    fn from_ical_data(
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
            .filter_map(|cmp| cmp.as_event())
            .collect::<Vec<&icalendar::Event>>();

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
        } = GeneralComponentProps::try_from(*first_event)?;

        let (starts_at, ends_at) = parse_event_start_and_end(&calendar_item)?;

        let new_event = NewVEvent {
            calendar_id: cal_id,
            uid: uid.to_string(),
            href: Some(href.to_string()),
            ical_data: Some(ical_data.to_string()),
            starts_at,
            ends_at,
            last_modified: Some(last_modified),
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
            etag: Some(etag.to_string()),
            synced_at: Some(chrono::Utc::now()),
        };
        let rrule_str = new_event.get_rrule_from_ical().map(|r| r.to_string());
        Ok(Some(NewVEvent {
            has_rrule: rrule_str.is_some(),
            rrule_str,
            ..new_event
        }))
    }
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

        println!("{event:?}");
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
            ".b %t Work at 20/05/24 13:00-16:00 every weekday #health"
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
            ComponentProps::XStatus.as_ref(),
            event.status.as_ref()
        );
        assert_property!(
            vevent,
            ComponentProps::Type.as_ref(),
            event.event_type.as_ref()
        );
    }
}
