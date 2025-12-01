use http::Method;
use libdav::{
    dav::WebDavError,
    requests::{DavRequest, ParseResponseError, PreparedRequest},
};

pub struct UpdateCalendarComponent<'a> {
    path: &'a str,
    etag: &'a str,
    component: &'a icalendar::Calendar,
}

/// Response from a `CreateCalendar` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCalendarResponse {
    /// Whether the calendar was successfully created.
    pub created: bool,
    /// Etag of the created calendar, if returned by the server.
    ///
    /// Note: Some servers don't return etags for collections.
    pub etag: Option<String>,
    // TODO: ctag
    //       https://github.com/apple/ccs-calendarserver/blob/master/doc/Extensions/caldav-ctag.txt
}

impl<'a> UpdateCalendarComponent<'a> {
    /// Create a new `CreateCalendar` request for the given path.
    ///
    /// The path should be a collection path relative to the server's base URL.
    #[must_use]
    pub fn new(path: &'a str, etag: &'a str, component: &'a icalendar::Calendar) -> Self {
        Self {
            path,
            etag,
            component,
        }
    }
}

impl DavRequest for UpdateCalendarComponent<'_> {
    type Response = UpdateCalendarResponse;
    type ParseError = ParseResponseError;
    type Error<E> = WebDavError<E>;

    // FIXME: error type needs refinement
    fn prepare_request(&self) -> Result<PreparedRequest, http::Error> {
        let body = self.component.to_string();

        Ok(PreparedRequest {
            method: Method::from_bytes(b"PUT")?,
            path: self.path.to_string(),
            body,
            headers: vec![("If-Match".into(), self.etag.into())],
        })
    }

    fn parse_response(
        &self,
        parts: &http::response::Parts,
        _body: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        let created = parts.status.is_success();

        if !created {
            return Err(ParseResponseError::BadStatusCode(parts.status));
        }

        let etag = parts
            .headers
            .get("etag")
            .map(|hv| std::str::from_utf8(hv.as_bytes()))
            .transpose()?
            .map(str::to_string);

        Ok(UpdateCalendarResponse { created, etag })
    }
}
