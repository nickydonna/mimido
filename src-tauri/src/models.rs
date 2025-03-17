use crate::{
    calendar_items::event_status::EventStatus, calendar_items::event_type::EventType, schema::*,
};
use chrono::Utc;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug, serde::Serialize, specta::Type)]
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize, specta::Type)]
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize)]
#[diesel(table_name = todo_lists)]
pub struct TodoList {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub ctag: String,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = todo_lists)]
pub struct NewTodoList {
    pub name: String,
    pub url: String,
    pub ctag: String,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize)]
#[diesel(table_name = todos)]
pub struct Todo {
    pub id: i32,
    pub list_id: i32,
    pub uid: String,
    pub etag: String,
    pub url: String,
    pub ical_data: String,
    pub last_modified: i64,
    pub completed: bool,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = todos)]
pub struct NewTodo {
    pub list_id: i32,
    pub uid: String,
    pub etag: String,
    pub url: String,
    pub ical_data: String,
    pub last_modified: i64,
    pub completed: bool,
}
