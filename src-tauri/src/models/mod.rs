use crate::{
    calendar_items::{event_status::EventStatus, event_type::EventType},
    models::{
        vevent::{NewVEvent, VEvent, VEventTrait},
        vtodo::{NewVTodo, VTodo, VTodoTrait},
    },
    schema::*,
};
use anyhow::anyhow;
use diesel::{dsl::update, prelude::*};
use libdav::FetchedResource;

pub(crate) mod vevent;
pub(crate) mod vtodo;

/// Enum to unify the [`VEvent`] and [`VTodo`] struct

#[derive(Debug)]
pub enum VCmp {
    Todo(VTodo),
    Event(VEvent),
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

    pub fn upsert_by_href(&self, conn: &mut SqliteConnection) -> anyhow::Result<VCmp> {
        match self {
            NewVCmp::Todo(new_vtodo) => new_vtodo.upsert_by_href(conn).map(VCmp::Todo),
            NewVCmp::Event(new_vevent) => new_vevent.upsert_by_href(conn).map(VCmp::Event),
        }
    }
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    Debug,
    serde::Serialize,
    specta::Type,
    Clone,
    PartialEq,
)]
#[diesel(table_name = servers)]
pub struct Server {
    pub id: i32,
    pub server_url: String,
    pub user: String,
    pub password: String,
    pub last_sync: Option<i64>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = servers)]
pub struct NewServer {
    pub server_url: String,
    pub user: String,
    pub password: String,
    pub last_sync: Option<i64>,
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
    pub fn by_id_with_server(
        conn: &mut SqliteConnection,
        calendar_id: i32,
    ) -> anyhow::Result<(Server, Calendar)> {
        use crate::schema::calendars::dsl as calendars_dsl;
        use crate::schema::servers::dsl as server_dsl;

        server_dsl::servers
            .inner_join(calendars_dsl::calendars)
            .filter(calendars_dsl::id.eq(calendar_id))
            .select((Server::as_select(), Calendar::as_select()))
            .first::<(Server, Calendar)>(conn)
            .map_err(anyhow::Error::new)
    }

    pub fn by_name(
        connection: &mut SqliteConnection,
        name: &str,
    ) -> anyhow::Result<Option<Calendar>> {
        use crate::schema::calendars::dsl as calendars_dsl;

        calendars_dsl::calendars
            .filter(calendars_dsl::name.eq(name))
            .select(Calendar::as_select())
            .first::<Calendar>(connection)
            .optional()
            .map_err(anyhow::Error::new)
    }

    pub fn update_sync_token(
        &self,
        conn: &mut SqliteConnection,
        new_token: &str,
    ) -> anyhow::Result<Calendar> {
        use crate::schema::calendars::dsl as calendars_dsl;

        update(calendars_dsl::calendars.filter(calendars_dsl::id.eq(self.id)))
            .set(calendars_dsl::sync_token.eq(Some(new_token)))
            .returning(Calendar::as_returning())
            .get_result(conn)
            .map_err(anyhow::Error::new)
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
}

pub trait IcalParseableTrait {
    fn get_ical_data(&self) -> String;
    fn get_summary(&self) -> String;
    fn get_description(&self) -> Option<String>;
    fn get_postponed(&self) -> i32;
    fn get_load(&self) -> i32;
    fn get_urgency(&self) -> i32;
    fn get_importance(&self) -> i32;
    fn get_status(&self) -> EventStatus;
    fn get_type(&self) -> EventType;
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
}

#[macro_export]
macro_rules! impl_ical_parseable {
    ($t: ty) => {
        impl IcalParseableTrait for $t {
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
    };
}
