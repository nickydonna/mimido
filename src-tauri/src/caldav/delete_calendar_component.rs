use http::Method;
use libdav::{
    dav::WebDavError,
    requests::{DavRequest, ParseResponseError, PreparedRequest},
};

pub struct DeleteCalendarComponent<'a> {
    path: &'a str,
    etag: &'a str,
}

/// Response from a `CreateCalendar` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteCalendarComponentResponse {
    pub deleted: bool,
}

impl<'a> DeleteCalendarComponent<'a> {
    /// Create a new `CreateCalendar` request for the given path.
    ///
    /// The path should be a collection path relative to the server's base URL.
    #[must_use]
    pub fn new(path: &'a str, etag: &'a str) -> Self {
        Self { path, etag }
    }
}

impl DavRequest for DeleteCalendarComponent<'_> {
    type Response = DeleteCalendarComponentResponse;
    type ParseError = ParseResponseError;
    type Error<E> = WebDavError<E>;

    // FIXME: error type needs refinement
    fn prepare_request(&self) -> Result<PreparedRequest, http::Error> {
        Ok(PreparedRequest {
            method: Method::from_bytes(b"DELETE")?,
            path: self.path.to_string(),
            body: "".to_string(),
            headers: vec![("If-Match".into(), self.etag.into())],
        })
    }

    fn parse_response(
        &self,
        parts: &http::response::Parts,
        _body: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        let deleted = parts.status.is_success();

        if !deleted {
            return Err(ParseResponseError::BadStatusCode(parts.status));
        }

        Ok(DeleteCalendarComponentResponse { deleted })
    }
}
