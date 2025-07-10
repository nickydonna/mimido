use std::fmt::Display;

use futures::future::try_join_all;
use http::{HeaderValue, Request, StatusCode, Uri};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client as HyperClient},
    rt::TokioExecutor,
};
use libdav::{
    dav::{mime_types, FoundCollection, WebDavClient, WebDavError},
    names, Depth,
};
use libdav::{CalDavClient, FetchedResource};
use log::info;
use tower_http::auth::AddAuthorization;

use crate::models::{NewCalendar, Server};

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

    pub async fn test(self) -> Result<bool, String> {
        self.caldav_client
            .find_current_user_principal()
            .await
            .map(|_| true)
            .map_err(|e| e.to_string())
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
            .get_properties(
                &collection.href,
                &[&names::DISPLAY_NAME, &names::SYNC_TOKEN],
            )
            .await?;

        let (_, display_name) = &properties[0];
        let (_, sync_token) = &properties[1];

        let uri = self.caldav_client.relative_uri(&collection.href)?;

        if let Some(sync_token) = sync_token {
            self.get_sync_report(&collection.href, sync_token).await?;
        }
        let prop_sync = self
            .caldav_client
            .propfind(
                &uri,
                &[&names::DISPLAY_NAME, &names::GETETAG, &names::SYNC_TOKEN],
                libdav::Depth::Zero,
            )
            .await?;
        check_status(prop_sync.0.status).map_err(WebDavError::BadStatusCode)?;
        print!("{prop_sync:?}");

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
        base_href: impl Display,
        id: impl Display,
        calendar: icalendar::Calendar,
    ) -> anyhow::Result<()> {
        let href = format!("{base_href}{id}.ics");
        info!("{href}");
        info!("{calendar}");
        let v = self
            .caldav_client
            .create_resource(
                &href,
                calendar.to_string().as_bytes().to_vec(),
                mime_types::CALENDAR,
            )
            .await?;
        Ok(())
    }

    pub async fn fetch_changes(
        &self,
        base_href: impl Display,
        id: impl Display,
        calendar: icalendar::Calendar,
    ) -> anyhow::Result<()> {
        let href = format!("{base_href}{id}.ics");
        info!("{href}");
        info!("{calendar}");
        let v = self
            .caldav_client
            .create_resource(
                &href,
                calendar.to_string().as_bytes().to_vec(),
                mime_types::CALENDAR,
            )
            .await?;
        info!("res {v:?}");
        Ok(())
    }

    async fn get_sync_report(
        &self,
        calendar_href: &str,
        sync_token: &str,
    ) -> Result<(), WebDavError> {
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
        println!("{body:?}");
        Ok(())
    }
}

/// Checks if the status code is success. If it is not, return it as an error.
#[inline]
pub(crate) fn check_status(status: StatusCode) -> Result<(), StatusCode> {
    if status.is_success() {
        Ok(())
    } else {
        Err(status)
    }
}
