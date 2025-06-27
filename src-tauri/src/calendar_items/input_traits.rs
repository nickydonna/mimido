use chrono::DateTime;

pub trait ToInput {
    fn to_input(&self, date_of_input: DateTime<chrono_tz::Tz>) -> String;
}

/// Tuple with the result, and the input string
pub struct ExtractedInput<T>(pub T, pub String);

impl<T> From<(T, String)> for ExtractedInput<T> {
    fn from(value: (T, String)) -> Self {
        ExtractedInput(value.0, value.1)
    }
}

pub trait ExtractableFromInput {
    fn extract_from_input(
        date_of_input: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<impl Into<ExtractedInput<Self>>, String>
    where
        Self: Sized;
}
