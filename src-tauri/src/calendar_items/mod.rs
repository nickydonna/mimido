use crate::models::NewEvent;
use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use component_props::{
    get_int_property, get_property_or_default, get_string_property, ComponentProps,
};
use event_status::EventStatus;
use event_type::EventType;
use icalendar::{Component, DatePerhapsTime};
use libdav::FetchedResource;
use recur::parse_rrule;

pub(crate) mod component_props;
pub(crate) mod date_parser;
pub(crate) mod event_status;
pub(crate) mod event_type;
pub(crate) mod recur;
pub(crate) mod rrule_parser;

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
    let postponed = get_int_property(first_event, ComponentProps::Postponed);

    let Some((uid, starts_at, ends_at)) = values else {
        return Ok(None);
    };
    // println!(
    //     "type: {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?}",
    //     event_type, tag, status, original_text, importance, urgency, load
    // );
    println!("{} {:#?}-{:#?}", summary, start, end);

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
        has_rrule: parse_rrule(first_event).is_some(),
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
