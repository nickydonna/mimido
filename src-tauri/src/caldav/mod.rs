use std::fmt::Display;

use crate::{
    caldav::{
        create_calendar_component::CreateCalendarComponent,
        delete_calendar_component::DeleteCalendarComponent,
        get_sync_report::{GetSyncReport, GetSyncReportResponse},
        update_calendar_component::UpdateCalendarComponent,
    },
    models::{NewCalendar, server::Server},
    util::{Etag, Href, SyncToken},
};
use futures::future::try_join_all;
use http::{StatusCode, Uri};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::{Client as HyperClient, connect::HttpConnector},
    rt::TokioExecutor,
};
use libdav::{
    CalDavClient, FetchedResource,
    caldav::{FindCalendarHomeSet, FindCalendars, GetCalendarResources},
    dav::GetProperties,
};
use libdav::{
    dav::{FoundCollection, ListResources, WebDavClient},
    names,
    sd::BootstrapError,
};
use tower_http::auth::AddAuthorization;

pub mod create_calendar_component;
pub mod delete_calendar_component;
pub mod get_sync_report;
pub mod update_calendar_component;
pub mod util;

pub type HyperAuthClient =
    CalDavClient<AddAuthorization<HyperClient<HttpsConnector<HttpConnector>, String>>>;

/// Response from a `CreateCalendar` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCalendarResponse {
    /// Whether the calendar was successfully created.
    pub created: bool,
    /// Etag of the created calendar, if returned by the server.
    ///
    /// Note: Some servers don't return etags for collections.
    pub etag: Option<String>,
    // TODO: ctag
    //       https://github.com/apple/ccs-calendarserver/blob/master/doc/Extensions/caldav-ctag.txt
}

#[derive(Debug)]
pub struct Caldav {
    server: Server,
    caldav_client: HyperAuthClient,
}

#[derive(Debug, thiserror::Error)]
pub enum CaldavError {
    #[error("Could not find {0} on xml")]
    NodeNotFound(String),
    #[error("Request returnes {0}")]
    ErrorResponse(StatusCode),
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

    pub async fn list_caldav_calendars(&self) -> anyhow::Result<Vec<NewCalendar>> {
        let urls = self.urls_for_finding_calendars().await?;
        let found_collections = try_join_all(
            urls.iter()
                .map(|url| self.caldav_client.request(FindCalendars::new(url))),
        )
        .await?
        .into_iter()
        .flat_map(|e| e.calendars)
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
        let resources = self.caldav_client.request(ListResources::new(href)).await?;
        let items = self
            .caldav_client
            .request(
                GetCalendarResources::new(href)
                    .with_hrefs(resources.resources.into_iter().map(|e| e.href)),
            )
            .await?;
        Ok(items.resources)
    }

    async fn fetch_calendar_details(
        &self,
        collection: FoundCollection,
    ) -> anyhow::Result<NewCalendar> {
        let properties = self
            .caldav_client
            .request(GetProperties::new(
                &collection.href,
                &[&names::DISPLAY_NAME, &names::SYNC_TOKEN],
            ))
            .await?
            .values;

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
                    .request(FindCalendarHomeSet::new(&principal))
                    // .find_calendar_home_set(&principal)
                    .await?
                    .home_sets;
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

    pub async fn create_component(
        &self,
        base_href: &Href,
        id: impl Display,
        calendar: &icalendar::Calendar,
    ) -> anyhow::Result<Href> {
        let href_str = format!("{base_href}{id}.ics");
        let _ = self
            .caldav_client
            .request(CreateCalendarComponent::new(&href_str, calendar))
            .await?;
        Ok(href_str.into())
    }

    pub async fn update_component(
        &self,
        cmp_href: &Href,
        etag: &Etag,
        calendar: &icalendar::Calendar,
    ) -> anyhow::Result<Option<Etag>> {
        let etag = self
            .caldav_client
            .request(UpdateCalendarComponent::new(&cmp_href.0, &etag.0, calendar))
            .await?
            .etag;
        Ok(etag.map(Etag))
    }

    pub async fn fetch_resource(
        &self,
        calendar_href: &Href,
        href: &Href,
    ) -> anyhow::Result<Option<FetchedResource>> {
        let mut v = self
            .caldav_client
            .request(GetCalendarResources::new(&calendar_href.0).with_hrefs([&href.0]))
            .await?
            .resources;
        Ok(v.pop())
    }

    pub async fn delete_resource(&self, href: &Href, etag: &Etag) -> anyhow::Result<()> {
        self.caldav_client
            .request(DeleteCalendarComponent::new(&href.0, &etag.0))
            .await?;
        Ok(())
    }

    pub async fn get_sync_report(
        &self,
        calendar_href: &Href,
        sync_token: &SyncToken,
    ) -> anyhow::Result<GetSyncReportResponse> {
        let uri = self.caldav_client.relative_uri(calendar_href)?;
        let res = self
            .caldav_client
            .request(GetSyncReport::new(&uri, sync_token))
            .await?;
        Ok(res)
        // let mut body = String::from(r#"<sync-collection xmlns="DAV:">"#);
        // body.push_str(&format!("<sync-token>{sync_token}</sync-token>"));
        // body.push_str(
        //     r#"
        //         <sync-level>1</sync-level>
        //         <prop>
        //             <getetag />
        //         </prop>
        //     </sync-collection>
        //     "#,
        // );
        //
        // let uri = self.caldav_client.relative_uri(calendar_href)?;
        // let request = Request::builder()
        //     .method("REPORT")
        //     .uri(uri)
        //     .header("Content-Type", "application/xml; charset=utf-8")
        //     .header("Depth", HeaderValue::from(Depth::Zero))
        //     .body(body)?;
        //
        // let (head, body) = self.caldav_client.request(request).await?;
        // check_status(head.status)?;
        // let body = std::str::from_utf8(&body)?;
        //
        // let doc = roxmltree::Document::parse(body)?;
        //
        // let sync_token =
        //     get_node_prop_by_name(doc.root_element(), names::SYNC_TOKEN).expect("Sync token");
        // info!("s {sync_token:?}");
        // let responses = get_node_by_name(doc.root_element(), names::RESPONSE)
        //     .ok_or(CaldavError::NodeNotFound("Response".to_string()))?;
        //
        // let result = responses
        //     .descendants()
        //     .filter_map(|res| {
        //         let href = get_node_prop_by_name(res, names::HREF)?;
        //         let status = get_node_prop_by_name(res, names::STATUS)?;
        //         info!("a {href} - {status}");
        //
        //         if status.contains("404") {
        //             Some(CmpSyncResult::Deleted(Href(href)))
        //         } else if status.contains("200") {
        //             let etag = get_node_prop_by_name(res, names::GETETAG).expect("To have etag");
        //             Some(CmpSyncResult::Upserted(Href(href), Etag(etag)))
        //         } else {
        //             None
        //         }
        //     })
        //     .collect::<Vec<CmpSyncResult>>();
        // Ok((sync_token, result))
    }
}
