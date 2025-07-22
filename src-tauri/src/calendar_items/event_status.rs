use std::str::FromStr;

use anyhow::anyhow;
use chrono::{DateTime, TimeZone};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{Output, ToSql},
    sql_types::Text,
    sqlite::{Sqlite, SqliteValue},
};
use icalendar::Property;
use regex::RegexBuilder;

use crate::calendar_items::input_traits::ExtractedInput;

use super::{
    component_props::ComponentProps,
    input_traits::{ExtractableFromInput, ToInput},
};

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    serde::Serialize,
    strum_macros::AsRefStr,
    strum_macros::Display,
    FromSqlRow,
    AsExpression,
    specta::Type,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[strum(serialize_all = "lowercase")]
pub enum EventStatus {
    #[strum(serialize = "back")]
    Backlog,
    Todo,
    InProgress,
    Done,
}

impl FromSql<Text, Sqlite> for EventStatus {
    fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(t.as_str().parse()?)
    }
}

impl ToSql<Text, Sqlite> for EventStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
        out.set_value(self.to_string());
        Ok(diesel::serialize::IsNull::No)
    }
}

impl From<EventStatus> for icalendar::Property {
    fn from(value: EventStatus) -> Self {
        Property::new(ComponentProps::Status.as_ref(), value.as_ref())
    }
}

impl FromStr for EventStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "backlog" | "back" | "b" => Ok(EventStatus::Backlog),
            "todo" | "t" => Ok(EventStatus::Todo),
            "doing" | "inprogress" | "i" => Ok(EventStatus::InProgress),
            "done" | "d" => Ok(EventStatus::Done),
            _ => Err(anyhow!("Invalid event status: {value}")),
        }
    }
}

const EVENT_STATUS_RE: &str = r"%(?P<event_status>backlog|todo|inprogress|doing|done|b|t|i|d)";

impl ExtractableFromInput for EventStatus {
    fn extract_from_input<Tz: TimeZone>(
        _: DateTime<Tz>,
        input: &str,
    ) -> anyhow::Result<impl Into<ExtractedInput<Self>>> {
        let re = RegexBuilder::new(EVENT_STATUS_RE)
            .case_insensitive(true)
            .build()?;

        let captured = re.captures(input);

        let Some(captured) = captured else {
            return Ok((EventStatus::Todo, input.to_string()));
        };

        let general = captured.get(0).expect("Already check if it's some");
        let captured = captured
            .name("event_status")
            .map(|e| e.as_str())
            .expect("Already check if it's some");

        let status: EventStatus = captured.parse()?;

        Ok((
            status,
            format!("{} {}", &input[0..general.start()], &input[general.end()..])
                .trim()
                .to_string(),
        ))
    }
}

impl From<EventStatus> for String {
    fn from(value: EventStatus) -> Self {
        format!("{value}")
    }
}

impl ToInput for EventStatus {
    fn to_input(&self, _: DateTime<chrono_tz::Tz>) -> String {
        format!("%{self}")
    }
}
