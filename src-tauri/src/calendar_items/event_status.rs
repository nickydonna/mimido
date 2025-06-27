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
    strum_macros::EnumString,
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
    Doing,
    Done,
}

impl FromSql<Text, Sqlite> for EventStatus {
    fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(t.as_str().try_into()?)
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

const EVENT_STATUS_RE: &str = r"%(?P<event_status>backlog|todo|doing|done)";

impl ExtractableFromInput for EventStatus {
    fn extract_from_input(
        _: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<impl Into<ExtractedInput<Self>>, String> {
        let re = RegexBuilder::new(EVENT_STATUS_RE)
            .case_insensitive(true)
            .build()
            .map_err(|e| e.to_string())?;

        let captured = re.captures(input);

        let Some(captured) = captured else {
            return Ok((EventStatus::Todo, input.to_string()));
        };

        let general = captured.get(0).expect("Already check if it's some");
        let captured = captured
            .name("event_status")
            .map(|e| e.as_str())
            .expect("Already check if it's some");

        let status = match captured.to_lowercase().as_str() {
            "backlog" => EventStatus::Backlog,
            "todo" => EventStatus::Todo,
            "doing" => EventStatus::Doing,
            "done" => EventStatus::Done,
            _ => Err(format!("Invalid event status: {}", captured))?,
        };
        Ok((
            status,
            format!("{} {}", &input[0..general.start()], &input[general.end()..])
                .trim()
                .to_string(),
        ))
    }
}

impl ToInput for EventStatus {
    fn to_input(&self, _: DateTime<chrono_tz::Tz>) -> String {
        format!("%{}", self)
    }
}
