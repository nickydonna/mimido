use crate::{
    calendar_items::{
        component_props::{ComponentProps, get_string_property},
        event_status::EventStatus,
        event_type::EventType,
        event_upsert::EventUpsertInfo,
    },
    db_conn::DbConn,
    establish_connection,
    models::{
        model_traits::{ById, DeleteById},
        server::Server,
        vevent::{NewVEvent, VEvent, VEventTrait},
        vtodo::{NewVTodo, VTodo, VTodoTrait},
    },
    schema::*,
};
use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::{dsl::update, prelude::*};
use icalendar::DatePerhapsTime;
use libdav::FetchedResource;
use rrule::{RRuleError, RRuleSet};

pub mod model_traits;
pub mod server;
pub mod vcmp_builder;
pub(crate) mod vevent;
pub(crate) mod vtodo;

use tauri::async_runtime::spawn_blocking;
pub use vcmp_builder::VCmpBuilder;

/// Enum to unify the [`VEvent`] and [`VTodo`] struct
#[derive(Debug)]
pub enum VCmp {
    Todo(VTodo),
    Event(VEvent),
}

impl ById for VCmp {
    async fn by_id(conn: DbConn, id: i32) -> anyhow::Result<Option<VCmp>> {
        let vevent = VEvent::by_id(conn.clone(), id).await?;
        if let Some(vevent) = vevent {
            return Ok(Some(VCmp::Event(vevent)));
        }
        let todo = VTodo::by_id(conn, id).await?;
        if let Some(todo) = todo {
            return Ok(Some(VCmp::Todo(todo)));
        }
        Ok(None)
    }
}

impl VCmp {
    pub fn get_calendar_id(&self) -> i32 {
        match self {
            VCmp::Todo(vtodo) => vtodo.calendar_id,
            VCmp::Event(vevent) => vevent.calendar_id,
        }
    }

    pub fn get_href(&self) -> Option<String> {
        match self {
            VCmp::Todo(vtodo) => vtodo.href.clone(),
            VCmp::Event(vevent) => vevent.href.clone(),
        }
    }

    pub fn get_etag(&self) -> Option<String> {
        match self {
            VCmp::Todo(vtodo) => vtodo.etag.clone(),
            VCmp::Event(vevent) => vevent.etag.clone(),
        }
    }

    pub async fn delete(&self, conn: DbConn) -> anyhow::Result<bool> {
        match self {
            VCmp::Todo(vtodo) => VTodo::delete_by_id(conn.clone(), vtodo.id).await,
            VCmp::Event(vevent) => VEvent::delete_by_id(conn, vevent.id).await,
        }
    }
    pub fn update_from_upsert<Tz: TimeZone>(
        &self,
        input: &str,
        extracted: EventUpsertInfo<Tz>,
        date_of_update: DateTime<Tz>,
    ) -> anyhow::Result<Self> {
        match self {
            VCmp::Todo(vtodo) => {
                let todo = vtodo.update_from_upsert(input, extracted, date_of_update)?;
                Ok(VCmp::Todo(todo))
            }
            VCmp::Event(vevent) => {
                let event = vevent.update_from_upsert(input, extracted)?;
                Ok(VCmp::Event(event))
            }
        }
    }
    pub fn set_status<Tz: TimeZone>(&mut self, status: EventStatus, date_of_update: DateTime<Tz>) {
        match self {
            VCmp::Todo(vtodo) => {
                vtodo.status = status;
                if matches!(status, EventStatus::Done) {
                    vtodo.completed = Some(date_of_update.to_utc());
                } else {
                    vtodo.completed = None;
                }
            }
            VCmp::Event(vevent) => {
                vevent.status = status;
            }
        }
    }
}

impl From<VCmp> for icalendar::CalendarComponent {
    fn from(value: VCmp) -> Self {
        match value {
            VCmp::Todo(vtodo) => vtodo.into(),
            VCmp::Event(vevent) => vevent.into(),
        }
    }
}

/// Enum to unify the [`NewVEvent`] and [`NewVTodo`] struct
#[derive(Debug)]
pub enum NewVCmp {
    Todo(NewVTodo),
    Event(NewVEvent),
}

impl NewVCmp {
    pub fn from_resource(
        calendar_id: i32,
        fetched_resource: &FetchedResource,
    ) -> anyhow::Result<NewVCmp> {
        let todo = NewVTodo::from_resource(calendar_id, fetched_resource)?;
        if let Some(todo) = todo {
            return Ok(NewVCmp::Todo(todo));
        }

        let event = NewVEvent::from_resource(calendar_id, fetched_resource)?;
        if let Some(event) = event {
            return Ok(NewVCmp::Event(event));
        }
        Err(anyhow!("No Supported Component found"))
    }

    pub async fn upsert_by_href(&self, conn: DbConn) -> anyhow::Result<VCmp> {
        match self {
            NewVCmp::Todo(new_vtodo) => {
                new_vtodo.upsert_by_href(conn.clone()).await.map(VCmp::Todo)
            }
            NewVCmp::Event(new_vevent) => new_vevent.upsert_by_href(conn).await.map(VCmp::Event),
        }
    }

    pub async fn create(&self, conn: DbConn) -> anyhow::Result<VCmp> {
        match self {
            NewVCmp::Todo(new_vtodo) => new_vtodo.create(conn.clone()).await.map(VCmp::Todo),
            NewVCmp::Event(new_vevent) => new_vevent.create(conn).await.map(VCmp::Event),
        }
    }
}

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Associations,
    AsChangeset,
    Debug,
    PartialEq,
    serde::Serialize,
    specta::Type,
)]
#[diesel(belongs_to(Server))]
#[diesel(table_name = calendars)]
pub struct Calendar {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub etag: Option<String>,
    pub server_id: i32,
    pub is_default: bool,
    pub sync_token: Option<String>,
}

impl Calendar {
    pub async fn by_id_with_server(
        conn: DbConn,
        calendar_id: i32,
    ) -> anyhow::Result<(Server, Calendar)> {
        use crate::schema::calendars::dsl as calendars_dsl;
        use crate::schema::servers::dsl as server_dsl;

        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            server_dsl::servers
                .inner_join(calendars_dsl::calendars)
                .filter(calendars_dsl::id.eq(calendar_id))
                .select((Server::as_select(), Calendar::as_select()))
                .first::<(Server, Calendar)>(conn)
                .map_err(anyhow::Error::new)
        })
        .await?
    }

    pub async fn by_name(name: &str) -> anyhow::Result<Option<Calendar>> {
        use crate::schema::calendars::dsl as calendars_dsl;
        let name = name.to_string();
        spawn_blocking(|| {
            let conn = &mut establish_connection();
            calendars_dsl::calendars
                .filter(calendars_dsl::name.eq(name))
                .select(Calendar::as_select())
                .first::<Calendar>(conn)
                .optional()
                .map_err(anyhow::Error::new)
        })
        .await?
    }

    pub async fn list_all() -> anyhow::Result<Vec<Calendar>> {
        use crate::schema::calendars::dsl as calendars_dsl;
        use crate::schema::servers::dsl as server_dsl;
        let servers = spawn_blocking(|| {
            let conn = &mut establish_connection();
            server_dsl::servers
                .inner_join(calendars_dsl::calendars)
                .select(Calendar::as_select())
                .load(conn)
        })
        .await??;

        Ok(servers)
    }

    pub async fn set_default_calendar(calendar_id: i32) -> anyhow::Result<()> {
        use crate::schema::calendars::dsl as calendars_dsl;
        spawn_blocking(move || {
            let conn = &mut establish_connection();
            conn.transaction::<(), diesel::result::Error, _>(|conn| {
                diesel::update(calendars_dsl::calendars)
                    .set(calendars_dsl::is_default.eq(false))
                    .execute(conn)?;

                diesel::update(calendars_dsl::calendars)
                    .filter(calendars_dsl::id.eq(calendar_id))
                    .set(calendars_dsl::is_default.eq(true))
                    .execute(conn)?;
                Ok(())
            })
            .map_err(anyhow::Error::new)
        })
        .await?
    }

    pub async fn update_sync_token(&self, new_token: &str) -> anyhow::Result<Calendar> {
        use crate::schema::calendars::dsl as calendars_dsl;
        let new_token = new_token.to_string();
        let id = self.id;
        spawn_blocking(move || {
            let conn = &mut establish_connection();
            update(calendars_dsl::calendars.filter(calendars_dsl::id.eq(id)))
                .set((
                    calendars_dsl::sync_token.eq(Some(new_token)),
                    calendars_dsl::synced_at.eq(Some(Utc::now())),
                ))
                .returning(Calendar::as_returning())
                .get_result(conn)
                .map_err(anyhow::Error::new)
        })
        .await?
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = calendars)]
pub struct NewCalendar {
    pub name: String,
    pub url: String,
    pub etag: Option<String>,
    pub server_id: i32,
    pub is_default: bool,
    pub sync_token: Option<String>,
    pub synced_at: Option<chrono::DateTime<Utc>>,
}

pub trait FromResource: Sized {
    /// Parses a [`FetchedResource`] for a particular calendar_id
    fn from_resource(
        calendar_id: i32,
        fetched_resource: &FetchedResource,
    ) -> anyhow::Result<Option<Self>>;
    /// function needed for testing, should be called by [`FromResource::fetched_resource`]
    fn from_ical_data(
        calendar_id: i32,
        href: &str,
        ical_data: &str,
        etag: &str,
    ) -> anyhow::Result<Option<Self>>;
}

pub trait IcalParseableTrait<Cmp: icalendar::Component> {
    fn get_ical_data(&self) -> Option<String>;
    fn get_summary(&self) -> String;
    fn get_description(&self) -> Option<String>;
    fn get_postponed(&self) -> i32;
    fn get_load(&self) -> i32;
    fn get_urgency(&self) -> i32;
    fn get_importance(&self) -> i32;
    fn get_status(&self) -> EventStatus;
    fn get_type(&self) -> EventType;
    fn parse_ical_data(&self) -> anyhow::Result<Cmp>;
    fn get_rrule_str(&self) -> Option<String>;
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

#[macro_export]
macro_rules! impl_ical_parseable {
    ($t: ty, $cmp: ty, $transform: expr) => {
        impl IcalParseableTrait<$cmp> for $t {
            fn get_ical_data(&self) -> Option<String> {
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
            fn get_rrule_str(&self) -> Option<String> {
                self.rrule_str.clone()
            }
            fn parse_ical_data(&self) -> Result<$cmp, anyhow::Error> {
                let cal: icalendar::Calendar = self
                    .get_ical_data()
                    .ok_or(anyhow!("$ty must have ical data to be parsed"))?
                    .parse()
                    .map_err(|e: String| anyhow!(e))?;
                let events = cal
                    .components
                    .iter()
                    .filter_map($transform)
                    .collect::<Vec<&$cmp>>();
                let event = events
                    .first()
                    .ok_or(anyhow!("iCal was parsed correctly but not event was found"))?;
                Ok((*event).clone())
            }
        }
    };
}

fn format_date_ical(date: NaiveDateTime) -> String {
    date.format("%Y%m%dT%H%M%S").to_string()
}

fn get_start_string(cmp: &impl icalendar::Component) -> Option<String> {
    let dt_start = cmp.get_start()?;

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
