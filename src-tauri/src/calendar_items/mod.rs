use crate::models::NewTodo;
use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use component_props::{
    get_int_property, get_property_or_default, get_string_property, ComponentProps,
};
use event_status::EventStatus;
use event_type::EventType;
use icalendar::{Component, DatePerhapsTime};
use libdav::FetchedResource;

pub(crate) mod component_props;
pub(crate) mod date_parser;
pub(crate) mod event_status;
pub(crate) mod event_type;
pub(crate) mod rrule_parser;

pub fn extract_todo(
    calendar_id: i32,
    fetched_resource: &FetchedResource,
) -> Result<Option<NewTodo>, String> {
    let href = fetched_resource.href.clone();
    let content = fetched_resource
        .content
        .as_ref()
        .map_err(|e| e.to_string())?;
    let calendar_item: Result<icalendar::Calendar, String> = content.data.parse();
    let calendar_item = calendar_item?;
    let first_todo = calendar_item
        .components
        .into_iter()
        .filter_map(|cmp| cmp.as_todo().cloned())
        .collect::<Vec<icalendar::Todo>>();

    let Some(first_todo) = first_todo.first() else {
        return Ok(None);
    };

    let uid = first_todo.get_uid().ok_or("Missing UID".to_string())?;
    let summary = first_todo.get_summary().unwrap_or("[No Summary]");
    let description = first_todo.get_description().map(|d| d.to_string());
    let last_modified = first_todo
        .get_last_modified()
        .map(|modified| modified.timestamp())
        .unwrap_or(Utc::now().timestamp());
    let event_type = get_property_or_default(first_todo, ComponentProps::Type, EventType::Event);
    let tag = get_string_property(first_todo, ComponentProps::Tag);
    let status = get_property_or_default(first_todo, ComponentProps::Status, EventStatus::Todo);
    let original_text = get_string_property(first_todo, ComponentProps::OriginalText);
    let importance = get_int_property(first_todo, ComponentProps::Importance);
    let urgency = get_int_property(first_todo, ComponentProps::Urgency);
    let load = get_int_property(first_todo, ComponentProps::Load);
    let postponed = get_int_property(first_todo, ComponentProps::Postponed);

    println!(
        "type: {:#?} {:#?} {:#?} {:#?} {:#?} {:#?}",
        event_type,
        format!("{:.10}", summary),
        status,
        importance,
        urgency,
        load
    );

    Ok(Some(NewTodo {
        calendar_id,
        uid: uid.to_string(),
        href,
        ical_data: content.data.clone(),
        last_modified,
        summary: summary.to_string(),
        // TODO: Use real completed and sycn with status
        completed: false,
        description,
        status,
        original_text,
        tag,
        event_type,
        importance,
        load,
        urgency,
        postponed,
    }))
}

pub fn date_from_calendar_to_utc(
    original: DatePerhapsTime,
    timezone: chrono_tz::Tz,
) -> Option<DateTime<Utc>> {
    match original {
        DatePerhapsTime::DateTime(calendar_date_time) => match calendar_date_time {
            icalendar::CalendarDateTime::Floating(_) => None,
            icalendar::CalendarDateTime::Utc(date_time) => Some(date_time),
            icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
                let tz: chrono_tz::Tz = tzid.parse().ok()?;
                tz.from_local_datetime(&date_time)
                    .earliest()
                    .map(|d| d.to_utc())
            }
        },
        DatePerhapsTime::Date(naive_date) => naive_date
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_local_timezone(timezone)
            .earliest()
            .map(|d| d.to_utc()),
    }
}
