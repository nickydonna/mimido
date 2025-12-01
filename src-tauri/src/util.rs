use chrono::{DateTime, FixedOffset};
use log::warn;
use newtype::NewType;
use regex::Regex;
use std::{fmt::Display, str::FromStr};

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

pub fn remove_multiple_spaces(s: &str) -> String {
    let re = Regex::new(r" +").unwrap();
    re.replace_all(s, " ").to_string()
}

#[derive(NewType, Debug, Clone, PartialEq, Eq)]
pub struct Href(pub String);

impl std::fmt::Display for Href {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&Href> for String {
    fn from(value: &Href) -> Self {
        value.0.to_string()
    }
}

#[derive(NewType, Debug, Clone, PartialEq, Eq)]
pub struct SyncToken(pub String);

impl std::fmt::Display for SyncToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<&str> for SyncToken {
    fn from(value: &str) -> Self {
        SyncToken(value.to_string())
    }
}

impl From<&SyncToken> for String {
    fn from(value: &SyncToken) -> Self {
        value.0.to_string()
    }
}

#[derive(NewType, Debug, Clone, PartialEq, Eq)]
pub struct Etag(pub String);

impl std::fmt::Display for Etag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&Etag> for String {
    fn from(value: &Etag) -> Self {
        value.0.to_string()
    }
}
