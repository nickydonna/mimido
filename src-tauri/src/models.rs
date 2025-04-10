use crate::{
    calendar_items::event_status::EventStatus, calendar_items::event_type::EventType, schema::*,
};
use chrono::Utc;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug, serde::Serialize, specta::Type, Clone)]
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize, specta::Type)]
#[diesel(table_name = calendars)]
pub struct Calendar {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub etag: Option<String>,
    pub server_id: i32,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = calendars)]
pub struct NewCalendar {
    pub name: String,
    pub url: String,
    pub etag: Option<String>,
    pub server_id: i32,
}

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

impl TryFrom<Event> for icalendar::Event {
    type Error = String;
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        let cal: icalendar::Calendar = value.ical_data.parse()?;
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize)]
#[diesel(table_name = todos)]
pub struct Todo {
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
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = todos)]
pub struct NewTodo {
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
}
