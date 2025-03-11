use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use icalendar::{Component, DatePerhapsTime};
use libdav::FetchedResource;

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
    let description = first_event.get_summary().map(|d| d.to_string());
    let start = first_event
        .get_start()
        .and_then(|s| date_from_calendar_to_sql(s, calendar_offset));
    let end = first_event
        .get_end()
        .and_then(|e| date_from_calendar_to_sql(e, calendar_offset));
    let recur = first_event.property_value("recur").map(|r| r.to_string());

    let values = match (uid, start, end) {
        (Some(uid), Some(start), Some(end)) => Some((uid, start, end)),
        _ => None,
    };

    let Some((uid, starts_at, ends_at)) = values else {
        return Ok(None);
    };
    println!("{:#?} - {:#?}", starts_at, ends_at);

    Ok(Some(NewEvent {
        calendar_id,
        uid: uid.to_string(),
        href: fetched_resource.href,
        ical_data: content.data,
        starts_at,
        ends_at,
        last_modified: Utc::now().timestamp(),
        summary: summary.to_string(),
        description,
        recur,
    }))
}

fn date_from_calendar_to_sql(
    original: DatePerhapsTime,
    offset: chrono_tz::Tz,
) -> Option<DateTime<chrono_tz::Tz>> {
    match original {
        DatePerhapsTime::DateTime(calendar_date_time) => match calendar_date_time {
            icalendar::CalendarDateTime::Floating(_) => None,
            icalendar::CalendarDateTime::Utc(date_time) => Some(date_time.with_timezone(&offset)),
            icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
                let tz: chrono_tz::Tz = tzid.parse().ok()?;
                tz.from_local_datetime(&date_time).earliest()
                // date_time.and_local_timezone(tz).earliest()
            }
        },
        DatePerhapsTime::Date(naive_date) => naive_date
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_local_timezone(offset)
            .earliest(),
    }
}
