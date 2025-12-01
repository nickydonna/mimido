use http::{HeaderValue, Method, Uri};
use libdav::{
    Depth, names,
    requests::{DavRequest, PreparedRequest},
};
use log::info;

use crate::{
    caldav::{
        CaldavError,
        util::{check_status, get_node_by_name, get_node_prop_by_name},
    },
    util::{Etag, Href, SyncToken},
};

pub struct GetSyncReport<'a> {
    uri: &'a Uri,
    sync_token: &'a SyncToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncResult {
    Deleted(Href),
    Upserted(Href, Etag),
}

/// Response from a `CreateCalendar` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSyncReportResponse {
    pub sync_token: SyncToken,
    pub report: Vec<SyncResult>,
}

impl<'a> GetSyncReport<'a> {
    /// Create a new `CreateCalendar` request for the given path.
    ///
    /// The path should be a collection path relative to the server's base URL.
    #[must_use]
    pub fn new(uri: &'a Uri, sync_token: &'a SyncToken) -> Self {
        Self { uri, sync_token }
    }
}

impl DavRequest for GetSyncReport<'_> {
    type Response = GetSyncReportResponse;
    type ParseError = anyhow::Error;
    type Error<E> = anyhow::Error;

    // FIXME: error type needs refinement
    fn prepare_request(&self) -> Result<PreparedRequest, http::Error> {
        let mut body = String::from(r#"<sync-collection xmlns="DAV:">"#);
        let sync_token = self.sync_token.to_string();
        body.push_str(&format!("<sync-token>{sync_token}</sync-token>"));
        body.push_str(
            r#"
                <sync-level>1</sync-level>
                <prop>
                    <getetag />
                </prop>
            </sync-collection>
            "#,
        );
        let depth_zero = HeaderValue::from(Depth::Zero).to_str().unwrap().to_string();

        Ok(PreparedRequest {
            method: Method::from_bytes(b"REPORT")?,
            path: self.uri.to_string(),
            body,
            headers: vec![
                (
                    "Content-Type".into(),
                    "application/xml; charset=utf-8".into(),
                ),
                ("Depth".into(), depth_zero),
            ],
        })
    }

    fn parse_response(
        &self,
        parts: &http::response::Parts,
        body: &[u8],
    ) -> Result<Self::Response, anyhow::Error> {
        check_status(parts.status)?;

        let body = std::str::from_utf8(body)?;
        let doc = roxmltree::Document::parse(body)?;

        let sync_token: SyncToken = get_node_prop_by_name(doc.root_element(), names::SYNC_TOKEN)
            .expect("Sync token")
            .into();
        info!("s {sync_token:?}");
        let responses = get_node_by_name(doc.root_element(), names::RESPONSE)
            .ok_or(CaldavError::NodeNotFound("Response".to_string()))?;

        let report = responses
            .descendants()
            .filter_map(|res| {
                let href = get_node_prop_by_name(res, names::HREF)?;
                let status = get_node_prop_by_name(res, names::STATUS)?;
                info!("a {href} - {status}");

                if status.contains("404") {
                    Some(SyncResult::Deleted(Href(href)))
                } else if status.contains("200") {
                    let etag = get_node_prop_by_name(res, names::GETETAG).expect("To have etag");
                    Some(SyncResult::Upserted(Href(href), Etag(etag)))
                } else {
                    None
                }
            })
            .collect::<Vec<SyncResult>>();

        Ok(GetSyncReportResponse { sync_token, report })
    }
}
