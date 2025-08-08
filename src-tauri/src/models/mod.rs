use crate::{
    calendar_items::{
        component_props::{get_string_property, ComponentProps},
        event_status::EventStatus,
        event_type::EventType,
    },
    models::{
        vevent::{NewVEvent, VEvent, VEventTrait},
        vtodo::{NewVTodo, VTodo, VTodoTrait},
    },
    schema::*,
};
use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use diesel::{dsl::update, prelude::*};
use icalendar::DatePerhapsTime;
use libdav::FetchedResource;
use rrule::{RRuleError, RRuleSet};

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

pub trait IcalParseableTrait<Cmp: icalendar::Component> {
    fn get_ical_data(&self) -> String;
    fn get_summary(&self) -> String;
    fn get_description(&self) -> Option<String>;
    fn get_postponed(&self) -> i32;
    fn get_load(&self) -> i32;
    fn get_urgency(&self) -> i32;
    fn get_importance(&self) -> i32;
    fn get_status(&self) -> EventStatus;
    fn get_type(&self) -> EventType;
    fn parse_ical_data(&self) -> Result<Cmp, String>;
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
            fn get_rrule_str(&self) -> Option<String> {
                self.rrule_str.clone()
            }
            fn parse_ical_data(&self) -> Result<$cmp, String> {
                let cal: icalendar::Calendar = self.get_ical_data().parse()?;
                let events = cal
                    .components
                    .iter()
                    .filter_map($transform)
                    .collect::<Vec<&$cmp>>();
                let event = events
                    .first()
                    .ok_or("iCal was parsed correctly but not event was found".to_string())?;
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
