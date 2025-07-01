use crate::{
    caldav::Caldav,
    calendar_items::{
        event_creator::EventUpsertInfo,
        input_traits::{ExtractableFromInput, ExtractedInput},
        DisplayUpsertInfo,
    },
    establish_connection,
    models::{
        event::{Event, EventTrait},
        Calendar, Server,
    },
    util::{stringify, DateTimeStr},
};
use chrono::{DateTime, FixedOffset, Utc};
use diesel::prelude::*;
use icalendar::Component;
use log::info;
use now::DateTimeNow;
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct ExtendedEvent {
    pub event: Event,
    /// The start date of the event, if recurrent the value for the current query
    pub starts_at: DateTime<Utc>,
    /// The end date of the event, if recurrent the value for the current query
    pub ends_at: DateTime<Utc>,
    pub natural_recurrence: Option<String>,
}

impl ExtendedEvent {
    pub fn on_day(event: &Event, query_date: DateTime<Utc>) -> Option<Self> {
        if !event.has_rrule && event.starts_at.date_naive() != query_date.date_naive() {
            log::warn!(
                "Event {} does not have a recurrence rule and is not on the requested date",
                event.uid
            );
            return None;
        }
        let base = query_date.beginning_of_day();
        let (starts_at, ends_at) = event.get_start_end_for_date(base);
        if starts_at > base && starts_at < query_date.end_of_day() {
            Some(Self {
                event: event.clone(),
                starts_at,
                ends_at,
                natural_recurrence: None,
            })
        } else {
            None
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn list_events_for_day(datetime: String) -> Result<Vec<ExtendedEvent>, String> {
    use crate::schema::events::dsl as event_dsl;

    let conn = &mut establish_connection();

    let parsed: DateTime<FixedOffset> = DateTimeStr(datetime).try_into()?;
    let parsed = parsed.to_utc();
    let start = parsed.beginning_of_day();
    let end = parsed.end_of_day();

    let events = event_dsl::events
        .filter(
            event_dsl::has_rrule.eq(true).or(event_dsl::starts_at
                .ge(start)
                .and(event_dsl::ends_at.le(end))),
        )
        .select(Event::as_select())
        .load(conn)
        .map_err(stringify)?;

    let events = events
        .iter()
        .filter_map(|event| ExtendedEvent::on_day(event, parsed))
        .collect::<Vec<ExtendedEvent>>();

    Ok(events)
}

#[tauri::command()]
#[specta::specta]
pub async fn parse_event(
    date_of_input_str: String,
    component_input: String,
) -> Result<DisplayUpsertInfo, String> {
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
) -> Result<(), String> {
    use crate::schema::calendars::dsl as calendars_dsl;
    use crate::schema::servers::dsl as server_dsl;

    let conn = &mut establish_connection();

    let (server, calendar) = server_dsl::servers
        .inner_join(calendars_dsl::calendars)
        .filter(calendars_dsl::id.eq(calendar_id))
        .select((Server::as_select(), Calendar::as_select()))
        .first::<(Server, Calendar)>(conn)
        .map_err(|e| e.to_string())?;

    let caldav = Caldav::new(server).await?;

    let parsed_date: DateTime<FixedOffset> = DateTimeStr(date_of_input_str).try_into()?;
    // let parsed_date = parsed_date.with_timezone(&Tz::UTC);
    info!("{parsed_date}");

    let ExtractedInput(data, _) =
        EventUpsertInfo::extract_from_input(parsed_date, &component_input)?.into();

    let mut new_calendar_cmp: icalendar::CalendarComponent = data.into();

    let uid = Uuid::new_v4().to_string();
    let new_calendar_cmp = set_uid_or(&mut new_calendar_cmp, &uid);
    let cal = icalendar::Calendar::new().push(new_calendar_cmp).done();

    caldav
        .create_cmp(calendar.url, uid, cal)
        .await
        .map_err(stringify)?;

    Ok(())
}

fn set_uid_or(cmp: &mut icalendar::CalendarComponent, uid: &str) -> icalendar::CalendarComponent {
    let updated: icalendar::CalendarComponent = match cmp {
        icalendar::CalendarComponent::Todo(todo) => todo.uid(uid).done().into(),
        icalendar::CalendarComponent::Event(event) => event.uid(uid).done().into(),
        _ => todo!(),
    };
    updated
}
