use chrono::{DateTime, Duration, Utc};

use super::{
    date_parser::EventDate, event_status::EventStatus, event_type::EventType,
    rrule_parser::EventRecurrence,
};

use super::input_traits::ExtractableFromInput;
use crate::calendar_items::input_traits::ExtractedInput;
use crate::models::{event::NewEvent, todo::NewTodo};

enum CalendarItem {
    Event(NewEvent),
    Todo(NewTodo),
}

#[derive(Clone)]
pub struct EventDateInfo {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventDateOption(pub Option<EventDateInfo>);

impl ExtractableFromInput for EventDateOption {
    fn extract_from_input(
        date_of_input: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<impl Into<ExtractedInput<Self>>, String> {
        let dates = EventDate::from_natural(input, date_of_input);
        let Some((start, end, stripped)) = dates else {
            return Ok((EventDateOption(None), input.to_string()));
        };
        Ok((
            EventDateOption(Some(EventDateInfo {
                start,
                end: end.unwrap_or(start + Duration::minutes(30)),
            })),
            stripped,
        ))
    }
}

pub struct EventUpsertInfo {
    pub summary: String,
    pub date_info: EventDateOption,
    pub recurrence: EventRecurrence,
    pub status: EventStatus,
    pub event_type: EventType,
    pub postponed: i32,
    pub urgency: i32,
    pub load: i32,
    pub priority: i32,
}

impl ExtractableFromInput for EventUpsertInfo {
    fn extract_from_input(
        date_of_input: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<impl Into<ExtractedInput<Self>>, String> {
        let ExtractedInput(date_info, input) =
            EventDateOption::extract_from_input(date_of_input, input)?.into();
        let ExtractedInput(recurrence, input) =
            EventRecurrence::extract_from_input(date_of_input, &input)?.into();
        let ExtractedInput(status, input) =
            EventStatus::extract_from_input(date_of_input, &input)?.into();
        let ExtractedInput(event_type, input) =
            EventType::extract_from_input(date_of_input, &input)?.into();

        Ok((
            EventUpsertInfo {
                summary: input.trim().to_string(),
                date_info,
                recurrence,
                status,
                event_type,
                postponed: 0,
                urgency: 0,
                load: 0,
                priority: 0,
            },
            input,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn should_parse_string_for_event() {
        let date_of_input = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 6, 10, 30, 0)
            .unwrap();
        let input = "@block %done Fly like an eagle tomorrow at 9";
        let ExtractedInput(info, _) = EventUpsertInfo::extract_from_input(date_of_input, input)
            .expect("To parse string")
            .into();
        let expected_date = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 7, 9, 0, 0)
            .unwrap()
            .to_utc();

        let EventDateInfo { start, end } = info.date_info.0.unwrap();

        assert_eq!(info.summary, "Fly like an eagle");
        assert_eq!(info.status, EventStatus::Done);
        assert_eq!(info.event_type, EventType::Block);
        assert_eq!(start, expected_date);
        assert_eq!(end, expected_date + Duration::minutes(30));
        assert_eq!(info.recurrence.0, None);
    }

    #[test]
    fn should_parse_string_for_event_recurrence() {
        let date_of_input = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 6, 10, 30, 0)
            .unwrap();
        let input = "@block %done Fly like an eagle tomorrow at 9 every weekday";
        let ExtractedInput(info, _) = EventUpsertInfo::extract_from_input(date_of_input, input)
            .expect("To parse string")
            .into();
        let expected_date = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 7, 9, 0, 0)
            .unwrap()
            .to_utc();

        let EventDateInfo { start, end } = info.date_info.0.unwrap();

        assert_eq!(info.summary, "Fly like an eagle");
        assert_eq!(info.status, EventStatus::Done);
        assert_eq!(info.event_type, EventType::Block);
        assert_eq!(start, expected_date);
        assert_eq!(end, expected_date + Duration::minutes(30));
        assert!(info.recurrence.0.is_some());
    }

    #[test]
    fn should_parse_string_for_event_with_end() {
        let date_of_input = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 6, 10, 30, 0)
            .unwrap();
        let input = "@task print in 2 days at 10-11:30";
        let ExtractedInput(info, _) = EventUpsertInfo::extract_from_input(date_of_input, input)
            .expect("To parse string")
            .into();
        let expected_date = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 8, 10, 0, 0)
            .unwrap()
            .to_utc();
        let expected_end = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 8, 11, 30, 0)
            .unwrap()
            .to_utc();

        let EventDateInfo { start, end } = info.date_info.0.unwrap();

        assert_eq!(info.summary, "print");
        assert_eq!(info.status, EventStatus::Todo);
        assert_eq!(info.event_type, EventType::Task);
        assert_eq!(start, expected_date);
        assert_eq!(end, expected_end);
        assert_eq!(info.recurrence.0, None);
    }
}
