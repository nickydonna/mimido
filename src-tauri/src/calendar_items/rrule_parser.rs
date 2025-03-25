use chrono::{DateTime, Utc};
use regex::RegexBuilder;
use rrule::{Frequency, NWeekday, RRule, RRuleSet, Unvalidated, Weekday};

struct RRuleParser;

impl RRuleParser {
    fn parse_weekdays(natural_string: &str) -> Option<Vec<NWeekday>> {
        let re = RegexBuilder::new(r"(?P<day>Mon|Tue|Wed|Thu|Fri|Sat|Sun|Monday|Tuesday|Wednesday|Thurdsay|Friday|Saturday|Sunday)")
            .case_insensitive(true)
            .build()
            .ok()?;
        let by_weekday = re
            .captures_iter(natural_string)
            .filter_map(|m| m.name("day"))
            .map(|day_match| day_match.as_str().to_string())
            .filter_map(|day_string| day_string.parse::<Weekday>().ok())
            .map(|weekday| NWeekday::new(None, weekday))
            .collect::<Vec<NWeekday>>();

        if by_weekday.is_empty() {
            None
        } else {
            Some(by_weekday)
        }
    }

    fn parse_frequency(natural_string: &str) -> Option<RRule<Unvalidated>> {
        let weekday = vec![
            NWeekday::new(None, Weekday::Mon),
            NWeekday::new(None, Weekday::Tue),
            NWeekday::new(None, Weekday::Wed),
            NWeekday::new(None, Weekday::Thu),
            NWeekday::new(None, Weekday::Fri),
        ];
        let weekend = vec![
            NWeekday::new(None, Weekday::Sat),
            NWeekday::new(None, Weekday::Sun),
        ];
        let re_weekday = RegexBuilder::new(r"every weekday")
            .case_insensitive(true)
            .build()
            .ok()?;
        if re_weekday.is_match(natural_string) {
            let rrule = RRule::new(Frequency::Weekly);
            let rrule = rrule.by_weekday(weekday);
            return Some(rrule);
        }
        let re_weekend = RegexBuilder::new(r"every weekend")
            .case_insensitive(true)
            .build()
            .ok()?;
        if re_weekend.is_match(natural_string) {
            let rrule = RRule::new(Frequency::Weekly);
            let rrule = rrule.by_weekday(weekend);
            return Some(rrule);
        }
        let re_day = RegexBuilder::new(r"every day")
            .case_insensitive(true)
            .build()
            .ok()?;
        if re_day.is_match(natural_string) {
            let rrule = RRule::new(Frequency::Daily);
            return Some(rrule);
        }
        let weekday_patterns = vec![
            (r"every week", Frequency::Weekly),
            (r"every month", Frequency::Monthly),
        ];
        let mut frequency = None;
        // Check weekday patterns first
        for (pattern, freq) in weekday_patterns {
            let re = RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
                .ok()?;
            if re.is_match(natural_string) {
                frequency = Some(freq);
            }
        }
        let rrule = RRule::new(frequency.unwrap_or(Frequency::Weekly));

        let by_weekday = RRuleParser::parse_weekdays(natural_string);
        by_weekday.map(|by_weekday| rrule.by_weekday(by_weekday))
    }

    /// Parse RRULE string into structured components
    fn from_natural(natural_string: &str, dt_start: DateTime<Utc>) -> Option<RRuleSet> {
        let starting_index = natural_string.to_lowercase().find("every")?;
        let natural_string = natural_string.get(starting_index..)?;

        let rrule = RRuleParser::parse_frequency(natural_string)?;
        rrule.build(dt_start.with_timezone(&rrule::Tz::UTC)).ok()
    }

    fn stringify_day(day: &Weekday) -> String {
        match day {
            Weekday::Mon => "Monday",
            Weekday::Tue => "Tuesday",
            Weekday::Wed => "Wednesday",
            Weekday::Thu => "Thurdsay",
            Weekday::Fri => "Friday",
            Weekday::Sat => "Saturday",
            Weekday::Sun => "Sunday",
        }
        .to_string()
    }

    fn is_every_day(nweekday: &NWeekday, compare_weekday: Weekday) -> bool {
        match nweekday {
            NWeekday::Every(weekday) => weekday == &compare_weekday,
            NWeekday::Nth(_, _) => false,
        }
    }

    fn has_weekday(days: &[NWeekday], compare_weekday: Weekday) -> bool {
        days.iter()
            .any(|d| RRuleParser::is_every_day(d, compare_weekday))
    }

    /// Convert parsed RRULE back to natural language
    fn to_natural_language(parsed_rule: &RRuleSet) -> String {
        let parsed_rule = parsed_rule.get_rrule().first().expect("To have rrule");

        // Frequency description
        let frequency = parsed_rule.get_freq();
        let days = parsed_rule.get_by_weekday();

        if frequency == Frequency::Weekly {
            if days.len() == 5
                && RRuleParser::has_weekday(days, Weekday::Mon)
                && RRuleParser::has_weekday(days, Weekday::Tue)
                && RRuleParser::has_weekday(days, Weekday::Wed)
                && RRuleParser::has_weekday(days, Weekday::Thu)
                && RRuleParser::has_weekday(days, Weekday::Fri)
            {
                return "Every weekday".to_string();
            } else if days.len() == 2
                && RRuleParser::has_weekday(days, Weekday::Sat)
                && RRuleParser::has_weekday(days, Weekday::Sun)
            {
                return "Every weekend".to_string();
            } else if !days.is_empty() {
            }
        }
        let freq_desc = match frequency {
            Frequency::Daily => "day",
            Frequency::Weekly => "week",
            Frequency::Monthly => "month",
            Frequency::Yearly => "year",
            Frequency::Hourly => "hour",
            Frequency::Minutely => "minute",
            Frequency::Secondly => "second",
        };
        format!(
            "Every {} on {} ",
            freq_desc,
            days.iter()
                .filter_map(|nweekday| match nweekday {
                    NWeekday::Every(weekday) => Some(weekday),
                    NWeekday::Nth(_, _) => None,
                })
                .map(RRuleParser::stringify_day)
                .collect::<Vec<String>>()
                .join(" , ")
        )
        .trim()
        .to_string()
    }
}

// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_recur_is_none() {
        let rrule = "hello";
        let date = Utc::now();
        let parsed = RRuleParser::from_natural(rrule, date);
        assert_eq!(parsed, None)
    }

    #[test]
    fn test_invalid_sequence_is_none() {
        let rrule = "every";
        let date = Utc::now();
        let parsed = RRuleParser::from_natural(rrule, date);
        assert_eq!(parsed, None)
    }

    #[test]
    fn test_alone_day_is_none() {
        let rrule = "Meet on monday";
        let date = Utc::now();
        let parsed = RRuleParser::from_natural(rrule, date);
        assert_eq!(parsed, None)
    }

    #[test]
    fn test_parse_basic_weekly_rrule() {
        let rrule = "every monday";
        let date = Utc::now();
        let parsed = RRuleParser::from_natural(rrule, date).expect("Should parse successfully");

        let rrule = parsed
            .get_rrule()
            .first()
            .expect("To have at least one RRule");
        assert_eq!(rrule.get_freq(), Frequency::Weekly);
        assert_eq!(rrule.get_interval(), 1);
        assert_eq!(
            rrule.get_by_weekday(),
            vec![NWeekday::new(None, Weekday::Mon)]
        );
        assert_eq!(rrule.get_count(), None);
    }

    #[test]
    fn test_parse_weekday_rrule() {
        let rrule = "every weekday";
        let date = Utc::now();

        let parsed = RRuleParser::from_natural(rrule, date).expect("Should parse successfully");
        let rrule = parsed
            .get_rrule()
            .first()
            .expect("To have at least one RRule");

        assert_eq!(rrule.get_freq(), Frequency::Weekly);
        assert_eq!(
            rrule.get_by_weekday(),
            vec![
                NWeekday::new(None, Weekday::Mon),
                NWeekday::new(None, Weekday::Tue),
                NWeekday::new(None, Weekday::Wed),
                NWeekday::new(None, Weekday::Thu),
                NWeekday::new(None, Weekday::Fri),
            ]
        );
    }

    #[test]
    fn test_parse_weekend_rrule() {
        let rrule = "every weekend";
        let date = Utc::now();

        let parsed = RRuleParser::from_natural(rrule, date).expect("Should parse successfully");
        let rrule = parsed
            .get_rrule()
            .first()
            .expect("To have at least one RRule");

        assert_eq!(rrule.get_freq(), Frequency::Weekly);
        assert_eq!(
            rrule.get_by_weekday(),
            vec![
                NWeekday::new(None, Weekday::Sat),
                NWeekday::new(None, Weekday::Sun),
            ]
        );
    }

    #[test]
    fn test_parse_month_rrule() {
        let rrule = "every month on monday";
        let date = Utc::now();
        let parsed = RRuleParser::from_natural(rrule, date).expect("Should parse successfully");

        let rrule = parsed
            .get_rrule()
            .first()
            .expect("To have at least one RRule");

        assert_eq!(rrule.get_freq(), Frequency::Monthly);
        assert_eq!(
            rrule.get_by_weekday(),
            vec![NWeekday::new(None, Weekday::Mon),]
        );
    }

    #[test]
    fn test_parse_week_rrule() {
        let rrule = "every week on tue, wed";
        let date = Utc::now();
        let parsed = RRuleParser::from_natural(rrule, date).expect("Should parse successfully");

        let rrule = parsed
            .get_rrule()
            .first()
            .expect("To have at least one RRule");

        assert_eq!(rrule.get_freq(), Frequency::Weekly);
        assert_eq!(
            rrule.get_by_weekday(),
            vec![
                NWeekday::new(None, Weekday::Tue),
                NWeekday::new(None, Weekday::Wed),
            ]
        );
    }

    #[test]
    fn full_strings() {
        let date = Utc::now();
        // Test cases for round-trip conversion
        let valid_test_rrules = vec![
            "Call mom every Mon",
            "Every weekday go to the forest",
            "bath Every weekend",
            "throw to the fire Every month on Tue,Friday",
            "kill capitalism Every week on Mon,Fri,Wed",
        ];

        for rrule in valid_test_rrules {
            let parsed = RRuleParser::from_natural(rrule, date);
            assert!(parsed.is_some())
        }
    }

    #[test]
    fn test_to_natural_language_basic() {
        let date = Utc::now();
        let rule = RRule::new(Frequency::Weekly)
            .interval(1)
            .by_weekday(vec![NWeekday::new(None, Weekday::Mon)])
            .build(date.with_timezone(&rrule::Tz::UTC))
            .expect("To buid rruleset");

        let description = RRuleParser::to_natural_language(&rule);
        assert_eq!(description, "Every week on Monday");
    }

    #[test]
    fn test_to_natural_language_weekday() {
        let date = Utc::now();
        let rule = RRule::new(Frequency::Weekly)
            .interval(1)
            .by_weekday(vec![
                NWeekday::new(None, Weekday::Mon),
                NWeekday::new(None, Weekday::Tue),
                NWeekday::new(None, Weekday::Wed),
                NWeekday::new(None, Weekday::Thu),
                NWeekday::new(None, Weekday::Fri),
            ])
            .build(date.with_timezone(&rrule::Tz::UTC))
            .expect("To buid rruleset");

        let description = RRuleParser::to_natural_language(&rule);
        assert_eq!(description, "Every weekday");
    }

    #[test]
    fn test_to_natural_language_weekend() {
        let date = Utc::now();

        let rule = RRule::new(Frequency::Weekly)
            .interval(1)
            .by_weekday(vec![
                NWeekday::new(None, Weekday::Sat),
                NWeekday::new(None, Weekday::Sun),
            ])
            .build(date.with_timezone(&rrule::Tz::UTC))
            .expect("To buid rruleset");

        let description = RRuleParser::to_natural_language(&rule);
        assert_eq!(description, "Every weekend");
    }

    #[test]
    fn test_round_trip_conversion() {
        let date = Utc::now();
        // Test cases for round-trip conversion
        let test_rrules = vec![
            "Every Mon",
            "Every weekday",
            "Every weekend",
            "Every month on Tue,Friday",
            "Every week on Mon,Fri,Wed",
        ];

        for rrule in test_rrules {
            let parsed = RRuleParser::from_natural(rrule, date).expect("Should parse successfully");
            let natural_language = RRuleParser::to_natural_language(&parsed);

            // // Ensure we can parse the result back
            let reparsed = RRuleParser::from_natural(&natural_language, date)
                .expect("Should be able to parse natural language back to RRULE");

            assert_eq!(
                parsed, reparsed,
                "Round-trip conversion failed for {}",
                rrule
            );
        }
    }
}
