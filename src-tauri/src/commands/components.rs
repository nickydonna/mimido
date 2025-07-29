use std::str::FromStr;

use crate::{
    caldav::{Caldav, Etag, Href},
    calendar_items::{
        event_status::EventStatus,
        event_upsert::EventUpsertInfo,
        input_traits::{ExtractableFromInput, ExtractedInput},
        DisplayUpsertInfo,
    },
    commands::{
        calendar::super_sync_calendar, errors::CommandError, extended_event::ExtendedEvent,
    },
    establish_connection,
    models::{vevent::VEvent, vtodo::VTodo, Calendar},
    util::DateTimeStr,
};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use diesel::prelude::*;
use icalendar::Component;
use now::DateTimeNow;
use uuid::Uuid;

#[tauri::command()]
#[specta::specta]
pub async fn list_todos(include_done: bool) -> Result<Vec<VTodo>, CommandError> {
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
pub async fn set_vtodo_status(vtodo_id: i32, status: String) -> Result<(), CommandError> {
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
pub async fn list_events_for_day(datetime: String) -> Result<Vec<ExtendedEvent>, CommandError> {
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
) -> Result<DisplayUpsertInfo, CommandError> {
    let parsed_date: DateTime<FixedOffset> = DateTimeStr(date_of_input_str).try_into()?;

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
) -> Result<(), CommandError> {
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
pub fn set_vevent_status(vevent_id: i32, status: String) -> Result<(), CommandError> {
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
pub async fn delete_vevent(vevent_id: i32) -> Result<(), CommandError> {
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
) -> Result<(), CommandError> {
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
