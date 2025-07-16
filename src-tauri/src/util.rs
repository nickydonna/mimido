use chrono::{DateTime, FixedOffset};
use log::warn;
use std::fmt::Display;

pub fn stringify<T: ToString>(e: T) -> String {
    format!("Error code: {}", e.to_string())
}

pub struct DateTimeStr(pub String);

impl TryInto<DateTime<FixedOffset>> for DateTimeStr {
    type Error = String;

    fn try_into(self) -> Result<DateTime<FixedOffset>, Self::Error> {
        DateTime::parse_from_rfc3339(&self.0).map_err(stringify)
    }
}

/// Useful for [`Iterator::filter_map`] to filter errs, but log to console
pub fn filter_err_and_map<O, E: Display>(res: Result<O, E>) -> Option<O> {
    match res {
        Ok(val) => Some(val),
        Err(e) => {
            warn!("{e}");
            None
        }
    }
}
