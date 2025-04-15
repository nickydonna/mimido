use chrono::DateTime;
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{Output, ToSql},
    sql_types::Text,
    sqlite::{Sqlite, SqliteValue},
};
use icalendar::Property;
use regex::RegexBuilder;

use super::{
    component_props::ComponentProps,
    input_traits::{ExtractableFromInput, ToInput},
};

#[derive(
    Debug,
    PartialEq,
    Clone,
    Copy,
    serde::Serialize,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
    strum_macros::Display,
    FromSqlRow,
    AsExpression,
    specta::Type,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[strum(serialize_all = "lowercase")]
pub enum EventType {
    Event,
    Block,
    Reminder,
    Task,
}

impl FromSql<Text, Sqlite> for EventType {
    fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(t.as_str().try_into()?)
    }
}

impl ToSql<Text, Sqlite> for EventType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
        out.set_value(self.to_string());
        Ok(diesel::serialize::IsNull::No)
    }
}

impl From<EventType> for icalendar::Property {
    fn from(value: EventType) -> Self {
        Property::new(ComponentProps::Type.as_ref(), value.as_ref())
    }
}

const EVENT_TYPE_RE: &str = r"@(?P<event_type>event|block|reminder|task)";

impl ExtractableFromInput for EventType {
    fn extract_from_input(
        _: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<(Self, String), String> {
        let re = RegexBuilder::new(EVENT_TYPE_RE)
            .case_insensitive(true)
            .build()
            .map_err(|e| e.to_string())?;

        let captured = re.captures(input);
        let Some(captured) = captured else {
            return Ok((EventType::Event, input.to_string()));
        };

        let general = captured.get(0).expect("Already check if it's some");
        let captured = captured
            .name("event_type")
            .map(|e| e.as_str())
            .expect("Already check if it's some");

        let event_type = match captured.to_lowercase().as_str() {
            "event" => EventType::Event,
            "block" => EventType::Block,
            "reminder" => EventType::Reminder,
            "task" => EventType::Task,
            _ => Err(format!("Invalid event type: {}", input))?,
        };

        Ok((
            event_type,
            format!("{} {}", &input[0..general.start()], &input[general.end()..])
                .trim()
                .to_string(),
        ))
    }
}

impl ToInput for EventType {
    fn to_input(&self, _: DateTime<chrono_tz::Tz>) -> String {
        format!("@{}", self)
    }
}
