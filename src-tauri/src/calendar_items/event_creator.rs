use chrono::{DateTime, Duration, Utc};

use super::{
    date_parser::EventDate, event_status::EventStatus, event_type::EventType,
    rrule_parser::EventRecurrence, ExtractableFromInput, PropertyMatch,
};
use crate::models::{event::NewEvent, todo::NewTodo};

enum CalendarItem {
    Event(NewEvent),
    Todo(NewTodo),
}

pub struct EventDateInfo {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}
impl ExtractableFromInput for EventDateInfo {
    fn extract_from_input(
        date_of_input: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<PropertyMatch<Self>, String> {
        let start = EventDate::from_natural(input, date_of_input);
        let Some((start, end)) = start else {
            return Err("Failed to parse start date".to_string());
        };
        Ok(PropertyMatch::default(EventDateInfo {
            start,
            end: end.unwrap_or(start + Duration::minutes(30)),
        }))
    }
}

pub struct EventUpsertInfo {
    summary: String,
    date_info: EventDateInfo,
    recurrence: EventRecurrence,
    status: EventStatus,
    event_type: EventType,
    postponed: i32,
    urgency: i32,
    load: i32,
    priority: i32,
}

impl ExtractableFromInput for EventUpsertInfo {
    fn extract_from_input(
        date_of_input: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<PropertyMatch<Self>, String> {
        let (date_info, input) = remove_property_match(
            input,
            EventDateInfo::extract_from_input(date_of_input, input)?,
        );
        let (recurrence, input) = remove_property_match(
            &input,
            EventRecurrence::extract_from_input(date_of_input, &input)?,
        );
        let (status, input) = remove_property_match(
            &input,
            EventStatus::extract_from_input(date_of_input, &input)?,
        );
        let (event_type, input) = remove_property_match(
            &input,
            EventType::extract_from_input(date_of_input, &input)?,
        );

        Ok(PropertyMatch::default(EventUpsertInfo {
            summary: input.trim().to_string(),
            date_info,
            recurrence,
            status,
            event_type,
            postponed: 0,
            urgency: 0,
            load: 0,
            priority: 0,
        }))
    }
}

fn remove_property_match<T: Sized>(input: &str, prop_match: PropertyMatch<T>) -> (T, String) {
    let extracted = if let Some((start, end)) = prop_match.start_end {
        let s = &input[0..start];
        let e = &input[end..];
        format!("{} {}", s, e)
    } else {
        input.to_string()
    };
    (prop_match.property, extracted)
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
        let info = EventUpsertInfo::extract_from_input(date_of_input, input)
            .expect("To parse string")
            .property;
        let expected_date = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 7, 9, 0, 0)
            .unwrap()
            .to_utc();

        assert_eq!(info.summary, "Fly like an eagle tomorrow at 9");
        assert_eq!(info.status, EventStatus::Done);
        assert_eq!(info.event_type, EventType::Block);
        assert_eq!(info.date_info.start, expected_date);
        assert_eq!(info.date_info.end, expected_date + Duration::minutes(30));
        assert_eq!(info.recurrence.0, None);
    }

    #[test]
    fn should_parse_string_for_event_with_end() {
        let date_of_input = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 6, 10, 30, 0)
            .unwrap();
        let input = "@task print in 2 days at 10-11:30";
        let info = EventUpsertInfo::extract_from_input(date_of_input, input)
            .expect("To parse string")
            .property;
        let expected_date = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 8, 10, 0, 0)
            .unwrap()
            .to_utc();
        let expected_end = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 8, 11, 30, 0)
            .unwrap()
            .to_utc();

        assert_eq!(info.summary, "print in 2 days at 10-11:30");
        assert_eq!(info.status, EventStatus::Todo);
        assert_eq!(info.event_type, EventType::Task);
        assert_eq!(info.date_info.start, expected_date);
        assert_eq!(info.date_info.end, expected_end);
        assert_eq!(info.recurrence.0, None);
    }
}
