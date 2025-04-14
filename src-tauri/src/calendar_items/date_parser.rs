use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeDelta, Utc};
use regex::{Regex, RegexBuilder};
use strum::IntoEnumIterator;

#[derive(Copy, Clone, Debug, strum_macros::EnumIter)]
enum DateExpressionCases {
    Tomorrow,
    Today,
    NextWeek,
    NextWeekday,
    RelativeTime,
}

#[derive(Copy, Clone, Debug, strum_macros::EnumString)]
#[strum(serialize_all = "lowercase")]
enum NamedTime {
    Morning,
    Afternoon,
    Evening,
    Night,
    Noon,
    Midnight,
}

const WEEKDAYS: &[&str] = &[
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
    "Mon",
    "Tue",
    "Wed",
    "Thu",
    "Fri",
    "Sat",
    "Sun",
];

const TIME_RE: &str = r"at +(?P<time>\d{1,2}(?::\d{2})?)";
const FROM_TO_RE: &str =
    r"(?P<start>\d{1,2}(?::\d{2})?) *(:?-|to|until) *(?P<end>\d{1,2}(?::\d{2})?)";
const NAMED_TIME_RE: &str = r"(at +)?(?P<time>morning|noon|afternoon|night|evening|midnight)";

impl From<DateExpressionCases> for Regex {
    fn from(value: DateExpressionCases) -> Self {
        let re_str = match value {
            DateExpressionCases::Tomorrow => r"tomorrow".to_string(),
            DateExpressionCases::Today => r"today".to_string(),
            DateExpressionCases::NextWeek => r"next week".to_string(),
            DateExpressionCases::NextWeekday => {
                format!(r"next +(?P<weekday>{})", WEEKDAYS.join("|"))
            }
            DateExpressionCases::RelativeTime => {
                r"in (?P<number>\d+) +(?P<unit>day|week|month|year)s?".to_string()
            }
        };
        RegexBuilder::new(&re_str)
            .case_insensitive(true)
            .build()
            .expect("Regex to compile")
    }
}

pub struct EventDate;

impl EventDate {
    fn parse_numbered_time_match(match_str: &str) -> Option<NaiveTime> {
        let parts = match_str.split(':').collect::<Vec<_>>();
        let hour: u32 = parts[0].parse().ok()?;
        let minute: u32 = if parts.len() > 1 {
            parts[1].parse().ok()?
        } else {
            0
        };

        if hour > 23 || minute > 59 {
            log::warn!("Invalid time: {}:{}", hour, minute);
            return None;
        }

        NaiveTime::from_hms_opt(hour, minute, 0)
    }

    fn parse_numbered_time(time_str: &str, starting_idx: usize) -> Option<NaiveTime> {
        let time_str = time_str.to_lowercase();
        let time_re = Regex::new(TIME_RE).expect("To Compile Regex");
        if time_str.len() <= starting_idx {
            return None;
        }
        let caps = time_re.captures(&time_str[starting_idx..])?;
        let time_str = caps.name("time")?.as_str();
        Self::parse_numbered_time_match(time_str)
    }

    fn parse_from_to(time_str: &str, starting_idx: usize) -> Option<(NaiveTime, NaiveTime)> {
        let time_str = time_str.to_lowercase();
        let time_re = Regex::new(FROM_TO_RE).expect("To Compile Regex");
        if time_str.len() <= starting_idx {
            return None;
        }
        let caps = time_re.captures(&time_str[starting_idx..])?;
        let start_time = caps.name("start")?.as_str();
        let start_time = Self::parse_numbered_time_match(start_time)?;
        let end_time = caps.name("end")?.as_str();
        let end_time = Self::parse_numbered_time_match(end_time)?;
        Some((start_time, end_time))
    }

    fn parse_named_time(time_str: &str, starting_idx: usize) -> Option<NaiveTime> {
        let time_str = time_str.to_lowercase();
        let time_re = Regex::new(NAMED_TIME_RE).expect("To Compile Regex");
        if time_str.len() <= starting_idx {
            return None;
        }
        let caps = time_re.captures(&time_str[starting_idx..])?;
        let time_str = caps.name("time")?.as_str();
        let named_time = NamedTime::from_str(time_str).ok()?;
        let (hour, minute) = match named_time {
            NamedTime::Morning => (8, 0),
            NamedTime::Afternoon => (16, 0),
            NamedTime::Evening => (18, 0),
            NamedTime::Night => (22, 0),
            NamedTime::Noon => (12, 0),
            NamedTime::Midnight => (0, 0),
        };

        NaiveTime::from_hms_opt(hour, minute, 0)
    }

    fn get_time(time_str: &str, starting_idx: usize) -> (NaiveTime, Option<NaiveTime>) {
        let range = Self::parse_from_to(time_str, starting_idx);
        if let Some((start, end)) = range {
            return (start, Some(end));
        }
        let start = Self::parse_numbered_time(time_str, starting_idx)
            .or_else(|| Self::parse_named_time(time_str, starting_idx));
        if let Some(start) = start {
            return (start, None);
        }
        (NaiveTime::from_hms_opt(12, 0, 0).unwrap(), None)
    }

    fn parse_relative_time(number: u32, unit: &str) -> Option<Duration> {
        match unit {
            "day" => Some(Duration::days(number as i64)),
            "week" => Some(Duration::weeks(number as i64)),
            _ => None,
        }
    }

    fn calculate_utc_date(
        base: DateTime<chrono_tz::Tz>,
        plus: Duration,
        start: NaiveTime,
        end: Option<NaiveTime>,
    ) -> Option<(DateTime<Utc>, Option<DateTime<Utc>>)> {
        let date = base + plus;
        let datetime = date.with_time(start);
        let start = datetime.earliest().map(|d| d.to_utc())?;
        let end = end
            .and_then(|end| date.with_time(end).earliest())
            .map(|d| d.to_utc());
        Some((start, end))
    }

    pub fn from_natural(
        date_string: &str,
        reference_date: DateTime<chrono_tz::Tz>,
    ) -> Option<(DateTime<Utc>, Option<DateTime<Utc>>)> {
        let (case, re) = DateExpressionCases::iter()
            .map(|case| (case, Regex::from(case)))
            .find(|(_, re)| re.is_match(date_string))?;

        let (duration, (start, end)): (TimeDelta, (NaiveTime, Option<NaiveTime>)) = match case {
            DateExpressionCases::Tomorrow => {
                let match_end = re.find(date_string)?.end();
                let time = Self::get_time(date_string, match_end);
                (Duration::days(1), time)
            }
            DateExpressionCases::Today => {
                let match_end = re.find(date_string)?.end();
                let time = Self::get_time(date_string, match_end);
                (Duration::days(0), time)
            }
            DateExpressionCases::NextWeek => {
                let match_end = re.find(date_string)?.end();
                let time = Self::get_time(date_string, match_end);
                (Duration::weeks(1), time)
            }
            DateExpressionCases::NextWeekday => {
                let weekday = re
                    .captures(date_string)?
                    .name("weekday")?
                    .as_str()
                    .parse::<chrono::Weekday>()
                    .ok()?;
                let duration = weekday.days_since(reference_date.weekday());
                let match_end = re.find(date_string)?.end();
                let time = Self::get_time(date_string, match_end);
                (Duration::days(duration as i64), time)
            }
            DateExpressionCases::RelativeTime => {
                let match_end = re.find(date_string)?.end();
                let caps = re.captures(date_string)?;
                let (Some(number), Some(unit)) = (caps.name("number"), caps.name("unit")) else {
                    return None;
                };
                let num = number.as_str().parse::<u32>().ok()?;
                let duration = Self::parse_relative_time(num, unit.as_str())?;
                let time = Self::get_time(date_string, match_end);
                (duration, time)
            }
        };

        Self::calculate_utc_date(reference_date, duration, start, end)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    use super::*;

    fn create_test_date() -> DateTime<chrono_tz::Tz> {
        // Create a fixed date for testing: 2024-03-15 12:00:00 UTC
        Utc.with_ymd_and_hms(2024, 3, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&chrono_tz::Tz::UTC)
    }

    #[test]
    fn test_tomorrow() {
        let reference = create_test_date();
        let result = EventDate::from_natural("tomorrow", reference).unwrap().0;
        let expected = reference + Duration::days(1);
        assert_eq!(result.date_naive(), expected.date_naive());
    }

    #[test]
    fn test_tomorrow_with_time() {
        let reference = create_test_date();
        let result = EventDate::from_natural("tomorrow at 19", reference)
            .unwrap()
            .0;
        let expected = (reference + Duration::days(1))
            .with_hour(19)
            .unwrap()
            .with_minute(0)
            .unwrap();
        assert_eq!(result.date_naive(), expected.date_naive());
        assert_eq!(result.hour(), expected.hour());
        assert_eq!(result.minute(), expected.minute());
    }

    #[test]
    fn test_next_weekday() {
        let reference = create_test_date();
        let result = EventDate::from_natural("next monday", reference).unwrap().0;
        let current_weekday = reference.weekday();
        let target_weekday = chrono::Weekday::Mon;
        let days_ahead = (target_weekday as i32 - current_weekday as i32 + 7) % 7;
        let expected = reference + Duration::days(days_ahead as i64);
        assert_eq!(result.date_naive(), expected.date_naive());
    }
    #[test]

    fn test_next_weekday_at_named_time() {
        let reference = create_test_date();
        let result = EventDate::from_natural("next monday afternoon", reference)
            .unwrap()
            .0;
        let current_weekday = reference.weekday();
        let target_weekday = chrono::Weekday::Mon;
        let days_ahead = (target_weekday as i32 - current_weekday as i32 + 7) % 7;
        let expected = reference + Duration::days(days_ahead as i64);
        let expected = expected
            .with_time(NaiveTime::from_hms_opt(16, 0, 0).unwrap())
            .unwrap();
        assert_eq!(result.date_naive(), expected.date_naive());
        assert_eq!(result.hour(), expected.hour());
        assert_eq!(result.minute(), expected.minute());
    }

    #[test]
    fn test_tomorrow_at_named_time() {
        let reference = create_test_date();
        let result = EventDate::from_natural("tomorrow morning", reference)
            .unwrap()
            .0;
        let expected = reference + Duration::days(1);
        let expected = expected
            .with_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap())
            .unwrap();
        assert_eq!(result.date_naive(), expected.date_naive());
        assert_eq!(result.hour(), expected.hour());
        assert_eq!(result.minute(), expected.minute());
    }

    #[test]
    fn test_relative_time() {
        let reference = create_test_date();
        let result = EventDate::from_natural("in 3 days", reference).unwrap().0;
        let expected = reference + Duration::days(3);
        assert_eq!(result.date_naive(), expected.date_naive());
    }

    #[test]
    fn test_specific_time() {
        let reference = create_test_date();
        let result = EventDate::from_natural("today at 14:30", reference)
            .unwrap()
            .0;
        let expected = reference
            .with_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
            .unwrap();
        assert_eq!(result.date_naive(), expected.date_naive());
        assert_eq!(result.hour(), expected.hour());
        assert_eq!(result.minute(), expected.minute());
    }

    #[test]
    fn test_range_dash() {
        let reference = create_test_date();
        let (start, end) = EventDate::from_natural("today at 14:30-16", reference).unwrap();
        let expected = reference
            .with_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
            .unwrap();
        assert_eq!(start.date_naive(), expected.date_naive());
        assert_eq!(start.hour(), expected.hour());
        assert_eq!(start.minute(), expected.minute());

        let expected = reference
            .with_time(NaiveTime::from_hms_opt(16, 0, 0).unwrap())
            .unwrap();
        assert!(end.is_some());
        let end = end.unwrap();
        assert_eq!(end.date_naive(), expected.date_naive());
        assert_eq!(end.hour(), expected.hour());
        assert_eq!(end.minute(), expected.minute());
    }

    #[test]
    fn test_range_until() {
        let reference = create_test_date();
        let (start, end) =
            EventDate::from_natural("steve lepoisson today from 18:45 until 19", reference)
                .unwrap();
        let expected = reference
            .with_time(NaiveTime::from_hms_opt(18, 45, 0).unwrap())
            .unwrap();
        assert_eq!(start.date_naive(), expected.date_naive());
        assert_eq!(start.hour(), expected.hour());
        assert_eq!(start.minute(), expected.minute());

        let expected = reference
            .with_time(NaiveTime::from_hms_opt(19, 0, 0).unwrap())
            .unwrap();
        assert!(end.is_some());
        let end = end.unwrap();
        assert_eq!(end.date_naive(), expected.date_naive());
        assert_eq!(end.hour(), expected.hour());
        assert_eq!(end.minute(), expected.minute());
    }
    #[test]
    fn test_range_to() {
        let reference = create_test_date();
        let (start, end) = EventDate::from_natural("today from 09:30-12:16", reference).unwrap();
        let expected = reference
            .with_time(NaiveTime::from_hms_opt(9, 30, 0).unwrap())
            .unwrap();
        assert_eq!(start.date_naive(), expected.date_naive());
        assert_eq!(start.hour(), expected.hour());
        assert_eq!(start.minute(), expected.minute());

        let expected = reference
            .with_time(NaiveTime::from_hms_opt(12, 16, 0).unwrap())
            .unwrap();
        assert!(end.is_some());
        let end = end.unwrap();
        assert_eq!(end.date_naive(), expected.date_naive());
        assert_eq!(end.hour(), expected.hour());
        assert_eq!(end.minute(), expected.minute());
    }

    #[test]
    fn test_invalid_date() {
        let reference = create_test_date();
        let result = EventDate::from_natural("something somethign", reference);
        assert!(result.is_none());
    }

    #[test]
    fn test_full_strings() {
        let reference = create_test_date();

        // Test various natural language strings
        let test_cases = vec![
            (
                "morning working tomorrow at 10",
                reference + Duration::days(1),
                10,
                0,
            ),
            (
                "call chris next week",
                reference + Duration::weeks(1),
                12,
                0,
            ),
            ("meeting with team today at 15:30", reference, 15, 30),
            (
                "doctor appointment next monday",
                {
                    let current_weekday = reference.weekday();
                    let target_weekday = chrono::Weekday::Mon;
                    let days_ahead = (target_weekday as i32 - current_weekday as i32 + 7) % 7;
                    reference + Duration::days(days_ahead as i64)
                },
                12,
                0,
            ),
            (
                "project deadline in 3 days",
                reference + Duration::days(3),
                12,
                0,
            ),
            ("team sync today at 14:00", reference, 14, 0),
            (
                "team sync tomorrow morning",
                reference + Duration::days(1),
                8,
                0,
            ),
            (
                "lunch meeting tomorrow",
                reference + Duration::days(1),
                12,
                0,
            ),
            (
                "client call next tuesday at 16:45",
                {
                    let current_weekday = reference.weekday();
                    let target_weekday = chrono::Weekday::Tue;
                    let days_ahead = (target_weekday as i32 - current_weekday as i32 + 7) % 7;
                    reference + Duration::days(days_ahead as i64)
                },
                16,
                45,
            ),
        ];

        for (input, expected_date, expected_hour, expected_minute) in test_cases {
            let result = EventDate::from_natural(input, reference).unwrap().0;
            assert_eq!(
                result.date_naive(),
                expected_date.date_naive(),
                "Date mismatch for input: {}",
                input
            );
            assert_eq!(
                result.hour(),
                expected_hour,
                "Hour mismatch for input: {}",
                input
            );
            assert_eq!(
                result.minute(),
                expected_minute,
                "Minute mismatch for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_invalid_full_strings() {
        let reference = create_test_date();

        let invalid_cases = vec![
            "invalid date string",
            "meeting at invalid time",
            "next invalid day",
        ];

        for input in invalid_cases {
            let result = EventDate::from_natural(input, reference);
            assert!(result.is_none(), "Should fail for input: {}", input);
        }
    }
}
