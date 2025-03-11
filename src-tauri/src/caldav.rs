use futures::future::try_join_all;
use http::Uri;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client as HyperClient},
    rt::TokioExecutor,
};
use libdav::{
    dav::{FoundCollection, WebDavClient},
    names,
};
use libdav::{CalDavClient, FetchedResource};
use tower_http::auth::AddAuthorization;

use crate::models::{Calendar, NewCalendar, Server};

pub type HyperAuthClient =
    CalDavClient<AddAuthorization<HyperClient<HttpsConnector<HttpConnector>, String>>>;

#[derive(Debug)]
pub struct Caldav {
    server: Server,
    caldav_client: HyperAuthClient,
}

impl Caldav {
    pub async fn new(server: Server) -> Result<Self, String> {
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
        let Ok(client) = CalDavClient::bootstrap_via_service_discovery(webdav).await else {
            return Err("Couldn't discover".to_string());
        };
        Ok(Caldav {
            server,
            caldav_client: client,
        })
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

    pub async fn get_calendar_items(&self, href: &str) -> Result<Vec<FetchedResource>, String> {
        let resources = self
            .caldav_client
            .list_resources(href)
            .await
            .map_err(|e| e.to_string())?;
        self.caldav_client
            .get_calendar_resources(href, resources.into_iter().map(|e| e.href))
            .await
            .map_err(|e| e.to_string())
    }

    async fn fetch_calendar_details(
        &self,
        collection: FoundCollection,
    ) -> anyhow::Result<NewCalendar> {
        let properties = self
            .caldav_client
            .get_properties(&collection.href, &[&names::DISPLAY_NAME])
            .await?;
        let name = properties
            .into_iter()
            .find(|p| p.0 == &names::DISPLAY_NAME && p.1.is_some())
            .and_then(|p| p.1);

        Ok(NewCalendar {
            url: collection.href.clone(),
            name: name.unwrap_or(collection.href.clone()),
            etag: collection.etag,
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
}
