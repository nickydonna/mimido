use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;
use regex::RegexBuilder;

use crate::calendar_items::input_traits::ExtractedInput;

use super::input_traits::{ExtractableFromInput, ToInput};

/// We can't use this as a Sql type because String doesn't work properly
#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct EventTags(pub Option<String>);

const EVENT_TYPE_RE: &str = r"#(?P<event_tag>\w+)";

impl<Tz: TimeZone> ExtractableFromInput<Tz> for EventTags {
    fn extract_from_input(
        _: DateTime<Tz>,
        input: &str,
    ) -> anyhow::Result<impl Into<ExtractedInput<Self>>> {
        let re = RegexBuilder::new(EVENT_TYPE_RE)
            .case_insensitive(true)
            .build()?;

        if !re.is_match(input) {
            return Ok((EventTags(None), input.to_string()));
        }
        let mut tags: Vec<String> = Vec::with_capacity(1);
        let mut haystack = input.to_string();

        while let Some(captured) = re.captures(&haystack) {
            let whole_cap = captured.get(0).expect("Already check if it's some");
            let tag = captured
                .name("event_tag")
                .map(|e| e.as_str())
                .expect("Already check if it's some")
                .trim();

            tags.push(tag.to_string());
            haystack = format!(
                "{} {}",
                &haystack[0..whole_cap.start()],
                &haystack[whole_cap.end()..]
            )
            .trim()
            .to_string();
        }
        Ok((EventTags(Some(tags.join(","))), haystack))
    }
}

impl From<EventTags> for String {
    fn from(value: EventTags) -> Self {
        value.to_input(&Utc::now())
    }
}

impl<Tz: TimeZone> ToInput<Tz> for EventTags {
    fn to_input(&self, _: &DateTime<Tz>) -> String {
        match &self.0 {
            Some(s) => s.split(",").map(|t| format!("#{t}")).join(" "),
            None => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn reference_date() -> DateTime<chrono_tz::Tz> {
        chrono_tz::America::Argentina::Buenos_Aires
            .with_ymd_and_hms(2025, 10, 10, 10, 30, 0)
            .unwrap()
    }

    #[test]
    fn test_tags() {
        let ExtractedInput(tags, stripped) =
            EventTags::extract_from_input(reference_date(), "tomorrow at 11 #hello")
                .unwrap()
                .into();
        assert_eq!(tags.0, Some("hello".to_string()));
        assert_eq!(stripped, "tomorrow at 11");
    }

    #[test]
    fn test_tags_begin_and_end() {
        let ExtractedInput(tags, stripped) =
            EventTags::extract_from_input(reference_date(), "#another tomorrow at 11 #hello")
                .unwrap()
                .into();
        assert_eq!(tags.0, Some("another,hello".to_string()));
        assert_eq!(stripped, "tomorrow at 11");
    }
}
