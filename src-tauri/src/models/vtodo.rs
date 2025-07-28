use crate::{
    caldav::Href,
    calendar_items::{
        component_props::{ComponentProps, GeneralComponentProps},
        event_status::EventStatus,
        event_type::EventType,
        input_traits::ToInput,
    },
    impl_ical_parseable,
    schema::*,
};
use anyhow::anyhow;
use chrono::{DateTime, TimeZone};
use diesel::{delete, insert_into, prelude::*, update};
use icalendar::{CalendarComponent, Component, TodoStatus};
use libdav::FetchedResource;

use super::IcalParseableTrait;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize, specta::Type)]
#[diesel(table_name = vtodos)]
pub struct VTodo {
    pub id: i32,
    pub calendar_id: i32,
    pub uid: String,
    pub href: String,
    pub ical_data: String,
    pub summary: String,
    pub description: Option<String>,
    pub completed: bool,
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
}

impl From<VTodo> for CalendarComponent {
    fn from(value: VTodo) -> Self {
        let todo = icalendar::Todo::new()
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

        todo.into()
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = vtodos)]
pub struct NewVTodo {
    pub calendar_id: i32,
    pub href: String,
    pub uid: String,
    pub ical_data: String,
    pub summary: String,
    pub description: Option<String>,
    pub completed: bool,
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

impl_ical_parseable!(VTodo);
impl_ical_parseable!(NewVTodo);

pub(crate) trait VTodoTrait: IcalParseableTrait {
    fn save(&self, conn: &mut SqliteConnection) -> anyhow::Result<VTodo>;
    fn update(&self, conn: &mut SqliteConnection, id: i32) -> anyhow::Result<VTodo>;
    fn upsert_by_href(&self, conn: &mut SqliteConnection) -> anyhow::Result<VTodo>;
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
                let href = Href(self.href.clone());
                let vevent = VTodo::by_href(conn, &href)?;
                match vevent {
                    Some(old) => self.update(conn, old.id),
                    None => self.save(conn),
                }
            }
        }

        impl ToInput for $t {
            fn to_input<Tz: TimeZone>(&self, reference_date: &DateTime<Tz>) -> String {
                format!(
                    "{} {} {}",
                    self.event_type.to_input(reference_date),
                    self.status.to_input(reference_date),
                    self.summary
                )
            }
        }
    };
}

impl_todo_trait!(VTodo);
impl_todo_trait!(NewVTodo);

impl NewVTodo {
    pub fn from_resource(
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

    pub fn from_ical_data(
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
            .into_iter()
            .filter_map(|cmp| cmp.as_todo().cloned())
            .collect::<Vec<icalendar::Todo>>();

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
        } = GeneralComponentProps::try_from(first_todo)?;

        Ok(Some(NewVTodo {
            calendar_id,
            uid: uid.to_string(),
            href: href.to_string(),
            ical_data: ical_data.to_string(),
            last_modified,
            summary: summary.to_string(),
            completed: matches!(status, EventStatus::Done),
            description,
            status,
            original_text,
            tag,
            event_type,
            importance,
            load,
            urgency,
            postponed,
            etag: etag.to_string(),
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::{fs, path::PathBuf};

    #[test]
    fn test_should_parsed_todo() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/todo.ics");
        let ics = fs::read_to_string(d).expect("To Load file");
        let todo = NewVTodo::from_ical_data(1, "test", ics.as_str(), "");

        assert!(todo.is_ok());
        let todo = todo.unwrap();
        assert!(todo.is_some());
        let todo = todo.unwrap();
        assert_eq!(todo.summary, "Yerba");

        let reference_date = Utc
            .with_ymd_and_hms(2024, 3, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&chrono_tz::Tz::UTC);

        assert_eq!(todo.to_input(&reference_date), ".task %todo Yerba");
    }
}
