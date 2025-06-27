use chrono::{DateTime, Utc};

pub fn stringify<T: ToString>(e: T) -> String {
    format!("Error code: {}", e.to_string())
}

pub struct DateTimeStr(pub String);

impl TryInto<DateTime<Utc>> for DateTimeStr {
    type Error = String;

    fn try_into(self) -> Result<DateTime<Utc>, Self::Error> {
        Ok(DateTime::parse_from_rfc3339(&self.0)
            .map_err(stringify)?
            .to_utc())
    }
}
