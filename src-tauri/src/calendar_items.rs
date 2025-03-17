use std::str::FromStr;

use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use icalendar::{Component, DatePerhapsTime};
use libdav::FetchedResource;
use strum_macros;

use crate::models::NewEvent;

pub fn extract_event(
    calendar_id: i32,
    fetched_resource: FetchedResource,
) -> Result<Option<NewEvent>, String> {
    let content = fetched_resource.content.map_err(|e| e.to_string())?;
    let calendar_item: Result<icalendar::Calendar, String> = content.data.parse();
    let calendar_item = calendar_item?;
    let calendar_offset = calendar_item
        .get_timezone()
        .and_then(|tzid| {
            let tz: Option<chrono_tz::Tz> = tzid.parse().ok();
            tz
        })
        .unwrap_or(chrono_tz::UTC);

    let first_event = calendar_item
        .components
        .into_iter()
        .filter(|cmp| matches!(cmp, icalendar::CalendarComponent::Event(_)))
        .collect::<Vec<icalendar::CalendarComponent>>();

    let Some(first_event) = first_event.first().and_then(|e| e.as_event()) else {
        return Ok(None);
    };

    let uid = first_event.get_uid();
    let summary = first_event.get_summary().unwrap_or("[No Summary]");
    let description = first_event.get_description().map(|d| d.to_string());
    let last_modified = first_event
        .get_last_modified()
        .map(|modified| modified.timestamp())
        .unwrap_or(Utc::now().timestamp());
    let start = first_event
        .get_start()
        .and_then(|s| date_from_calendar_to_sql(s, calendar_offset));
    let end = first_event
        .get_end()
        .and_then(|e| date_from_calendar_to_sql(e, calendar_offset));
    let recur = get_string_property(first_event, ComponentProps::Recur);
    let values = match (uid, start, end) {
        (Some(uid), Some(start), Some(end)) => Some((uid, start, end)),
        _ => None,
    };
    let event_type = get_property_or_default(first_event, ComponentProps::Type, EventType::Event);
    let tag = get_string_property(first_event, ComponentProps::Tag);
    let status = get_property_or_default(first_event, ComponentProps::Status, EventStatus::Todo);
    let original_text = get_string_property(first_event, ComponentProps::OriginalText);
    let importance = get_int_property(first_event, ComponentProps::Importance);
    let urgency = get_int_property(first_event, ComponentProps::Urgency);
    let load = get_int_property(first_event, ComponentProps::Load);

    let Some((uid, starts_at, ends_at)) = values else {
        return Ok(None);
    };
    println!(
        "type: {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?}",
        event_type, tag, status, original_text, importance, urgency, load
    );

    Ok(Some(NewEvent {
        calendar_id,
        uid: uid.to_string(),
        href: fetched_resource.href,
        ical_data: content.data,
        starts_at,
        ends_at,
        last_modified,
        summary: summary.to_string(),
        description,
        recur,
    }))
}

#[derive(Debug, PartialEq, strum_macros::AsRefStr)]
enum ComponentProps {
    #[strum(serialize = "lowercase")]
    Recur,
    #[strum(serialize = "X-TYPE")]
    Type,
    #[strum(serialize = "X-TAG")]
    Tag,
    #[strum(serialize = "X-URGENCY")]
    Urgency,
    #[strum(serialize = "X-STATUS")]
    Status,
    #[strum(serialize = "X-ORIGINAL-TEXT")]
    OriginalText,
    #[strum(serialize = "X-IMPORTANCE")]
    Importance,
    #[strum(serialize = "X-LOAD")]
    Load,
}

#[derive(Debug, PartialEq, strum_macros::AsRefStr, strum_macros::EnumString)]
#[strum(serialize_all = "lowercase")]
enum EventType {
    Event,
    Block,
    Reminder,
    Task,
}

#[derive(Debug, PartialEq, strum_macros::AsRefStr, strum_macros::EnumString)]
#[strum(serialize_all = "lowercase")]
enum EventStatus {
    #[strum(serialize = "back")]
    Backlog,
    Todo,
    Doing,
    Done,
}

fn get_property_or_default<T: FromStr>(
    event: &icalendar::Event,
    property: ComponentProps,
    default: T,
) -> T {
    let raw_type = event.property_value(property.as_ref());
    let Some(raw_type) = raw_type else {
        return default;
    };
    T::from_str(raw_type).ok().unwrap_or(default)
}

fn get_string_property(event: &icalendar::Event, property: ComponentProps) -> Option<String> {
    event
        .property_value(property.as_ref())
        .map(|e| e.to_string())
}

fn get_int_property(event: &icalendar::Event, property: ComponentProps) -> i32 {
    event
        .property_value(property.as_ref())
        .map(|e| e.to_string())
        .and_then(|e| e.parse::<i32>().ok())
        .unwrap_or(0)
}

fn date_from_calendar_to_sql(
    original: DatePerhapsTime,
    offset: chrono_tz::Tz,
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
            .and_local_timezone(offset)
            .earliest()
            .map(|d| d.to_utc()),
    }
}
