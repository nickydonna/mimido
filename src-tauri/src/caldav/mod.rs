use std::fmt::Display;

use crate::{
    caldav::get_sync_report::{GetSyncReport, GetSyncReportResponse},
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
    dav::{Delete, GetProperties, PutResource, PutResourceResponse, mime_types},
};
use libdav::{
    dav::{FoundCollection, ListResources, WebDavClient},
    names,
    sd::BootstrapError,
};
use tower_http::auth::AddAuthorization;

pub mod get_sync_report;
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
            synced_at: None,
        })
    }

    async fn urls_for_finding_calendars(&self) -> anyhow::Result<Vec<Uri>> {
        let urls = match self.caldav_client.find_current_user_principal().await? {
            Some(principal) => {
                let home_set = self
                    .caldav_client
                    .request(FindCalendarHomeSet::new(&principal))
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
    ) -> anyhow::Result<(Href, Option<Etag>)> {
        let href_str = format!("{base_href}{id}.ics");
        let body = calendar.to_string();
        let create_req = PutResource::new(&href_str).create(body, mime_types::CALENDAR);
        let PutResourceResponse { etag } = self.caldav_client.request(create_req).await?;

        Ok((href_str.into(), etag.map(Etag)))
    }

    pub async fn update_component(
        &self,
        cmp_href: &Href,
        etag: &Etag,
        calendar: &icalendar::Calendar,
    ) -> anyhow::Result<Option<Etag>> {
        let body = calendar.to_string();
        let href = cmp_href.to_string();
        let create_req = PutResource::new(&href).update(body, mime_types::CALENDAR, etag);
        let PutResourceResponse { etag } = self.caldav_client.request(create_req).await?;

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
        let request = Delete::new(&href.0).with_etag(etag);
        self.caldav_client.request(request).await?;
        Ok(())
    }

    pub async fn get_sync_report(
        &self,
        calendar_href: &Href,
        sync_token: &SyncToken,
    ) -> anyhow::Result<GetSyncReportResponse> {
        let res = self
            .caldav_client
            .request(GetSyncReport::new(calendar_href, sync_token))
            .await?;
        Ok(res)
    }
}
