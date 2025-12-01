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
    impl_ical_parseable,
    models::FromResource,
    schema::*,
    util::{Href, remove_multiple_spaces},
};
use anyhow::anyhow;
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use diesel::{delete, insert_into, prelude::*, update};
use icalendar::{CalendarComponent, Component, TodoStatus};
use libdav::FetchedResource;

use super::IcalParseableTrait;

#[derive(
    Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize, specta::Type, Clone,
)]
#[diesel(table_name = vtodos)]
pub struct VTodo {
    pub id: i32,
    pub calendar_id: i32,
    pub uid: String,
    pub href: Option<String>,
    pub ical_data: Option<String>,
    pub summary: String,
    pub description: Option<String>,
    pub starts_at: Option<chrono::DateTime<Utc>>,
    pub ends_at: Option<chrono::DateTime<Utc>>,
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
    pub etag: Option<String>,
    pub synced_at: i64,
    pub completed: Option<chrono::DateTime<Utc>>,
}

impl VTodo {
    pub fn by_href(conn: &mut SqliteConnection, vtodo_href: &Href) -> anyhow::Result<Option<Self>> {
        use crate::schema::vtodos::dsl as todo_dsl;

        todo_dsl::vtodos
            .filter(todo_dsl::href.eq(&vtodo_href.0))
            .select(Self::as_select())
            .first::<Self>(conn)
            .optional()
            .map_err(anyhow::Error::new)
    }

    pub fn by_id(conn: &mut SqliteConnection, id: i32) -> anyhow::Result<Option<Self>> {
        use crate::schema::vtodos::dsl as todo_dsl;

        todo_dsl::vtodos
            .filter(todo_dsl::id.eq(id))
            .select(Self::as_select())
            .first::<Self>(conn)
            .optional()
            .map_err(anyhow::Error::new)
    }

    pub fn delete_all(conn: &mut SqliteConnection, calendar_id: i32) -> anyhow::Result<()> {
        use crate::schema::vtodos::dsl as todo_dsl;
        delete(todo_dsl::vtodos)
            .filter(todo_dsl::calendar_id.eq(calendar_id))
            .execute(conn)?;
        Ok(())
    }

    /// will return true if the vevent has been found and deleted
    pub fn delete_by_id(conn: &mut SqliteConnection, vevent_id: i32) -> anyhow::Result<bool> {
        use crate::schema::vtodos::dsl as todo_dsl;
        let res = delete(todo_dsl::vtodos)
            .filter(todo_dsl::id.eq(vevent_id))
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

    pub fn update_from_upsert<Tz: TimeZone>(
        &self,
        input: &str,
        extracted: EventUpsertInfo<Tz>,
        date_of_update: DateTime<Tz>,
    ) -> anyhow::Result<Self> {
        let mut todo = self.clone();
        if let Some(date_info) = extracted.date_info.0 {
            todo.starts_at = Some(date_info.start.to_utc());
            todo.ends_at = Some(date_info.get_end_or_default(extracted.event_type).to_utc());
        }
        todo.event_type = extracted.event_type;
        todo.status = extracted.status;
        todo.postponed = extracted.postponed;
        todo.urgency = extracted.urgency;
        todo.load = extracted.load;
        todo.importance = extracted.importance;
        todo.summary = extracted.summary;
        todo.tag = extracted.tag.0;
        todo.original_text = Some(input.to_string());
        // Add completed if task was not completed
        if self.status != extracted.status && matches!(extracted.status, EventStatus::Done) {
            todo.completed = Some(date_of_update.to_utc())
        }
        // Remove completed if task is not completed
        if !matches!(extracted.status, EventStatus::Done) && self.completed.is_some() {
            todo.completed = None
        }
        Ok(todo)
    }
}

impl From<VTodo> for CalendarComponent {
    fn from(value: VTodo) -> Self {
        let mut todo = icalendar::Todo::new()
            .summary(&value.summary)
            .uid(&value.uid)
            .add_property(ComponentProps::Type, value.event_type)
            .add_property(ComponentProps::XStatus, value.status)
            .add_property(ComponentProps::Load, value.load.to_string())
            .add_property(ComponentProps::Urgency, value.urgency.to_string())
            .add_property(ComponentProps::Importance, value.importance.to_string())
            .status(match value.status {
                EventStatus::Done => TodoStatus::Completed,
                EventStatus::InProgress => TodoStatus::InProcess,
                _ => TodoStatus::NeedsAction,
            })
            .percent_complete(match value.status {
                EventStatus::Backlog => 0,
                EventStatus::Todo => 0,
                EventStatus::InProgress => 1,
                EventStatus::Done => 100,
            })
            .done();

        if let Some(tag) = value.tag {
            todo.add_property(ComponentProps::Tag, tag.to_lowercase());
            todo.add_property(ComponentProps::Categories, tag.to_uppercase());
        }

        if let Some(completed_date) = value.completed {
            todo.completed(completed_date);
        }

        todo.into()
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = vtodos)]
pub struct NewVTodo {
    pub calendar_id: i32,
    pub uid: String,
    pub href: Option<String>,
    pub ical_data: Option<String>,
    pub summary: String,
    pub description: Option<String>,
    pub starts_at: Option<chrono::DateTime<Utc>>,
    pub ends_at: Option<chrono::DateTime<Utc>>,
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
    pub etag: Option<String>,
    pub synced_at: i64,
    pub completed: Option<chrono::DateTime<Utc>>,
}

impl_ical_parseable!(VTodo, icalendar::Todo, |f| f.as_todo());
impl_ical_parseable!(NewVTodo, icalendar::Todo, |f| f.as_todo());

pub(crate) trait VTodoTrait: IcalParseableTrait<icalendar::Todo> {
    fn save(&self, conn: &mut SqliteConnection) -> anyhow::Result<VTodo>;
    fn update(&self, conn: &mut SqliteConnection, id: i32) -> anyhow::Result<VTodo>;
    fn upsert_by_href(&self, conn: &mut SqliteConnection) -> anyhow::Result<VTodo>;
    /// Get [`Event::starts_at`]
    fn get_start(&self) -> Option<&DateTime<Utc>>;
    /// Get [`Event::ends_at`]
    fn get_end(&self) -> Option<&DateTime<Utc>>;
    fn get_start_end_for_date<Tz: TimeZone>(
        &self,
        base_date: &DateTime<Tz>,
    ) -> Option<(DateTime<Tz>, DateTime<Tz>)> {
        let start = self.get_start()?;
        let end = self.get_end()?;
        let tz = base_date.timezone();
        let val = self
            .get_next_recurrence_from_date(base_date)
            .map(|d| d.with_timezone(&tz));
        let duration = *end - *start;
        let start = match val {
            Some(date) => date,
            None => start.with_timezone(&tz),
        };
        Some((start.clone(), start + duration))
    }
}

macro_rules! impl_todo_trait {
    ($t: ty) => {
        impl VTodoTrait for $t {
            fn save(&self, conn: &mut SqliteConnection) -> anyhow::Result<VTodo> {
                use crate::schema::vtodos::dsl as todo_dsl;
                let val = insert_into(todo_dsl::vtodos)
                    .values(self)
                    .returning(VTodo::as_returning())
                    .get_result(conn)?;
                Ok(val)
            }
            fn update(&self, conn: &mut SqliteConnection, id: i32) -> anyhow::Result<VTodo> {
                use crate::schema::vtodos::dsl as todo_dsl;
                let val = update(todo_dsl::vtodos.filter(todo_dsl::id.eq(id)))
                    .set(self)
                    .returning(VTodo::as_returning())
                    .get_result(conn)?;
                Ok(val)
            }

            fn upsert_by_href(&self, conn: &mut SqliteConnection) -> anyhow::Result<VTodo> {
                let href = Href(self.href.clone().expect("$t must have href"));
                let vevent = VTodo::by_href(conn, &href)?;
                match vevent {
                    Some(old) => self.update(conn, old.id),
                    None => self.save(conn),
                }
            }

            fn get_start(&self) -> Option<&DateTime<Utc>> {
                self.starts_at.as_ref()
            }

            fn get_end(&self) -> Option<&DateTime<Utc>> {
                self.ends_at.as_ref()
            }
        }

        impl<Tz: TimeZone> ToUserInput<Tz> for $t {
            fn to_input(&self, reference_date: &DateTime<Tz>) -> String {
                let timezone = reference_date.timezone();
                let Some(start) = self.starts_at else {
                    let value = format!(
                        "{} {} {} {}",
                        self.event_type.to_input(reference_date),
                        self.status.to_input(reference_date),
                        EventTags(self.tag.clone()).to_input(reference_date),
                        self.summary
                    );

                    return remove_multiple_spaces(&value);
                };
                let end = self
                    .ends_at
                    .unwrap_or(start + TimeDelta::minutes(15))
                    .with_timezone(&timezone);
                let start = start.with_timezone(&timezone);
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
                remove_multiple_spaces(&value).trim().to_string()
            }
        }
    };
}

impl_todo_trait!(VTodo);
impl_todo_trait!(NewVTodo);

impl FromResource for NewVTodo {
    fn from_resource(
        calendar_id: i32,
        fetched_resource: &FetchedResource,
    ) -> anyhow::Result<Option<NewVTodo>> {
        let href = &fetched_resource.href;
        let content = fetched_resource
            .content
            .as_ref()
            .map_err(|e| anyhow!("Resource returned {e}"))?;
        NewVTodo::from_ical_data(calendar_id, href, &content.data, &content.etag)
    }

    fn from_ical_data(
        calendar_id: i32,
        href: &str,
        ical_data: &str,
        etag: &str,
    ) -> anyhow::Result<Option<Self>> {
        let calendar_item: icalendar::Calendar = ical_data
            .parse()
            .map_err(|s| anyhow!("Error parsing calendar data {s}"))?;
        let first_todo = calendar_item
            .components
            .iter()
            .filter_map(|cmp| cmp.as_todo())
            .collect::<Vec<&icalendar::Todo>>();

        let Some(first_todo) = first_todo.first() else {
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
        } = GeneralComponentProps::try_from(*first_todo)?;

        let (starts_at, ends_at) = match parse_todo_start_and_end(&calendar_item)? {
            Some((start, end)) => (Some(start), Some(end)),
            None => (None, None),
        };

        let new_todo = NewVTodo {
            calendar_id,
            uid: uid.to_string(),
            href: Some(href.to_string()),
            ical_data: Some(ical_data.to_string()),
            last_modified,
            summary: summary.to_string(),
            completed: None,
            description,
            status,
            original_text,
            tag,
            event_type,
            importance,
            load,
            urgency,
            postponed,
            etag: Some(etag.to_string()),
            synced_at: chrono::Utc::now().timestamp(),
            has_rrule: false,
            rrule_str: None,
            starts_at,
            ends_at,
        };

        let rrule_str = new_todo.get_rrule_from_ical().map(|r| r.to_string());
        Ok(Some(NewVTodo {
            has_rrule: rrule_str.is_some(),
            rrule_str,
            ..new_todo
        }))
    }
}

fn parse_todo_start_and_end(
    calendar: &icalendar::Calendar,
) -> anyhow::Result<Option<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)>> {
    let timezone = calendar
        .get_timezone()
        .and_then(|tzid| {
            let tz: Option<chrono_tz::Tz> = tzid.parse().ok();
            tz
        })
        .unwrap_or(chrono_tz::UTC);

    let todo = calendar
        .components
        .iter()
        .filter_map(|cmp| cmp.as_todo().cloned())
        .next()
        .ok_or(anyhow!("No todo component"))?;

    let start = todo
        .get_start()
        .and_then(|s| date_from_calendar_to_utc(s, timezone));

    let Some(start) = start else {
        return Ok(None);
    };

    let end = todo.get_end().or(todo.get_due());
    let end = if let Some(end) = end {
        date_from_calendar_to_utc(end, timezone)
    } else {
        todo.property_value(ComponentProps::Duration.as_ref())
            .and_then(parse_duration)
            .map(|dur| start + dur)
    }
    .unwrap_or_else(|| start + TimeDelta::minutes(15));
    Ok(Some((start, end)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::{fs, path::PathBuf};

    fn load_file(path: &str) -> String {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push(path);
        fs::read_to_string(d).expect("To Load file")
    }

    #[test]
    fn test_should_parsed_todo() {
        let ics = load_file("./fixtures/todo.ics");
        let todo = NewVTodo::from_ical_data(1, "test", ics.as_str(), "");

        let todo = todo.unwrap();
        let todo = todo.unwrap();
        assert_eq!(todo.summary, "Yerba");
        assert_eq!(todo.starts_at, None);
        assert_eq!(todo.ends_at, None);

        let reference_date = Utc
            .with_ymd_and_hms(2024, 3, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&chrono_tz::Tz::UTC);

        // Is done because the ICS is completed
        assert_eq!(todo.to_input(&reference_date), ".t %d Yerba");
    }

    #[test]
    fn test_should_parsed_todo_with_date() {
        let ics = load_file("./fixtures/todo_date.ics");
        let todo = NewVTodo::from_ical_data(1, "test", ics.as_str(), "");

        let todo = todo.unwrap();
        let todo = todo.unwrap();
        let reference_date = Utc
            .with_ymd_and_hms(2024, 3, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&chrono_tz::Tz::UTC);

        let (start, end) = todo.get_start_end_for_date(&reference_date).unwrap();
        assert_eq!(todo.summary, "Yerba");
        assert_eq!(
            start,
            chrono_tz::Tz::UTC
                .with_ymd_and_hms(2024, 5, 20, 13, 0, 0)
                .unwrap()
        );
        assert_eq!(
            end,
            chrono_tz::Tz::UTC
                .with_ymd_and_hms(2024, 5, 20, 16, 0, 0)
                .unwrap()
        );

        // Is done because the ICS is completed
        assert_eq!(
            todo.to_input(&reference_date),
            ".t %d Yerba at 20/05/24 13:00-16:00"
        );
    }
}
