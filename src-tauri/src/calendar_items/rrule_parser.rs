use chrono::{DateTime, Utc};
use regex::{Regex, RegexBuilder};
use rrule::{Frequency, NWeekday, RRule, RRuleSet, Weekday};
use strum::IntoEnumIterator;

#[derive(Copy, Clone, Debug, strum_macros::EnumIter)]
enum NaturalLangCases {
    EveryXDays,
    MonthOnXDays,
    EveryWeekday,
    EveryWeekend,
    EveryDay,
    WeekOnXDays,
    EveryXWeeksOnXDays,
}

const DAYS_PATTERN: &str = r"(?P<day>Mon|Tue|Wed|Thu|Fri|Sat|Sun|Monday|Tuesday|Wednesday|Thurdsay|Friday|Saturday|Sunday)";

impl From<NaturalLangCases> for Regex {
    fn from(value: NaturalLangCases) -> Self {
        let re_str = match value {
            NaturalLangCases::EveryXDays => (r"every (?P<interval>[0-9]{1,3}) days").to_string(),
            NaturalLangCases::EveryWeekday => r"every weekday".to_string(),
            NaturalLangCases::EveryWeekend => r"every weekend".to_string(),
            NaturalLangCases::EveryDay => r"every day".to_string(),
            NaturalLangCases::MonthOnXDays => format!(r"every month on (:?{DAYS_PATTERN},? ?)+"),
            NaturalLangCases::WeekOnXDays => format!(r"every (:?{DAYS_PATTERN},? ?)+"),
            NaturalLangCases::EveryXWeeksOnXDays => {
                format!("every (?P<interval>[0-9]{{1,3}}) on (:?{DAYS_PATTERN},? ?)+")
            }
        };
        RegexBuilder::new(&re_str)
            .case_insensitive(true)
            .build()
            .expect("Re to compile")
    }
}

impl TryFrom<&RRuleSet> for NaturalLangCases {
    type Error = String;

    fn try_from(parsed_rule: &RRuleSet) -> Result<Self, Self::Error> {
        let parsed_rule = parsed_rule.get_rrule().first().expect("To have rrule");

        // Frequency description
        let frequency = parsed_rule.get_freq();
        let interval = parsed_rule.get_interval();
        let days = parsed_rule.get_by_weekday();
        if frequency == Frequency::Daily && interval > 1 {
            return Ok(NaturalLangCases::EveryXDays);
        }
        if frequency == Frequency::Weekly {
            if days.len() == 5
                && RRuleParser::has_weekday(days, Weekday::Mon)
                && RRuleParser::has_weekday(days, Weekday::Tue)
                && RRuleParser::has_weekday(days, Weekday::Wed)
                && RRuleParser::has_weekday(days, Weekday::Thu)
                && RRuleParser::has_weekday(days, Weekday::Fri)
            {
                return Ok(NaturalLangCases::EveryWeekday);
            } else if days.len() == 2
                && RRuleParser::has_weekday(days, Weekday::Sat)
                && RRuleParser::has_weekday(days, Weekday::Sun)
            {
                return Ok(NaturalLangCases::EveryWeekend);
            } else if days.len() == 7 {
                return Ok(NaturalLangCases::EveryDay);
            } else if days.is_empty() {
                // TODO: add case
                return Err("Missing case for weekly freq and empty days".to_string());
            }

            if interval > 1 {
                return Ok(NaturalLangCases::EveryXWeeksOnXDays);
            }
            return Ok(NaturalLangCases::WeekOnXDays);
        }

        if frequency == Frequency::Monthly {
            return Ok(NaturalLangCases::MonthOnXDays);
        }
        Err("Case not handled".to_string())
    }
}

pub struct RRuleParser;

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

    fn get_interval(natural_string: &str, re: &Regex) -> Option<u16> {
        re.captures_iter(natural_string)
            .filter_map(|m| m.name("interval"))
            .filter_map(|m| m.as_str().parse::<u16>().ok())
            .collect::<Vec<u16>>()
            .first()
            .copied()
    }

    fn from_natural(natural_string: &str, dt_start: DateTime<Utc>) -> Option<RRuleSet> {
        let (case, re) = NaturalLangCases::iter()
            .map(|case| (case, Regex::from(case)))
            .find(|(_, re)| re.is_match(natural_string))?;

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

        let (freq, days, interval) = match case {
            NaturalLangCases::EveryWeekday => (Frequency::Weekly, Some(weekday), None),
            NaturalLangCases::EveryWeekend => (Frequency::Weekly, Some(weekend), None),
            NaturalLangCases::EveryDay => {
                (Frequency::Weekly, Some([weekday, weekend].concat()), None)
            }
            NaturalLangCases::WeekOnXDays => {
                let days = RRuleParser::parse_weekdays(natural_string)?;
                (Frequency::Weekly, Some(days), None)
            }
            NaturalLangCases::MonthOnXDays => {
                let days = RRuleParser::parse_weekdays(natural_string)?;
                (Frequency::Monthly, Some(days), None)
            }
            NaturalLangCases::EveryXWeeksOnXDays => {
                let days = RRuleParser::parse_weekdays(natural_string)?;
                let interval = RRuleParser::get_interval(natural_string, &re)?;
                (Frequency::Weekly, Some(days), Some(interval))
            }
            NaturalLangCases::EveryXDays => {
                let interval = RRuleParser::get_interval(natural_string, &re)?;
                (Frequency::Daily, None, Some(interval))
            }
        };

        let mut rrule = RRule::new(freq).interval(interval.unwrap_or(1));
        if let Some(days) = days {
            rrule = rrule.by_weekday(days);
        }
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
    pub fn to_natural_language(ruleset: &RRuleSet) -> Result<String, String> {
        let parsed_rule = ruleset.get_rrule().first().expect("To have rrule");

        // Frequency description
        let days = parsed_rule.get_by_weekday();
        let interval = parsed_rule.get_interval();

        let case = NaturalLangCases::try_from(ruleset)?;
        let days_string = days
            .iter()
            .filter_map(|nweekday| match nweekday {
                NWeekday::Every(weekday) => Some(weekday),
                NWeekday::Nth(_, _) => None,
            })
            .map(RRuleParser::stringify_day)
            .collect::<Vec<String>>()
            .join(" , ");

        match case {
            NaturalLangCases::EveryXDays => Ok(format!("every {} days", interval)),
            NaturalLangCases::MonthOnXDays => Ok(format!("every month on {}", days_string)),
            NaturalLangCases::EveryWeekday => Ok("every weekday".to_string()),
            NaturalLangCases::EveryWeekend => Ok("every weekend".to_string()),
            NaturalLangCases::EveryDay => Ok("every day".to_string()),
            NaturalLangCases::WeekOnXDays => Ok(format!("every {}", days_string)),
            NaturalLangCases::EveryXWeeksOnXDays => {
                Ok(format!("every {} weeks on {}", interval, days_string))
            }
        }
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
    fn test_parse_every_x_days() {
        let rrule = "every 2 days";
        let date = Utc::now();
        let parsed = RRuleParser::from_natural(rrule, date).expect("Should parse successfully");

        let rrule = parsed
            .get_rrule()
            .first()
            .expect("To have at least one RRule");
        assert_eq!(rrule.get_freq(), Frequency::Daily);
        assert_eq!(rrule.get_interval(), 2);
        assert_eq!(rrule.get_by_weekday().len(), 0);
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
        let rrule = "every tue, wed";
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
    fn test_full_strings() {
        let date = Utc::now();
        // Test cases for round-trip conversion
        let valid_test_rrules = vec![
            "Call mom every Mon",
            "Every weekday go to the forest",
            "bath Every weekend",
            "throw to the fire Every month on Tue,Friday",
            "kill capitalism Every Mon,Fri,Wed",
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
        assert!(description.is_ok());
        assert_eq!(description.unwrap(), "every Monday");
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
        assert!(description.is_ok());
        assert_eq!(description.unwrap(), "every weekday");
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
        assert!(description.is_ok());
        assert_eq!(description.unwrap(), "every weekend");
    }

    #[test]
    fn test_round_trip_conversion() {
        let date = Utc::now();
        // Test cases for round-trip conversion
        let test_rrules = vec![
            "Every 2 days",
            // "Every Mon",
            // "Every weekday",
            // "Every weekend",
            // "Every month on Tue,Friday",
            // "Every Mon,Fri,Wed",
        ];

        for rrule in test_rrules {
            let parsed = RRuleParser::from_natural(rrule, date).expect("Should parse successfully");
            let natural_language =
                RRuleParser::to_natural_language(&parsed).expect("To be natural lang");

            // Ensure we can parse the result back
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
