use chrono::{DateTime, TimeZone};

pub trait ToInput {
    fn to_input<Tz: TimeZone>(&self, referece_date: &DateTime<Tz>) -> String;
}

/// Tuple with the result, and the input string
pub struct ExtractedInput<T>(pub T, pub String);

impl<T> From<(T, String)> for ExtractedInput<T> {
    fn from(value: (T, String)) -> Self {
        ExtractedInput(value.0, value.1)
    }
}

pub trait ExtractableFromInput {
    fn extract_from_input<Tz: TimeZone>(
        date_of_input: DateTime<Tz>,
        input: &str,
    ) -> anyhow::Result<impl Into<ExtractedInput<Self>>>
    where
        Self: Sized;
}
