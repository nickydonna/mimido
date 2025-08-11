use chrono::{DateTime, TimeZone, Utc};
use now::DateTimeNow;

use crate::{
    calendar_items::input_traits::ToUserInput,
    models::vtodo::{VTodo, VTodoTrait},
};

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct ExtendedTodo {
    /// Date when the extended event was calculated
    pub query_date: DateTime<Utc>,
    pub todo: VTodo,
    /// The start date of the event, if recurrent the value for the current query
    pub starts_at: DateTime<Utc>,
    /// The end date of the event, if recurrent the value for the current query
    pub ends_at: DateTime<Utc>,
    pub natural_recurrence: Option<String>,
    pub natural_string: String,
}

impl ExtendedTodo {
    pub fn on_day<Tz: TimeZone>(todo: &VTodo, query_date: &DateTime<Tz>) -> Option<Self> {
        let start = todo.starts_at?;
        if !todo.has_rrule && start.date_naive() != query_date.date_naive() {
            log::warn!(
                "Todo {} does not have a recurrence rule and is not on the requested date",
                todo.uid
            );
            return None;
        }
        let base = query_date.beginning_of_day();
        let (starts_at, ends_at) = todo.get_start_end_for_date(&base)?;
        if starts_at > base && starts_at < query_date.end_of_day() {
            Some(Self {
                query_date: query_date.to_utc(),
                todo: todo.clone(),
                starts_at: starts_at.to_utc(),
                ends_at: ends_at.to_utc(),
                natural_recurrence: None,
                natural_string: todo.to_input(query_date),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct UnscheduledTodo {
    pub todo: VTodo,
    pub natural_string: String,
}

impl UnscheduledTodo {
    // Proving dates for [`ToUserInput`] trait
    // It will also help with relative dates for completing in the future
    pub fn on_day<Tz: TimeZone>(todo: &VTodo, query_date: &DateTime<Tz>) -> Self {
        Self {
            todo: todo.clone(),
            natural_string: todo.to_input(query_date),
        }
    }
}
