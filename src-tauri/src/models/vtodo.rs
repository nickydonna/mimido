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
    commands::extended_todo::UnscheduledTodo,
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
    util::{Etag, Href, remove_multiple_spaces},
};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset, TimeDelta, TimeZone, Utc};
use diesel::{delete, insert_into, prelude::*, update};
use icalendar::{CalendarComponent, Component, TodoStatus};
use libdav::FetchedResource;
use now::DateTimeNow;
use tauri::async_runtime::spawn_blocking;

use super::IcalParseableTrait;

#[derive(
    Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize, specta::Type, Clone,
)]
#[diesel(table_name = vtodos)]
#[diesel(treat_none_as_null = true)]
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
    pub last_modified: Option<chrono::DateTime<Utc>>,
    pub etag: Option<String>,
    pub synced_at: Option<chrono::DateTime<Utc>>,
    pub completed: Option<chrono::DateTime<Utc>>,
    pub out_of_sync: bool,
}

impl ById for VTodo {
    async fn by_id(conn: DbConn, id: i32) -> anyhow::Result<Option<Self>> {
        use crate::schema::vtodos::dsl as todo_dsl;

        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            todo_dsl::vtodos
                .filter(todo_dsl::id.eq(id))
                .select(Self::as_select())
                .first::<Self>(conn)
                .optional()
                .map_err(anyhow::Error::new)
        })
        .await?
    }
}

impl ByHref for VTodo {
    async fn by_href(conn: DbConn, href: &Href) -> anyhow::Result<Option<Self>> {
        use crate::schema::vtodos::dsl as todo_dsl;
        let href_str = href.to_string();

        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            todo_dsl::vtodos
                .filter(todo_dsl::href.eq(href_str))
                .select(Self::as_select())
                .first::<Self>(conn)
                .optional()
                .map_err(anyhow::Error::new)
        })
        .await?
    }
}

impl DeleteById for VTodo {
    async fn delete_by_id(conn: DbConn, vevent_id: i32) -> anyhow::Result<bool> {
        use crate::schema::vtodos::dsl as todo_dsl;
        let res = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            delete(todo_dsl::vtodos)
                .filter(todo_dsl::id.eq(vevent_id))
                .execute(conn)
        })
        .await??;
        Ok(res > 0)
    }
}

impl DeleteAllByCalendar for VTodo {
    async fn delete_all_by_calendar(conn: DbConn, calendar_id: i32) -> anyhow::Result<()> {
        use crate::schema::vtodos::dsl as todo_dsl;
        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            delete(todo_dsl::vtodos)
                .filter(todo_dsl::calendar_id.eq(calendar_id))
                .execute(conn)
        })
        .await??;
        Ok(())
    }
}

impl ListForDayOrRecurring for VTodo {
    async fn list_for_day_or_recurring(
        conn: DbConn,
        date: DateTime<FixedOffset>,
    ) -> anyhow::Result<Vec<Self>> {
        use crate::schema::vtodos::dsl as todos_dsl;
        let start = date.beginning_of_day();
        let end = date.end_of_day();
        let todos = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            todos_dsl::vtodos
                .filter(
                    todos_dsl::has_rrule.eq(true).or(todos_dsl::starts_at
                        .ge(start)
                        .and(todos_dsl::ends_at.le(end))),
                )
                .select(VTodo::as_select())
                .load(conn)
        })
        .await??;
        Ok(todos)
    }
}

impl CalendarAndSyncStatus for VTodo {
    async fn by_calendar_id_and_not_sync(
        conn: DbConn,
        calendar_id: i32,
    ) -> anyhow::Result<Vec<Self>> {
        use crate::schema::vtodos::dsl as todos_dsl;

        let todos = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            todos_dsl::vtodos
                .filter(
                    todos_dsl::calendar_id
                        .eq(calendar_id)
                        .and(todos_dsl::synced_at.is_null()),
                )
                .select(VTodo::as_select())
                .load(conn)
        })
        .await??;
        Ok(todos)
    }

    async fn by_calendar_id_and_out_of_sync(
        conn: DbConn,
        calendar_id: i32,
    ) -> anyhow::Result<Vec<Self>> {
        use crate::schema::vtodos::dsl as todos_dsl;

        let todos = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            todos_dsl::vtodos
                .filter(
                    todos_dsl::calendar_id
                        .eq(calendar_id)
                        .and(todos_dsl::out_of_sync.eq(true)),
                )
                .select(VTodo::as_select())
                .load(conn)
        })
        .await??;
        Ok(todos)
    }
}

impl SetSyncedAt for VTodo {
    async fn set_synced_at(
        self,
        conn: DbConn,
        etag: Option<Etag>,
        synced_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        use crate::schema::vtodos::dsl as todos_dsl;

        let id = self.id;
        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();
            update(todos_dsl::vtodos)
                .filter(todos_dsl::id.eq(id))
                .set((
                    todos_dsl::etag.eq(etag.map(|e| e.to_string())),
                    todos_dsl::out_of_sync.eq(false),
                    todos_dsl::synced_at.eq(synced_at),
                ))
                .execute(conn)
        })
        .await??;
        Ok(())
    }
}

impl VTodo {
    /// Will try to find it, if it doesn't find it it will do nothing
    /// will return true if the vevent has been found and deleted
    pub async fn try_delete_by_href(conn: DbConn, vtodo_href: &Href) -> anyhow::Result<bool> {
        match Self::by_href(conn.clone(), vtodo_href).await? {
            Some(vevent) => Self::delete_by_id(conn, vevent.id).await,
            None => Ok(false),
        }
    }

    pub async fn list_unscheduled(
        conn: DbConn,
        include_done: bool,
    ) -> anyhow::Result<Vec<UnscheduledTodo>> {
        use crate::schema::vtodos::dsl as todo_dsl;

        let todos = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            if include_done {
                todo_dsl::vtodos
                    .filter(todo_dsl::starts_at.is_null())
                    .select(VTodo::as_select())
                    .load(conn)
            } else {
                todo_dsl::vtodos
                    .filter(
                        todo_dsl::status
                            .is_not(EventStatus::Done)
                            .and(todo_dsl::starts_at.is_null()),
                    )
                    .select(VTodo::as_select())
                    .load(conn)
            }
        })
        .await??;

        Ok(todos
            .iter()
            .map(|t| UnscheduledTodo::on_day(t, &Utc::now()))
            .collect::<Vec<UnscheduledTodo>>())
    }

    pub async fn update_status_by_id(
        conn: DbConn,
        vtodo_id: i32,
        status: EventStatus,
        update_at: DateTime<FixedOffset>,
    ) -> anyhow::Result<Option<VTodo>> {
        use crate::schema::vtodos::dsl as vtodos_dsl;
        let conn_2 = conn.clone();
        spawn_blocking(move || {
            let conn = &mut *conn_2.0.lock().unwrap();

            let completed_update = if matches!(status, EventStatus::Done) {
                vtodos_dsl::completed.eq(Some(update_at.to_utc()))
            } else {
                vtodos_dsl::completed.eq(None)
            };

            diesel::update(vtodos_dsl::vtodos)
                .filter(vtodos_dsl::id.eq(vtodo_id))
                .set((
                    vtodos_dsl::status.eq(status),
                    completed_update,
                    vtodos_dsl::out_of_sync.eq(true),
                ))
                .execute(conn)
        })
        .await??;
        Self::by_id(conn, vtodo_id).await
    }

    pub fn apply_upsert<Tz: TimeZone>(
        &self,
        input: &str,
        extracted: EventUpsertInfo<Tz>,
        date_of_update: DateTime<Tz>,
        out_of_sync: Option<bool>,
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
        if let Some(out_of_sync) = out_of_sync {
            todo.out_of_sync = out_of_sync
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
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
    pub last_modified: Option<chrono::DateTime<Utc>>,
    pub etag: Option<String>,
    pub synced_at: Option<chrono::DateTime<Utc>>,
    pub completed: Option<chrono::DateTime<Utc>>,
    pub out_of_sync: bool,
}

impl_ical_parseable!(VTodo, icalendar::Todo, |f| f.as_todo());
impl_ical_parseable!(NewVTodo, icalendar::Todo, |f| f.as_todo());

pub(crate) trait VTodoTrait: IcalParseableTrait<icalendar::Todo> {
    async fn create(&self, conn: DbConn) -> anyhow::Result<VTodo>;
    async fn update(&self, conn: DbConn, id: i32) -> anyhow::Result<VTodo>;
    async fn upsert_by_href(&self, conn: DbConn) -> anyhow::Result<VTodo>;
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
            async fn create(&self, conn: DbConn) -> anyhow::Result<VTodo> {
                use crate::schema::vtodos::dsl as todo_dsl;

                let todo = self.clone();
                let val = spawn_blocking(move || {
                    let conn = &mut *conn.0.lock().unwrap();

                    insert_into(todo_dsl::vtodos)
                        .values(todo)
                        .returning(VTodo::as_returning())
                        .get_result(conn)
                })
                .await??;
                Ok(val)
            }
            async fn update(&self, conn: DbConn, id: i32) -> anyhow::Result<VTodo> {
                use crate::schema::vtodos::dsl as todo_dsl;

                let todo = self.clone();
                let val = spawn_blocking(move || {
                    let conn = &mut *conn.0.lock().unwrap();
                    update(todo_dsl::vtodos.filter(todo_dsl::id.eq(id)))
                        .set(todo)
                        .returning(VTodo::as_returning())
                        .get_result(conn)
                })
                .await??;
                Ok(val)
            }

            async fn upsert_by_href(&self, conn: DbConn) -> anyhow::Result<VTodo> {
                let href = Href(self.href.clone().expect("$t must have href"));
                let vevent = VTodo::by_href(conn.clone(), &href).await?;
                match vevent {
                    Some(old) => self.update(conn.clone(), old.id).await,
                    None => self.create(conn).await,
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
            last_modified: Some(last_modified),
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
            synced_at: Some(chrono::Utc::now()),
            has_rrule: false,
            rrule_str: None,
            starts_at,
            ends_at,
            out_of_sync: false,
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
