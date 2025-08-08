use chrono::{DateTime, TimeZone};

use super::input_traits::FromUserInput;
use crate::calendar_items::event_date::EventDateOption;
use crate::calendar_items::event_status::EventStatus;
use crate::calendar_items::event_tags::EventTags;
use crate::calendar_items::event_type::EventType;
use crate::calendar_items::input_traits::ExtractedInput;

/// Struct that holds information for updating or upserting an event
pub struct EventUpsertInfo<Tz: TimeZone> {
    pub summary: String,
    pub date_info: EventDateOption<Tz>,
    pub status: EventStatus,
    pub event_type: EventType,
    pub postponed: i32,
    pub urgency: i32,
    pub load: i32,
    pub importance: i32,
    pub tag: EventTags,
}

impl<Tz: TimeZone> FromUserInput<Tz> for EventUpsertInfo<Tz> {
    fn extract_from_input(
        date_of_input: DateTime<Tz>,
        input: &str,
    ) -> anyhow::Result<impl Into<ExtractedInput<Self>>> {
        let ExtractedInput(date_info, input) =
            EventDateOption::extract_from_input(date_of_input.clone(), input)?.into();
        let ExtractedInput(status, input) =
            EventStatus::extract_from_input(date_of_input.clone(), &input)?.into();
        let ExtractedInput(event_type, input) =
            EventType::extract_from_input(date_of_input.clone(), &input)?.into();
        let ExtractedInput(tag, input) =
            EventTags::extract_from_input(date_of_input.clone(), &input)?.into();

        Ok((
            EventUpsertInfo {
                summary: input.trim().to_string(),
                date_info,
                status,
                event_type,
                postponed: 0,
                urgency: 0,
                load: 0,
                importance: 0,
                tag,
            },
            input,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::calendar_items::event_date::{EventDateInfo, EventRecurrence};

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

        let EventDateInfo {
            start,
            end,
            recurrence,
        } = info.date_info.0.unwrap();

        assert_eq!(info.summary, "Fly like an eagle");
        assert_eq!(info.status, EventStatus::Done);
        assert_eq!(info.event_type, EventType::Block);
        assert_eq!(start, expected_date);
        assert_eq!(end, None);
        assert_eq!(recurrence, EventRecurrence(None));
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

        let EventDateInfo {
            start,
            end,
            recurrence,
        } = info.date_info.0.unwrap();

        assert_eq!(info.summary, "Fly like an eagle");
        assert_eq!(info.status, EventStatus::Done);
        assert_eq!(info.event_type, EventType::Block);
        assert_eq!(start, expected_date);
        assert_eq!(end, None);
        assert!(recurrence.0.is_some());
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
            .unwrap();

        let EventDateInfo {
            start,
            end,
            recurrence,
        } = info.date_info.0.unwrap();

        assert_eq!(info.summary, "print");
        assert_eq!(info.status, EventStatus::Todo);
        assert_eq!(info.event_type, EventType::Task);
        assert_eq!(start, expected_date);
        assert_eq!(end, Some(expected_end));
        assert_eq!(recurrence, EventRecurrence::none());
    }

    #[test]
    fn should_parse_string_with_tags() {
        let date_of_input = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 6, 10, 30, 0)
            .unwrap();
        let input = "@task print in 2 days at 10-11:30 #hello";
        let ExtractedInput(info, _) = EventUpsertInfo::extract_from_input(date_of_input, input)
            .expect("To parse string")
            .into();
        let expected_date = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 8, 10, 0, 0)
            .unwrap();
        let expected_end = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 8, 11, 30, 0)
            .unwrap();

        let EventDateInfo {
            start,
            end,
            recurrence,
        } = info.date_info.0.unwrap();

        assert_eq!(info.summary, "print");
        assert_eq!(info.status, EventStatus::Todo);
        assert_eq!(info.event_type, EventType::Task);
        assert_eq!(start, expected_date);
        assert_eq!(end, Some(expected_end));
        assert_eq!(recurrence, EventRecurrence::none());
        assert_eq!(info.tag, EventTags(Some("hello".to_string())))
    }

    #[test]
    fn should_parse_full_example() {
        let date_of_input = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 6, 10, 30, 0)
            .unwrap();
        let input = ".reminder %todo Dientes at 30/07/25 09:00-09:15 every weekday #health";
        let ExtractedInput(info, _) = EventUpsertInfo::extract_from_input(date_of_input, input)
            .expect("To parse string")
            .into();
        let expected_date = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 7, 30, 9, 0, 0)
            .unwrap()
            .to_utc();
        let expected_end = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 7, 30, 9, 15, 0)
            .unwrap();

        let EventDateInfo {
            start,
            end,
            recurrence,
        } = info.date_info.0.unwrap();

        assert_eq!(info.summary, "Dientes");
        assert_eq!(info.status, EventStatus::Todo);
        assert_eq!(info.event_type, EventType::Reminder);
        assert_eq!(start, expected_date);
        assert_eq!(end, Some(expected_end));
        assert!(recurrence.0.is_some());
        assert_eq!(info.tag, EventTags(Some("health".to_string())))
    }
}
