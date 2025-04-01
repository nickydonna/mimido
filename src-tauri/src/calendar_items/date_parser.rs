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
    SpecificTime,
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

const DEFAULT_TIME: (u32, u32) = (12, 0);

const TIME_RE: &str = r"at +(?P<time>\d{1,2}(?::\d{2})?)";
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
            DateExpressionCases::SpecificTime => r"at \d{1,2}(:\d{2})?".to_string(),
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

struct DateParser;

impl DateParser {
    fn parse_numbered_time(time_str: &str, starting_idx: usize) -> Option<(u32, u32)> {
        let time_str = time_str.to_lowercase();
        let time_re = Regex::new(TIME_RE).ok().expect("To Compile Regex");
        if time_str.len() <= starting_idx {
            return None;
        }
        let caps = time_re.captures(&time_str[starting_idx..])?;
        let time_str = caps.name("time")?.as_str();
        let parts = time_str.split(':').collect::<Vec<_>>();
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

        Some((hour, minute))
    }

    fn parse_named_time(time_str: &str, starting_idx: usize) -> Option<(u32, u32)> {
        let time_str = time_str.to_lowercase();
        let time_re = Regex::new(NAMED_TIME_RE).ok().expect("To Compile Regex");
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
        Some((hour, minute))
    }

    fn get_time(time_str: &str, starting_idx: usize) -> NaiveTime {
        let (hour, minute) = Self::parse_numbered_time(time_str, starting_idx)
            .or_else(|| Self::parse_named_time(time_str, starting_idx))
            .unwrap_or(DEFAULT_TIME);
        NaiveTime::from_hms_opt(hour, minute, 0).expect("Valid time")
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
        time: NaiveTime,
    ) -> Option<DateTime<Utc>> {
        let date = base + plus;
        let date = date.with_time(time);
        date.earliest().map(|d| d.to_utc())
    }

    pub fn from_natural(
        date_string: &str,
        reference_date: DateTime<Utc>,
        timezone: chrono_tz::Tz,
    ) -> Option<DateTime<Utc>> {
        let (case, re) = DateExpressionCases::iter()
            .map(|case| (case, Regex::from(case)))
            .find(|(_, re)| re.is_match(date_string))?;
        let base_time = reference_date.with_timezone(&timezone);

        let (duration, time): (TimeDelta, NaiveTime) = match case {
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
                let duration = weekday.days_since(base_time.weekday());
                let match_end = re.find(date_string)?.end();
                let time = Self::get_time(date_string, match_end);
                (Duration::days(duration as i64), time)
            }
            DateExpressionCases::SpecificTime => {
                let time = Self::get_time(date_string, 0);
                (Duration::days(0), time)
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

        Self::calculate_utc_date(base_time, duration, time)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    use super::*;

    fn create_test_date() -> DateTime<Utc> {
        // Create a fixed date for testing: 2024-03-15 12:00:00 UTC
        Utc.with_ymd_and_hms(2024, 3, 15, 12, 0, 0).unwrap()
    }

    #[test]
    fn test_tomorrow() {
        let reference = create_test_date();
        let result = DateParser::from_natural("tomorrow", reference, chrono_tz::Tz::UTC).unwrap();
        let expected = reference + Duration::days(1);
        assert_eq!(result.date_naive(), expected.date_naive());
    }

    #[test]
    fn test_tomorrow_with_time() {
        let reference = create_test_date();
        let result =
            DateParser::from_natural("tomorrow at 19", reference, chrono_tz::Tz::UTC).unwrap();
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
        let result =
            DateParser::from_natural("next monday", reference, chrono_tz::Tz::UTC).unwrap();
        let current_weekday = reference.weekday();
        let target_weekday = chrono::Weekday::Mon;
        let days_ahead = (target_weekday as i32 - current_weekday as i32 + 7) % 7;
        let expected = reference + Duration::days(days_ahead as i64);
        assert_eq!(result.date_naive(), expected.date_naive());
    }
    #[test]

    fn test_next_weekday_at_named_time() {
        let reference = create_test_date();
        let result =
            DateParser::from_natural("next monday afternoon", reference, chrono_tz::Tz::UTC)
                .unwrap();
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
        let result =
            DateParser::from_natural("tomorrow morning", reference, chrono_tz::Tz::UTC).unwrap();
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
        let result = DateParser::from_natural("in 3 days", reference, chrono_tz::Tz::UTC).unwrap();
        let expected = reference + Duration::days(3);
        assert_eq!(result.date_naive(), expected.date_naive());
    }

    #[test]
    fn test_specific_time() {
        let reference = create_test_date();
        let result = DateParser::from_natural("at 14:30", reference, chrono_tz::Tz::UTC).unwrap();
        let expected = reference
            .with_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
            .unwrap();
        assert_eq!(result.date_naive(), expected.date_naive());
        assert_eq!(result.hour(), expected.hour());
        assert_eq!(result.minute(), expected.minute());
    }

    #[test]
    fn test_invalid_date() {
        let reference = create_test_date();
        let result = DateParser::from_natural("something somethign", reference, chrono_tz::Tz::UTC);
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
            ("team sync at 14:00", reference, 14, 0),
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
            let result = DateParser::from_natural(input, reference, chrono_tz::Tz::UTC).unwrap();
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
            let result = DateParser::from_natural(input, reference, chrono_tz::Tz::UTC);
            assert!(result.is_none(), "Should fail for input: {}", input);
        }
    }
}

