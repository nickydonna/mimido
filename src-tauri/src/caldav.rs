use std::fmt::Display;

use crate::models::{NewCalendar, Server};
use futures::future::try_join_all;
use http::{HeaderValue, Request, StatusCode, Uri};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client as HyperClient},
    rt::TokioExecutor,
};
use libdav::{
    dav::{mime_types, FoundCollection, WebDavClient},
    names,
    sd::BootstrapError,
    Depth, PropertyName,
};
use libdav::{CalDavClient, FetchedResource};
use log::info;
use newtype::NewType;
use tower_http::auth::AddAuthorization;

pub type HyperAuthClient =
    CalDavClient<AddAuthorization<HyperClient<HttpsConnector<HttpConnector>, String>>>;

#[derive(Debug)]
pub struct Caldav {
    server: Server,
    caldav_client: HyperAuthClient,
}

#[derive(Debug, thiserror::Error)]
enum CaldavError {
    #[error("Could not find {0} on xml")]
    NodeNotFound(String),
    #[error("Request returnes {0}")]
    ErrorResponse(StatusCode),
}

#[derive(NewType, Debug)]
pub struct Href(pub String);
#[derive(NewType, Debug)]
pub struct Etag(pub String);

impl std::fmt::Display for Href {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Etag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum CmpSyncResult {
    Deleted(Href),
    Upserted(Href, Etag),
}

impl Caldav {
    pub async fn new(server: Server) -> Result<Self, BootstrapError> {
        let uri = Uri::try_from(server.server_url.clone()).unwrap();
        let https_connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_or_http()
            .enable_http1()
            .build();
        let https_client = HyperClient::builder(TokioExecutor::new()).build(https_connector);
        let https_client = AddAuthorization::basic(https_client, &server.user, &server.password);
        let webdav = WebDavClient::new(uri, https_client);
        let client = CalDavClient::bootstrap_via_service_discovery(webdav).await?;
        Ok(Caldav {
            server,
            caldav_client: client,
        })
    }

    pub async fn test(self) -> anyhow::Result<bool> {
        let res = self
            .caldav_client
            .find_current_user_principal()
            .await
            .map(|_| true)?;
        Ok(res)
    }

    pub async fn list_calendars(&self) -> anyhow::Result<Vec<NewCalendar>> {
        let urls = self.urls_for_finding_calendars().await?;
        let found_collections = try_join_all(
            urls.iter()
                .map(|url| self.caldav_client.find_calendars(url)),
        )
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<FoundCollection>>();

        let calendars = try_join_all(
            found_collections
                .into_iter()
                .map(|collection| self.fetch_calendar_details(collection)),
        )
        .await?;
        Ok(calendars)
    }

    pub async fn get_calendar_items(&self, href: &str) -> anyhow::Result<Vec<FetchedResource>> {
        let resources = self.caldav_client.list_resources(href).await?;
        let items = self
            .caldav_client
            .get_calendar_resources(href, resources.into_iter().map(|e| e.href))
            .await?;
        Ok(items)
    }

    async fn fetch_calendar_details(
        &self,
        collection: FoundCollection,
    ) -> anyhow::Result<NewCalendar> {
        let properties = self
            .caldav_client
            .get_properties(
                &collection.href,
                &[&names::DISPLAY_NAME, &names::SYNC_TOKEN],
            )
            .await?;

        let (_, display_name) = &properties[0];
        let (_, sync_token) = &properties[1];

        Ok(NewCalendar {
            url: collection.href.clone(),
            name: display_name.clone().unwrap_or(collection.href.clone()),
            etag: collection.etag,
            is_default: false,
            sync_token: sync_token.clone(),
            server_id: self.server.id,
        })
    }

    async fn urls_for_finding_calendars(&self) -> anyhow::Result<Vec<Uri>> {
        let urls = match self.caldav_client.find_current_user_principal().await? {
            Some(principal) => {
                let home_set = self
                    .caldav_client
                    .find_calendar_home_set(&principal)
                    .await?;
                if home_set.is_empty() {
                    vec![self.caldav_client.base_url().clone()]
                } else {
                    home_set
                }
            }
            None => vec![self.caldav_client.base_url().clone()],
        };
        Ok(urls)
    }

    pub async fn create_cmp(
        &self,
        base_href: &Href,
        id: impl Display,
        calendar: icalendar::Calendar,
    ) -> anyhow::Result<Href> {
        let href_str = format!("{base_href}{id}.ics");
        let _ = self
            .caldav_client
            .create_resource(
                &href_str,
                calendar.to_string().as_bytes().to_vec(),
                mime_types::CALENDAR,
            )
            .await?;
        Ok(href_str.into())
    }

    pub async fn fetch_resource(
        &self,
        calendar_href: &Href,
        href: &Href,
    ) -> anyhow::Result<Option<FetchedResource>> {
        let mut v = self
            .caldav_client
            .get_calendar_resources(&calendar_href.0, [&href.0])
            .await?;
        Ok(v.pop())
    }

    pub async fn get_sync_report(
        &self,
        calendar_href: &Href,
        sync_token: impl Display,
    ) -> anyhow::Result<(String, Vec<CmpSyncResult>)> {
        let mut body = String::from(r#"<sync-collection xmlns="DAV:">"#);
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

        let uri = self.caldav_client.relative_uri(calendar_href)?;
        let request = Request::builder()
            .method("REPORT")
            .uri(uri)
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", HeaderValue::from(Depth::Zero))
            .body(body)?;

        let (head, body) = self.caldav_client.request(request).await?;
        check_status(head.status)?;
        let body = std::str::from_utf8(&body)?;

        let doc = roxmltree::Document::parse(body)?;

        let sync_token =
            get_node_prop_by_name(doc.root_element(), names::SYNC_TOKEN).expect("Sync token");
        info!("s {sync_token:?}");
        let responses = get_node_by_name(doc.root_element(), names::RESPONSE)
            .ok_or(CaldavError::NodeNotFound("Response".to_string()))?;

        let result = responses
            .descendants()
            .filter_map(|res| {
                let href = get_node_prop_by_name(res, names::HREF)?;
                let status = get_node_prop_by_name(res, names::STATUS)?;
                info!("a {href} - {status}");

                if status.contains("404") {
                    Some(CmpSyncResult::Deleted(Href(href)))
                } else if status.contains("200") {
                    let etag = get_node_prop_by_name(res, names::GETETAG).expect("To have etag");
                    Some(CmpSyncResult::Upserted(Href(href), Etag(etag)))
                } else {
                    None
                }
            })
            .collect::<Vec<CmpSyncResult>>();
        Ok((sync_token, result))
    }
}

#[inline]
fn get_node_prop_by_name(node: roxmltree::Node, prop: PropertyName) -> Option<String> {
    get_node_by_name(node, prop).and_then(|node| node.text().map(str::to_string))
}

#[inline]
fn get_node_by_name<'a, 'b>(
    node: roxmltree::Node<'a, 'b>,
    prop: PropertyName<'_, '_>,
) -> Option<roxmltree::Node<'a, 'b>> {
    node.descendants().find(|node| node.tag_name() == prop)
}
/// Checks if the status code is success. If it is not, return it as an error.
#[inline]
fn check_status(status: StatusCode) -> Result<(), CaldavError> {
    if status.is_success() {
        Ok(())
    } else {
        Err(CaldavError::ErrorResponse(status))
    }
}
