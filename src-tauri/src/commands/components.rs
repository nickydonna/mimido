use crate::{
    calendar_items::{event_type::EventType, recur::parse_rrule},
    establish_connection,
    models::Event,
    util::stringify,
};
use chrono::{DateTime, Days};
use diesel::prelude::*;
use now::DateTimeNow;

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn list_events_for_day(datetime: String) -> Result<Vec<Event>, String> {
    use crate::schema::events::dsl as event_dsl;

    let conn = &mut establish_connection();

    let parsed = DateTime::parse_from_rfc3339(&datetime)
        .map_err(stringify)?
        .to_utc();
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
        .into_iter()
        .filter_map(|event| {
            if !event.has_rrule {
                return Some(event.to_owned());
            }
            let duration = event.ends_at - event.starts_at;

            let ical_event: icalendar::Event = event.clone().try_into().ok()?;

            let r_rule = parse_rrule(&ical_event)?
                .after(parsed.with_timezone(&rrule::Tz::UTC) - Days::new(1));
            let rrecurence = r_rule
                .all(2)
                .dates
                .into_iter()
                .find(|d| d >= &start && d <= &end);
            if let Some(date) = rrecurence {
                let new_event = Event {
                    starts_at: date.to_utc(),
                    ends_at: (date + duration).to_utc(),
                    ..event
                };
                Some(new_event.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<Event>>();

    Ok(events)
}
