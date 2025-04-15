use chrono::DateTime;

pub trait ToInput {
    fn to_input(&self, date_of_input: DateTime<chrono_tz::Tz>) -> String;
}

pub trait ExtractableFromInput {
    fn extract_from_input(
        date_of_input: DateTime<chrono_tz::Tz>,
        input: &str,
    ) -> Result<(Self, String), String>
    where
        Self: Sized;
}
