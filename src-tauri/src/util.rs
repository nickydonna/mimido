use chrono::{DateTime, FixedOffset};
use log::warn;
use std::fmt::Display;

pub fn stringify<T: ToString>(e: T) -> String {
    format!("Error code: {}", e.to_string())
}

pub struct DateTimeStr(pub String);

impl TryInto<DateTime<FixedOffset>> for DateTimeStr {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<DateTime<FixedOffset>, Self::Error> {
        let val = DateTime::parse_from_rfc3339(&self.0)?;
        Ok(val)
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
