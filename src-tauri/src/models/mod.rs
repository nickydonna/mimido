use crate::{
    calendar_items::{event_status::EventStatus, event_type::EventType},
    schema::*,
};
use diesel::prelude::*;

pub(crate) mod event;
pub(crate) mod todo;

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
