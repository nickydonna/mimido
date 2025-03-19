use crate::{
    calendar_items::recur::parse_rrule, establish_connection, models::Event, util::stringify,
};
use chrono::DateTime;
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
        .iter()
        .filter_map(|event| {
            if !event.has_rrule {
                return Some(event);
            }

            let ical_event: icalendar::Event = event.try_into().ok()?;

            let r_rule = parse_rrule(&ical_event)?.after(parsed.with_timezone(&rrule::Tz::UTC));
            let is_valid_for_date = r_rule.all(2).dates.iter().any(|d| d >= &start && d <= &end);
            if is_valid_for_date {
                Some(event)
            } else {
                None
            }
        })
        .cloned()
        .collect::<Vec<Event>>();

    Ok(events)
}
