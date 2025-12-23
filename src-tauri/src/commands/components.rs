use std::str::FromStr;

use crate::{
    SyncEventPayload,
    caldav::Caldav,
    calendar_items::{
        DisplayUpsertInfo,
        event_status::EventStatus,
        event_upsert::EventUpsertInfo,
        input_traits::{ExtractedInput, FromUserInput},
    },
    commands::{
        errors::CommandError,
        extended_event::ExtendedEvent,
        extended_todo::{ExtendedTodo, UnscheduledTodo},
    },
    db_conn::DbConn,
    models::{
        Calendar, VCmp, VCmpBuilder,
        model_traits::{ById, ListForDayOrRecurring},
        vevent::VEvent,
        vtodo::VTodo,
    },
    util::{DateTimeStr, Href},
};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use icalendar::Component;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[tauri::command()]
#[specta::specta]
pub async fn list_unscheduled_todos(
    include_done: bool,
) -> Result<Vec<UnscheduledTodo>, CommandError> {
    let conn = DbConn::new().await?;

    let todos = VTodo::list_unscheduled(conn, include_done).await?;
    Ok(todos)
}

#[tauri::command()]
#[specta::specta]
pub async fn set_vtodo_status(
    app: AppHandle,
    vtodo_id: i32,
    status: String,
    date_of_change: String,
) -> Result<(), CommandError> {
    let updated_at: DateTime<FixedOffset> = DateTimeStr(date_of_change).try_into()?;
    let status = EventStatus::from_str(status.as_ref())?;
    let conn = DbConn::new().await?;

    let vtodo = VTodo::update_status_by_id(conn, vtodo_id, status, updated_at)
        .await?
        .ok_or(anyhow!("No todo with id {vtodo_id}"))?;

    let _ = app.emit("sync", SyncEventPayload::new(vtodo.calendar_id));

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn set_vevent_status(
    app: AppHandle,
    vevent_id: i32,
    status: String,
) -> Result<(), CommandError> {
    let status = EventStatus::from_str(status.as_ref())?;
    let conn = DbConn::new().await?;

    let vevent = VEvent::update_status_by_id(conn, vevent_id, status)
        .await?
        .ok_or(anyhow!("No todo with id {vevent_id}"))?;

    let _ = app.emit("sync", SyncEventPayload::new(vevent.calendar_id));

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn list_events_for_day(datetime: String) -> Result<Vec<ExtendedEvent>, CommandError> {
    let conn = DbConn::new().await?;
    let parsed: DateTime<FixedOffset> = DateTimeStr(datetime).try_into()?;

    let events = VEvent::list_for_day_or_recurring(conn, parsed).await?;
    let events = events
        .iter()
        .filter_map(|event| ExtendedEvent::on_day(event, &parsed))
        .collect::<Vec<ExtendedEvent>>();

    Ok(events)
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn list_todos_for_day(datetime: String) -> Result<Vec<ExtendedTodo>, CommandError> {
    let conn = DbConn::new().await?;

    let parsed: DateTime<FixedOffset> = DateTimeStr(datetime).try_into()?;

    let todos = VTodo::list_for_day_or_recurring(conn, parsed).await?;
    let todos = todos
        .iter()
        .filter_map(|vtodo| ExtendedTodo::on_day(vtodo, &parsed))
        .collect::<Vec<ExtendedTodo>>();

    Ok(todos)
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
pub async fn create_component(
    calendar_id: i32,
    date_of_input_str: String,
    component_input: String,
) -> Result<(), CommandError> {
    let conn = DbConn::new().await?;

    let (server, calendar) = Calendar::by_id_with_server(conn.clone(), calendar_id).await?;
    let caldav = Caldav::new(server).await?;

    let parsed_date: DateTime<FixedOffset> = DateTimeStr(date_of_input_str).try_into()?;

    let ExtractedInput(data, _) =
        EventUpsertInfo::extract_from_input(parsed_date, &component_input)?.into();

    let uid = Uuid::new_v4().to_string();

    let builder = VCmpBuilder::from(&data)
        .calendar_id(calendar_id)
        .uid(&uid)
        .calendar_href(Href(calendar.url));

    builder.build_new()?.create(conn).await?;

    // Move this to background events
    // async_runtime::spawn(async move {
    //     let new_calendar_cmp = set_uid(&mut new_calendar_cmp, &uid);
    //     let cal = icalendar::Calendar::new().push(new_calendar_cmp).done();
    //     let res = caldav.create_component(&cal_href, uid, &cal).await;
    //     match res {
    //         Ok(_) => app.emit("sync", SyncEventPayload { calendar_id }).unwrap(),
    //         Err(err) => {
    //             error!("Error while creating component {err:?}");
    //         }
    //     }
    // });

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn delete_vcmp(vevent_id: i32) -> Result<(), CommandError> {
    let conn = DbConn::new().await?;
    let cmp = VCmp::by_id(conn.clone(), vevent_id)
        .await?
        .ok_or(anyhow!("No cmp with id {vevent_id}"))?;

    cmp.delete(conn).await?;

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn update_vcmp(
    vevent_id: i32,
    date_of_input_str: String,
    component_input: String,
) -> Result<(), CommandError> {
    let conn = DbConn::new().await?;
    let parsed_date: DateTime<FixedOffset> = DateTimeStr(date_of_input_str).try_into()?;

    let ExtractedInput(data, _) =
        EventUpsertInfo::extract_from_input(parsed_date, &component_input)?.into();

    let vevent = VCmp::by_id(conn, vevent_id).await?;
    let vcmp = vevent.ok_or(anyhow!("No event with id {vevent_id}"))?;
    vcmp.update_from_upsert(&component_input, data, parsed_date)?;

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
