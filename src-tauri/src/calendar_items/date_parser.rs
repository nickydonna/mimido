use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use regex::{Match, Regex, RegexBuilder};
use strum::IntoEnumIterator;

#[derive(Copy, Clone, Debug, strum_macros::EnumIter)]
enum DateExpressionCases {
    AbsoluteDates,
    AbsoluteRange,
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
    r"(at +)?(?P<start>\d{1,2}(?::\d{2})?) *(:?-|to|until) *(?P<end>\d{1,2}(?::\d{2})?)";
const NAMED_TIME_RE: &str = r"(at +)?(?P<time>morning|noon|afternoon|night|evening|midnight)";

impl From<DateExpressionCases> for Regex {
    fn from(value: DateExpressionCases) -> Self {
        let re_str = match value {
            DateExpressionCases::AbsoluteDates => r"at (?P<start>\d{2}\/\d{2}\/\d{2} \d{2}:\d{2})-(?P<end>\d{2}\/\d{2}\/\d{2} \d{2}:\d{2})".to_string(),
            DateExpressionCases::AbsoluteRange => r"at (?P<date>\d{2}\/\d{2}\/\d{2}) (?P<start_time>\d{2}:\d{2})-(?P<end_time>\d{2}:\d{2})".to_string(),
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

fn parse_numbered_time_match(match_str: &str) -> Option<NaiveTime> {
    let parts = match_str.split(':').collect::<Vec<_>>();
    let hour: u32 = parts[0].parse().ok()?;
    let minute: u32 = if parts.len() > 1 {
        parts[1].parse().ok()?
    } else {
        0
    };

    if hour > 23 || minute > 59 {
        log::warn!("Invalid time: {hour}:{minute}");
        return None;
    }

    NaiveTime::from_hms_opt(hour, minute, 0)
}

fn extract_numbered_time(time_str: &str) -> Option<(NaiveTime, String)> {
    let time_re = Regex::new(TIME_RE).expect("To Compile Regex");
    let caps = time_re.captures(time_str)?;
    let matched = caps.get(0)?;
    let named = caps.name("time")?.as_str();
    let time = parse_numbered_time_match(named)?;
    Some((time, remove_matched(time_str, matched)))
}

fn extract_from_to(time_str: &str) -> Option<(NaiveTime, NaiveTime, String)> {
    let time_re = Regex::new(FROM_TO_RE).expect("To Compile Regex");
    let caps = time_re.captures(time_str)?;
    let matched = caps.get(0)?;
    let start_time = caps.name("start")?.as_str();
    let start_time = parse_numbered_time_match(start_time)?;
    let end_time = caps.name("end")?.as_str();
    let end_time = parse_numbered_time_match(end_time)?;
    Some((start_time, end_time, remove_matched(time_str, matched)))
}

fn extract_named_time(time_str: &str) -> Option<(NaiveTime, String)> {
    let time_re = Regex::new(NAMED_TIME_RE).expect("To Compile Regex");
    let caps = time_re.captures(time_str)?;
    let matched = caps.get(0)?;
    let named = caps.name("time")?.as_str();
    let named_time = NamedTime::from_str(named).ok()?;
    let (hour, minute) = match named_time {
        NamedTime::Morning => (8, 0),
        NamedTime::Afternoon => (16, 0),
        NamedTime::Evening => (18, 0),
        NamedTime::Night => (22, 0),
        NamedTime::Noon => (12, 0),
        NamedTime::Midnight => (0, 0),
    };

    let time = NaiveTime::from_hms_opt(hour, minute, 0)?;

    Some((time, remove_matched(time_str, matched)))
}

fn extract_time(time_str: &str) -> (NaiveTime, Option<NaiveTime>, String) {
    let range = extract_from_to(time_str);
    if let Some((start, end, stripped_string)) = range {
        return (start, Some(end), stripped_string);
    }
    let start = extract_numbered_time(time_str).or_else(|| extract_named_time(time_str));
    if let Some((start, stripped_string)) = start {
        return (start, None, stripped_string);
    }
    (
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        None,
        time_str.to_string(),
    )
}

fn parse_relative_time(number: u32, unit: &str) -> Option<Duration> {
    match unit {
        "day" => Some(Duration::days(number as i64)),
        "week" => Some(Duration::weeks(number as i64)),
        _ => None,
    }
}

fn calculate_date<Tz: TimeZone>(
    reference_date: DateTime<Tz>,
    addition_unit: Duration,
    start_time: NaiveTime,
    end_time: Option<NaiveTime>,
) -> Option<(DateTime<Tz>, Option<DateTime<Tz>>)> {
    let date = reference_date + addition_unit;
    let datetime = date.with_time(start_time);
    let start = datetime.earliest()?;
    let end = end_time.and_then(|end| date.with_time(end).earliest());
    println!("{} -> {}", date.to_rfc2822(), start.to_rfc2822());
    Some((start, end))
}

fn remove_matched(input: &str, matched: Match<'_>) -> String {
    format!("{} {}", &input[0..matched.start()], &input[matched.end()..])
        .trim()
        .to_string()
}

pub fn extract_start_end<Tz: TimeZone>(
    date_string: &str,
    reference_date: DateTime<Tz>,
) -> Option<(DateTime<Tz>, Option<DateTime<Tz>>, String)> {
    let tz = reference_date.timezone();
    let (case, re) = DateExpressionCases::iter()
        .map(|case| (case, Regex::from(case)))
        .find(|(_, re)| re.is_match(date_string))?;

    let (start, end, stripped_string): (DateTime<Tz>, Option<DateTime<Tz>>, String) = match case {
        DateExpressionCases::AbsoluteDates => {
            let matched = re.captures(date_string)?;
            let start = matched.name("start")?.as_str();
            let start = NaiveDateTime::parse_from_str(start, "%d/%m/%y %H:%M").ok()?;
            let start = tz.from_local_datetime(&start).earliest()?;
            let end = matched.name("end")?.as_str();
            let end = NaiveDateTime::parse_from_str(end, "%d/%m/%y %H:%M").ok()?;
            let end = tz.from_local_datetime(&end).earliest()?;
            let stripped = remove_matched(date_string, matched.get(0)?);
            (start, Some(end), stripped)
        }
        DateExpressionCases::AbsoluteRange => {
            let matched = re.captures(date_string)?;
            let base = matched.name("date")?.as_str();
            let base = NaiveDate::parse_from_str(base, "%d/%m/%y").ok()?;
            let start_time = matched.name("start_time")?.as_str();
            let start_time = NaiveTime::parse_from_str(start_time, "%H:%M").ok()?;
            let end_time = matched.name("end_time")?.as_str();
            let end_time = NaiveTime::parse_from_str(end_time, "%H:%M").ok()?;
            let start = tz
                .from_local_datetime(&base.and_time(start_time))
                .earliest()?;
            let end = tz
                .from_local_datetime(&base.and_time(end_time))
                .earliest()?;
            let stripped = remove_matched(date_string, matched.get(0)?);
            (start, Some(end), stripped)
        }
        DateExpressionCases::Tomorrow => {
            let matched = re.find(date_string)?;
            let stripped = remove_matched(date_string, matched);
            let (s, e, stripped) = extract_time(&stripped);
            let (start, end) = calculate_date(reference_date, Duration::days(1), s, e)?;
            (start, end, stripped)
        }
        DateExpressionCases::Today => {
            let matched = re.find(date_string)?;
            let stripped = remove_matched(date_string, matched);
            let (s, e, stripped) = extract_time(&stripped);
            let (start, end) = calculate_date(reference_date, Duration::days(0), s, e)?;
            (start, end, stripped)
        }
        DateExpressionCases::NextWeek => {
            let matched = re.find(date_string)?;
            let stripped = remove_matched(date_string, matched);
            let (s, e, stripped) = extract_time(&stripped);
            let (start, end) = calculate_date(reference_date, Duration::weeks(1), s, e)?;
            (start, end, stripped)
        }
        DateExpressionCases::NextWeekday => {
            let weekday = re
                .captures(date_string)?
                .name("weekday")?
                .as_str()
                .parse::<chrono::Weekday>()
                .ok()?;
            let duration = weekday.days_since(reference_date.weekday());
            let matched = re.find(date_string)?;
            let stripped = remove_matched(date_string, matched);
            let (s, e, stripped) = extract_time(&stripped);
            let (start, end) =
                calculate_date(reference_date, Duration::days(duration as i64), s, e)?;
            (start, end, stripped)
        }
        DateExpressionCases::RelativeTime => {
            let caps = re.captures(date_string)?;
            let (Some(number), Some(unit)) = (caps.name("number"), caps.name("unit")) else {
                return None;
            };
            let num = number.as_str().parse::<u32>().ok()?;
            let duration = parse_relative_time(num, unit.as_str())?;
            let matched = re.find(date_string)?;
            let stripped = remove_matched(date_string, matched);
            let (s, e, stripped) = extract_time(&stripped);
            let (start, end) = calculate_date(reference_date, duration, s, e)?;
            (start, end, stripped)
        }
    };

    Some((start, end, stripped_string))
}

pub fn start_end_to_natural<RefTz: TimeZone, Tz: TimeZone>(
    reference_date: &DateTime<RefTz>,
    start_date: &DateTime<Tz>,
    end_date: &DateTime<Tz>,
) -> String {
    let tz = reference_date.timezone();
    let start_date = start_date.with_timezone(&tz);
    let end_date = end_date.with_timezone(&tz);
    if start_date.day() == end_date.day() {
        format!(
            "at {} {}-{}",
            start_date.naive_local().format("%d/%m/%y"),
            time_to_natural(start_date.time()),
            time_to_natural(end_date.time())
        )
    } else {
        format!(
            "at {} {}-{} {}",
            start_date.naive_local().format("%d/%m/%y"),
            time_to_natural(start_date.time()),
            end_date.naive_local().format("%d/%m/%y"),
            time_to_natural(end_date.time()),
        )
    }
}

pub fn time_to_natural(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use crate::calendar_items::Utc;
    use chrono::{TimeDelta, TimeZone, Timelike};

    use super::*;

    fn create_test_date() -> DateTime<chrono_tz::Tz> {
        // Create a fixed date for testing: 2024-03-15 12:00:00 UTC
        Utc.with_ymd_and_hms(2024, 3, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&chrono_tz::Tz::UTC)
    }

    fn compare_date<Tz1: TimeZone, Tz2: TimeZone>(expected: DateTime<Tz1>, value: DateTime<Tz2>) {
        let e_utc = expected.to_utc();
        let v_utc = value.to_utc();
        assert_eq!(e_utc.year(), v_utc.year());
        assert_eq!(e_utc.month(), v_utc.month());
        assert_eq!(e_utc.day(), v_utc.day());
        assert_eq!(e_utc.hour(), v_utc.hour());
        assert_eq!(e_utc.minute(), v_utc.minute());
        assert_eq!(e_utc.second(), v_utc.second());
    }

    #[test]
    fn test_tomorrow_with_other_tz() {
        let reference = chrono_tz::America::Argentina::Buenos_Aires
            .with_ymd_and_hms(2025, 10, 10, 10, 30, 0)
            .unwrap();
        println!("{reference}");
        let (result, end, stripped) = extract_start_end("tomorrow at 11", reference).unwrap();
        let expected = (reference + Duration::days(1))
            .with_hour(11)
            .unwrap()
            .with_minute(0)
            .unwrap();

        compare_date(expected, result);
        assert!(end.is_none());
        assert_eq!(stripped, "");
    }

    #[test]
    fn test_tomorrow() {
        let reference = create_test_date();
        let (result, end, stripped) = extract_start_end("tomorrow", reference).unwrap();
        let expected = reference + Duration::days(1);
        compare_date(expected, result);
        assert!(end.is_none());
        assert_eq!(stripped, "");
    }

    #[test]
    fn test_tomorrow_with_time() {
        let reference = create_test_date();
        let result = extract_start_end("tomorrow at 19", reference).unwrap().0;
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
    fn test_absolute_dates() {
        let reference = create_test_date();
        let (start, end, stripped) =
            extract_start_end("hello at 03/03/25 14:30-04/03/25 15:30", reference).unwrap();
        let expected = Utc.with_ymd_and_hms(2025, 3, 3, 14, 30, 0).unwrap();
        assert_eq!(stripped, "hello");
        assert_eq!(start.date_naive(), expected.date_naive());
        assert_eq!(start.hour(), expected.hour());
        assert_eq!(start.minute(), expected.minute());

        assert!(end.is_some());
        let end = end.unwrap();
        let expected = Utc.with_ymd_and_hms(2025, 3, 4, 15, 30, 0).unwrap();
        assert_eq!(end.date_naive(), expected.date_naive());
        assert_eq!(end.hour(), expected.hour());
        assert_eq!(end.minute(), expected.minute());
    }

    #[test]
    fn test_absolute_ranges() {
        let reference = create_test_date();
        let (start, end, stripped) =
            extract_start_end("hello at 03/03/25 14:30-15:30", reference).unwrap();
        assert_eq!(stripped, "hello");
        let expected = Utc.with_ymd_and_hms(2025, 3, 3, 14, 30, 0).unwrap();
        assert_eq!(start.date_naive(), expected.date_naive());
        assert_eq!(start.hour(), expected.hour());
        assert_eq!(start.minute(), expected.minute());

        assert!(end.is_some());
        let end = end.unwrap();
        let expected = Utc.with_ymd_and_hms(2025, 3, 3, 15, 30, 0).unwrap();
        assert_eq!(end.date_naive(), expected.date_naive());
        assert_eq!(end.hour(), expected.hour());
        assert_eq!(end.minute(), expected.minute());
    }

    #[test]
    fn test_next_weekday() {
        let reference = create_test_date();
        let result = extract_start_end("next monday", reference).unwrap().0;
        let current_weekday = reference.weekday();
        let target_weekday = chrono::Weekday::Mon;
        let days_ahead = (target_weekday as i32 - current_weekday as i32 + 7) % 7;
        let expected = reference + Duration::days(days_ahead as i64);
        assert_eq!(result.date_naive(), expected.date_naive());
    }
    #[test]

    fn test_next_weekday_at_named_time() {
        let reference = create_test_date();
        let result = extract_start_end("next monday afternoon", reference)
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
        let result = extract_start_end("tomorrow morning", reference).unwrap().0;
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
        let result = extract_start_end("in 3 days", reference).unwrap().0;
        let expected = reference + Duration::days(3);
        assert_eq!(result.date_naive(), expected.date_naive());
    }

    #[test]
    fn test_specific_time() {
        let reference = create_test_date();
        let result = extract_start_end("today at 14:30", reference).unwrap().0;
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
        let (start, end, _) = extract_start_end("today at 14:30-16", reference).unwrap();
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
        let (start, end, _) =
            extract_start_end("steve lepoisson today from 18:45 until 19", reference).unwrap();
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
        let (start, end, _) = extract_start_end("today from 09:30-12:16", reference).unwrap();
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
        let result = extract_start_end("something somethign", reference);
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
            let result = extract_start_end(input, reference).unwrap().0;
            assert_eq!(
                result.date_naive(),
                expected_date.date_naive(),
                "Date mismatch for input: {input}",
            );
            assert_eq!(
                result.hour(),
                expected_hour,
                "Hour mismatch for input: {input}",
            );
            assert_eq!(
                result.minute(),
                expected_minute,
                "Minute mismatch for input: {input}",
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
            let result = extract_start_end(input, reference);
            assert!(result.is_none(), "Should fail for input: {input}");
        }
    }

    #[test]
    fn test_to_natural_same_day() {
        let start_date = create_test_date();
        let end_date = start_date + TimeDelta::hours(1);
        assert_eq!(
            start_end_to_natural(&start_date, &start_date, &end_date),
            "at 15/03/24 12:00-13:00"
        )
    }

    #[test]
    fn test_to_natural_same_day_half_hour() {
        let start_date = create_test_date();
        let end_date = start_date + TimeDelta::hours(1) + TimeDelta::minutes(30);
        assert_eq!(
            start_end_to_natural(&start_date, &start_date, &end_date),
            "at 15/03/24 12:00-13:30"
        )
    }

    #[test]
    fn test_to_natural_next_day() {
        let start_date = create_test_date();
        let end_date = start_date + TimeDelta::hours(30);
        assert_eq!(
            start_end_to_natural(&start_date, &start_date, &end_date),
            "at 15/03/24 12:00-16/03/24 18:00"
        )
    }
}
