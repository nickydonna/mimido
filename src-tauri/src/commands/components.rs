use std::str::FromStr;

use crate::{
    caldav::{Caldav, Etag, Href},
    calendar_items::{
        event_creator::EventUpsertInfo,
        event_status::EventStatus,
        input_traits::{ExtractableFromInput, ExtractedInput, ToInput},
        DisplayUpsertInfo,
    },
    commands::calendar::{super_sync_calendar, CalendarCommandError},
    establish_connection,
    models::{
        vevent::{VEvent, VEventTrait},
        vtodo::VTodo,
        Calendar,
    },
    util::DateTimeStr,
};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use diesel::prelude::*;
use icalendar::Component;
use libdav::sd::BootstrapError;
use log::info;
use now::DateTimeNow;
use specta::Type;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ComponentsCommandError {
    #[error("Diesel error {0:?}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Could not connect to caldav")]
    CaldavBootstrap(#[from] BootstrapError),
    #[error("Error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Calendar(#[from] CalendarCommandError),
}

impl From<ComponentsCommandError> for String {
    fn from(value: ComponentsCommandError) -> Self {
        value.to_string()
    }
}

impl Type for ComponentsCommandError {
    fn inline(
        type_map: &mut specta::TypeCollection,
        generics: specta::Generics,
    ) -> specta::datatype::DataType {
        String::inline(type_map, generics)
    }
}

impl serde::Serialize for ComponentsCommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct ExtendedEvent {
    /// Date when the extended event was calculated
    pub query_date: DateTime<Utc>,
    pub event: VEvent,
    /// The start date of the event, if recurrent the value for the current query
    pub starts_at: DateTime<Utc>,
    /// The end date of the event, if recurrent the value for the current query
    pub ends_at: DateTime<Utc>,
    pub natural_recurrence: Option<String>,
    pub natural_string: String,
}

impl ExtendedEvent {
    pub fn on_day<Tz: TimeZone>(event: &VEvent, query_date: &DateTime<Tz>) -> Option<Self> {
        if !event.has_rrule && event.starts_at.date_naive() != query_date.date_naive() {
            log::warn!(
                "Event {} does not have a recurrence rule and is not on the requested date",
                event.uid
            );
            return None;
        }
        let base = query_date.beginning_of_day();
        let (starts_at, ends_at) = event.get_start_end_for_date(&base);
        if starts_at > base && starts_at < query_date.end_of_day() {
            Some(Self {
                query_date: query_date.to_utc(),
                event: event.clone(),
                starts_at: starts_at.to_utc(),
                ends_at: ends_at.to_utc(),
                natural_recurrence: None,
                natural_string: event.to_input(query_date),
            })
        } else {
            None
        }
    }
}

#[tauri::command()]
#[specta::specta]
pub async fn list_todos(include_done: bool) -> Result<Vec<VTodo>, ComponentsCommandError> {
    use crate::schema::vtodos::dsl as todo_dsl;

    let conn = &mut establish_connection();

    let todos = if include_done {
        todo_dsl::vtodos.select(VTodo::as_select()).load(conn)?
    } else {
        todo_dsl::vtodos
            .filter(todo_dsl::status.is_not(EventStatus::Done))
            .select(VTodo::as_select())
            .load(conn)?
    };

    Ok(todos)
}

#[tauri::command()]
#[specta::specta]
pub async fn set_vtodo_status(vtodo_id: i32, status: String) -> Result<(), ComponentsCommandError> {
    use crate::schema::vtodos::dsl as vtodos_dsl;

    let status = EventStatus::from_str(status.as_ref())?;

    let conn = &mut establish_connection();
    diesel::update(vtodos_dsl::vtodos)
        .filter(vtodos_dsl::id.eq(vtodo_id))
        .set(vtodos_dsl::status.eq(status))
        .execute(conn)?;

    let conn = &mut establish_connection();
    let vtodo = VTodo::by_id(conn, vtodo_id)?;
    let vtodo = vtodo.ok_or(anyhow!("No todo with id {vtodo_id}"))?;
    let etag: Etag = vtodo.etag.clone().into();
    let href: Href = vtodo.href.clone().into();
    let (server, calendar) = Calendar::by_id_with_server(conn, vtodo.calendar_id)?;

    let caldav = Caldav::new(server).await?;
    let updated_calendar_cmp: icalendar::CalendarComponent = vtodo.into();

    let cal = icalendar::Calendar::new().push(updated_calendar_cmp).done();

    caldav.update_cmp(&href, &etag, cal).await?;

    super_sync_calendar(calendar.id).await?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn list_events_for_day(
    datetime: String,
) -> Result<Vec<ExtendedEvent>, ComponentsCommandError> {
    use crate::schema::vevents::dsl as event_dsl;

    let conn = &mut establish_connection();

    let parsed: DateTime<FixedOffset> = DateTimeStr(datetime).try_into()?;
    // let parsed = parsed.to_utc();
    let start = parsed.beginning_of_day();
    let end = parsed.end_of_day();

    let events = event_dsl::vevents
        .filter(
            event_dsl::has_rrule.eq(true).or(event_dsl::starts_at
                .ge(start)
                .and(event_dsl::ends_at.le(end))),
        )
        .select(VEvent::as_select())
        .load(conn)?;

    let events = events
        .iter()
        .filter_map(|event| ExtendedEvent::on_day(event, &parsed))
        .collect::<Vec<ExtendedEvent>>();

    Ok(events)
}

#[tauri::command()]
#[specta::specta]
pub async fn parse_event(
    date_of_input_str: String,
    component_input: String,
) -> Result<DisplayUpsertInfo, ComponentsCommandError> {
    let parsed_date: DateTime<FixedOffset> = DateTimeStr(date_of_input_str).try_into()?;
    // let parsed_date = parsed_date.with_timezone(&Tz::UTC);
    info!("{parsed_date}");

    let ExtractedInput(data, _) =
        EventUpsertInfo::extract_from_input(parsed_date, &component_input)?.into();
    Ok(data.into())
}

#[tauri::command()]
#[specta::specta]
pub async fn save_event(
    calendar_id: i32,
    date_of_input_str: String,
    component_input: String,
) -> Result<(), ComponentsCommandError> {
    let conn = &mut establish_connection();

    let (server, calendar) = Calendar::by_id_with_server(conn, calendar_id)?;
    let caldav = Caldav::new(server).await?;

    let parsed_date: DateTime<FixedOffset> = DateTimeStr(date_of_input_str).try_into()?;

    let ExtractedInput(data, _) =
        EventUpsertInfo::extract_from_input(parsed_date, &component_input)?.into();

    let mut new_calendar_cmp: icalendar::CalendarComponent = data.into();

    let uid = Uuid::new_v4().to_string();
    let new_calendar_cmp = set_uid(&mut new_calendar_cmp, &uid);
    let cal = icalendar::Calendar::new().push(new_calendar_cmp).done();

    let cal_href: Href = calendar.url.clone().into();
    caldav.create_cmp(&cal_href, uid, cal).await?;
    super_sync_calendar(calendar.id).await?;

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub fn set_vevent_status(vevent_id: i32, status: String) -> Result<(), ComponentsCommandError> {
    use crate::schema::vevents::dsl as vevents_dsl;

    let status = EventStatus::from_str(status.as_ref())?;

    let conn = &mut establish_connection();
    diesel::update(vevents_dsl::vevents)
        .filter(vevents_dsl::id.eq(vevent_id))
        .set(vevents_dsl::status.eq(status))
        .execute(conn)?;
    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn delete_vevent(vevent_id: i32) -> Result<(), ComponentsCommandError> {
    let conn = &mut establish_connection();
    let vevent = VEvent::by_id(conn, vevent_id)?;
    let vevent = vevent.ok_or(anyhow!("No event with id {vevent_id}"))?;
    let (server, _) = Calendar::by_id_with_server(conn, vevent.calendar_id)?;

    let caldav = Caldav::new(server).await?;
    caldav
        .delete_resource(&vevent.href.into(), &vevent.etag.into())
        .await?;
    VEvent::delete_by_id(conn, vevent_id)?;

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn update_vevent(
    vevent_id: i32,
    date_of_input_str: String,
    component_input: String,
) -> Result<(), ComponentsCommandError> {
    let parsed_date: DateTime<FixedOffset> = DateTimeStr(date_of_input_str).try_into()?;

    let ExtractedInput(data, _) =
        EventUpsertInfo::extract_from_input(parsed_date, &component_input)?.into();

    let conn = &mut establish_connection();
    let vevent = VEvent::by_id(conn, vevent_id)?;
    let vevent = vevent.ok_or(anyhow!("No event with id {vevent_id}"))?;
    let updated = vevent.update_from_upsert(&component_input, data)?;
    let (server, calendar) = Calendar::by_id_with_server(conn, vevent.calendar_id)?;

    let caldav = Caldav::new(server).await?;
    let updated_calendar_cmp: icalendar::CalendarComponent = updated.into();

    let cal = icalendar::Calendar::new().push(updated_calendar_cmp).done();

    caldav
        .update_cmp(&vevent.href.into(), &vevent.etag.into(), cal)
        .await?;

    super_sync_calendar(calendar.id).await?;

    Ok(())
}

fn set_uid(cmp: &mut icalendar::CalendarComponent, uid: &str) -> icalendar::CalendarComponent {
    let updated: icalendar::CalendarComponent = match cmp {
        icalendar::CalendarComponent::Todo(todo) => todo.uid(uid).done().into(),
        icalendar::CalendarComponent::Event(event) => event.uid(uid).done().into(),
        _ => todo!(),
    };
    updated
}
