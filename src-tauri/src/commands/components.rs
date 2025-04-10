use crate::{
    establish_connection,
    models::event::{Event, EventTrait},
    util::stringify,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use now::DateTimeNow;

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
        if !event.has_rrule {
            return Some(Self {
                event: event.clone(),
                starts_at: event.starts_at,
                ends_at: event.ends_at,
                natural_recurrence: None,
            });
        }

        let duration = event.ends_at - event.starts_at;

        let occurance_date = event.get_recurrence_for_date(query_date)?;
        event.get_occurrence_natural().map(|nat| Self {
            event: event.clone(),
            starts_at: occurance_date.to_utc(),
            ends_at: (occurance_date + duration).to_utc(),
            natural_recurrence: Some(nat),
        })
    }
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn list_events_for_day(datetime: String) -> Result<Vec<ExtendedEvent>, String> {
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
        .filter_map(|event| ExtendedEvent::on_day(event, parsed))
        .collect::<Vec<ExtendedEvent>>();

    Ok(events)
}
