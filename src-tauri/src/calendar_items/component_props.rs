use std::str::FromStr;

use icalendar::Component;

#[derive(Debug, PartialEq, strum_macros::AsRefStr)]
pub enum ComponentProps {
    #[strum(serialize = "X-TYPE")]
    Type,
    #[strum(serialize = "X-TAG")]
    Tag,
    #[strum(serialize = "X-URGENCY")]
    Urgency,
    #[strum(serialize = "X-STATUS")]
    Status,
    #[strum(serialize = "X-ORIGINAL-TEXT")]
    OriginalText,
    #[strum(serialize = "X-IMPORTANCE")]
    Importance,
    #[strum(serialize = "X-LOAD")]
    Load,
    #[strum(serialize = "X-POSTPONED")]
    Postponed,
    #[strum(serialize = "DTSTART")]
    DtStart,
    #[strum(serialize = "RRULE")]
    RRule,
    #[strum(serialize = "RDATE")]
    RDate,
    #[strum(serialize = "EXDATE")]
    Exdate,
}

pub fn get_property_or_default<T: FromStr>(
    event: &icalendar::Event,
    property: ComponentProps,
    default: T,
) -> T {
    let raw_type = event.property_value(property.as_ref());
    let Some(raw_type) = raw_type else {
        return default;
    };
    T::from_str(raw_type).ok().unwrap_or(default)
}

pub fn get_string_property(event: &icalendar::Event, property: ComponentProps) -> Option<String> {
    event
        .property_value(property.as_ref())
        .map(|e| e.to_string())
}

pub fn get_int_property(event: &icalendar::Event, property: ComponentProps) -> i32 {
    event
        .property_value(property.as_ref())
        .map(|e| e.to_string())
        .and_then(|e| e.parse::<i32>().ok())
        .unwrap_or(0)
}
