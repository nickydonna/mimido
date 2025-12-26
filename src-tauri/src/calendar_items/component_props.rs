use anyhow::anyhow;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use icalendar::{Component, TodoStatus};

use super::{event_status::EventStatus, event_type::EventType};

#[derive(Debug, PartialEq, strum_macros::AsRefStr)]
pub enum ComponentProps {
    #[strum(serialize = "X-TYPE")]
    Type,
    #[strum(serialize = "X-TAG")]
    Tag,
    #[strum(serialize = "CATEGORIES")]
    Categories,
    #[strum(serialize = "X-URGENCY")]
    Urgency,
    #[strum(serialize = "X-STATUS")]
    XStatus,
    #[strum(serialize = "X-ORIGINAL-TEXT")]
    OriginalText,
    #[strum(serialize = "X-IMPORTANCE")]
    Importance,
    #[strum(serialize = "X-LOAD")]
    Load,
    #[strum(serialize = "X-POSTPONED")]
    Postponed,
    #[strum(serialize = "RRULE")]
    RRule,
    #[strum(serialize = "RDATE")]
    RDate,
    #[strum(serialize = "EXDATE")]
    Exdate,
    #[strum(serialize = "DURATION")]
    Duration,
}

impl From<ComponentProps> for String {
    fn from(value: ComponentProps) -> Self {
        value.as_ref().to_string()
    }
}

pub fn get_property<Cmp: icalendar::Component, T: FromStr>(
    event: &Cmp,
    property: ComponentProps,
) -> Option<T> {
    event
        .property_value(property.as_ref())
        .and_then(|s| T::from_str(s).ok())
}

pub fn get_property_or_default<Cmp: icalendar::Component, T: FromStr>(
    event: &Cmp,
    property: ComponentProps,
    default: T,
) -> T {
    let raw_type = event.property_value(property.as_ref());
    let Some(raw_type) = raw_type else {
        return default;
    };
    T::from_str(raw_type).ok().unwrap_or(default)
}

pub fn get_string_property<Cmp: icalendar::Component>(
    event: &Cmp,
    property: ComponentProps,
) -> Option<String> {
    event
        .property_value(property.as_ref())
        .map(|e| e.to_string())
}

pub fn get_int_property<Cmp: icalendar::Component>(event: &Cmp, property: ComponentProps) -> i32 {
    event
        .property_value(property.as_ref())
        .map(|e| e.to_string())
        .and_then(|e| e.parse::<i32>().ok())
        .unwrap_or(0)
}

/// General properties from a Calendar Component
pub struct GeneralComponentProps {
    pub uid: String,
    pub summary: String,
    pub description: Option<String>,
    pub event_type: EventType,
    pub tag: Option<String>,
    pub urgency: i32,
    pub status: EventStatus,
    pub original_text: Option<String>,
    pub importance: i32,
    pub load: i32,
    pub postponed: i32,
    pub last_modified: DateTime<Utc>,
}

impl TryFrom<&icalendar::Event> for GeneralComponentProps {
    type Error = anyhow::Error;
    fn try_from(first_event: &icalendar::Event) -> Result<Self, Self::Error> {
        let uid = first_event
            .get_uid()
            .ok_or_else(|| anyhow!("Missing UID"))?
            .to_string();
        let summary = first_event
            .get_summary()
            .unwrap_or("[No Summary]")
            .to_string();
        let description = first_event.get_description().map(|d| d.to_string());
        let last_modified = first_event.get_last_modified().unwrap_or(Utc::now());
        let event_type =
            get_property_or_default(first_event, ComponentProps::Type, EventType::Event);
        let tag = get_string_property(first_event, ComponentProps::Tag);
        let status =
            get_property_or_default(first_event, ComponentProps::XStatus, EventStatus::Todo);
        let original_text = get_string_property(first_event, ComponentProps::OriginalText);
        let importance = get_int_property(first_event, ComponentProps::Importance);
        let urgency = get_int_property(first_event, ComponentProps::Urgency);
        let load = get_int_property(first_event, ComponentProps::Load);
        let postponed = get_int_property(first_event, ComponentProps::Postponed);

        Ok(GeneralComponentProps {
            uid,
            summary,
            description,
            event_type,
            tag,
            urgency,
            status,
            original_text,
            importance,
            load,
            postponed,
            last_modified,
        })
    }
}

impl TryFrom<&icalendar::Todo> for GeneralComponentProps {
    type Error = anyhow::Error;
    fn try_from(first_todo: &icalendar::Todo) -> Result<Self, Self::Error> {
        let uid = first_todo
            .get_uid()
            .ok_or(anyhow!("Missing UID"))?
            .to_string();
        let summary = first_todo
            .get_summary()
            .unwrap_or("[No Summary]")
            .to_string();
        let description = first_todo.get_description().map(|d| d.to_string());
        let last_modified = first_todo.get_last_modified().unwrap_or(Utc::now());
        let event_type = get_property_or_default(first_todo, ComponentProps::Type, EventType::Task);
        let tag = get_string_property(first_todo, ComponentProps::Tag);
        let todo_status_completed = first_todo
            .get_status()
            .map(|s| matches!(s, TodoStatus::Completed))
            .unwrap_or(false);
        // If XStatus is not set try to use the status from the todo
        let status = get_property(first_todo, ComponentProps::XStatus).unwrap_or({
            if todo_status_completed {
                EventStatus::Done
            } else {
                EventStatus::Todo
            }
        });

        let original_text = get_string_property(first_todo, ComponentProps::OriginalText);
        let importance = get_int_property(first_todo, ComponentProps::Importance);
        let urgency = get_int_property(first_todo, ComponentProps::Urgency);
        let load = get_int_property(first_todo, ComponentProps::Load);
        let postponed = get_int_property(first_todo, ComponentProps::Postponed);

        Ok(GeneralComponentProps {
            uid,
            summary,
            description,
            event_type,
            tag,
            urgency,
            status,
            original_text,
            importance,
            load,
            postponed,
            last_modified,
        })
    }
}
