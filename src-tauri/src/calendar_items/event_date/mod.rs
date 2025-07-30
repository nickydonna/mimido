use chrono::{DateTime, TimeDelta, TimeZone};

mod date_parser;
mod rrule_parser;

use self::date_parser::extract_start_end;
use crate::calendar_items::{
    event_date::date_parser::start_end_to_natural,
    event_type::EventType,
    input_traits::{ExtractableFromInput, ExtractedInput, ToInput},
};

// Re-export
pub use self::rrule_parser::EventRecurrence;

#[derive(Clone)]
pub struct EventDateInfo<Tz: TimeZone> {
    pub start: DateTime<Tz>,
    pub end: Option<DateTime<Tz>>,
    pub recurrence: EventRecurrence,
}

impl<Tz: TimeZone> EventDateInfo<Tz> {
    pub fn new(
        start: DateTime<Tz>,
        end: DateTime<Tz>,
        recurrence: impl Into<EventRecurrence>,
    ) -> Self {
        Self {
            start,
            end: Some(end),
            recurrence: recurrence.into(),
        }
    }

    pub fn once(start: DateTime<Tz>, end: DateTime<Tz>) -> Self {
        Self::new(start, end, EventRecurrence::none())
    }

    pub fn get_end_or_default(&self, event_type: EventType) -> DateTime<Tz> {
        if let Some(end) = &self.end {
            end.clone()
        } else {
            let duration = match event_type {
                EventType::Event => TimeDelta::hours(1),
                EventType::Block => TimeDelta::hours(1),
                EventType::Reminder => TimeDelta::minutes(15),
                EventType::Task => TimeDelta::minutes(30),
            };
            self.start.clone() + duration
        }
    }
}

#[derive(Clone)]
pub struct EventDateOption<Tz: TimeZone>(pub Option<EventDateInfo<Tz>>);

impl<Tz: TimeZone> EventDateInfo<Tz> {
    pub fn get_recurrence_as_cal_property(self) -> Option<String> {
        let rule_set = self.recurrence.0?;
        let rule = rule_set.get_rrule().first()?;
        Some(rule.to_string())
    }
}

impl<Tz: TimeZone> ExtractableFromInput<Tz> for EventDateOption<Tz> {
    fn extract_from_input(
        date_of_input: DateTime<Tz>,
        input: &str,
    ) -> anyhow::Result<impl Into<ExtractedInput<Self>>> {
        let dates = extract_start_end(input, date_of_input);
        let Some((start, end, stripped)) = dates else {
            return Ok((EventDateOption(None), input.to_string()));
        };

        let rrule = EventRecurrence::from_natural(&stripped, &start);
        match rrule {
            Some((rrule, recur_stripped)) => Ok((
                EventDateOption(Some(EventDateInfo {
                    start,
                    end,
                    recurrence: EventRecurrence::some(rrule),
                })),
                recur_stripped,
            )),
            None => Ok((
                EventDateOption(Some(EventDateInfo {
                    start,
                    end,
                    recurrence: EventRecurrence::none(),
                })),
                stripped,
            )),
        }
    }
}

impl<Tz: TimeZone> ToInput<Tz> for EventDateInfo<Tz> {
    fn to_input(&self, reference_date: &DateTime<Tz>) -> String {
        let base = start_end_to_natural(
            reference_date,
            &self.start,
            &self
                .end
                .clone()
                .expect("Can't make input from date with no end"),
        );
        match self.recurrence.to_natural_language().ok() {
            Some(recurrence_str) => format!("{base} {recurrence_str}"),
            None => base,
        }
    }
}
