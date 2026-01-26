use crate::{
    caldav::{
        Caldav,
        get_sync_report::{GetSyncReportResponse, SyncResult},
    },
    commands::errors::CommandError,
    db_conn::DbConn,
    models::{
        Calendar, NewVCmp, VCmp,
        model_traits::{ById, CalendarAndSyncStatus, DeleteAllByCalendar, ListAll, SetSyncedAt},
        server::Server,
        vevent::VEvent,
        vtodo::VTodo,
    },
    util::{Etag, Href, filter_err_and_map},
};
use anyhow::anyhow;
use chrono::Utc;
use diesel::{dsl::update, prelude::*};
use futures::future::join_all;
use itertools::Itertools;
use tauri::async_runtime::spawn_blocking;

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn sync_all_calendars() -> Result<(), CommandError> {
    use crate::schema::servers::dsl as server_dsl;

    let conn = DbConn::new().await?;
    // Get all servers and fetch their calendars
    let servers = Server::list_all(conn.clone()).await?;
    let syncs = servers
        .iter()
        .map(|server| fetch_calendars_from_caldav(server.id));
    join_all(syncs)
        .await
        .into_iter()
        .collect::<Result<Vec<Vec<Calendar>>, CommandError>>()?;

    let calendars = list_calendars().await?;

    // Sync them sequentially
    for cal in calendars {
        sync_calendar(cal.id).await?;
    }

    spawn_blocking(move || {
        let now = chrono::Utc::now().timestamp();
        let conn = &mut *conn.0.lock().unwrap();

        update(server_dsl::servers)
            .filter(server_dsl::id.eq_any(servers.iter().map(|s| s.id)))
            .set(server_dsl::last_sync.eq(now))
            .execute(conn)
    })
    .await??;

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn sync_calendar(calendar_id: i32) -> Result<(), CommandError> {
    let conn = DbConn::new().await?;

    let (server, calendar) = Calendar::by_id_with_server(conn.clone(), calendar_id).await?;

    let caldav = Caldav::new(server).await?;

    let items = caldav.get_calendar_items(&calendar.url).await?;

    // Clean the events from that calendar
    VEvent::delete_all_by_calendar(conn.clone(), calendar_id).await?;
    VTodo::delete_all_by_calendar(conn.clone(), calendar_id).await?;

    let _ = join_all(
        items
            .iter()
            .map(|fetched_resource| NewVCmp::from_resource(calendar_id, fetched_resource))
            .filter_map(|new_v| match new_v {
                Ok(new_v) => Some(new_v),
                Err(e) => {
                    log::warn!("{e}");
                    None
                }
            })
            .map(|cmp| cmp.upsert_by_href(conn.clone())),
    )
    .await;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn list_calendars() -> Result<Vec<Calendar>, CommandError> {
    let calendars = Calendar::list_all().await?;
    Ok(calendars)
}

#[tauri::command()]
#[specta::specta]
pub async fn set_default_calendar(calendar_id: i32) -> Result<(), CommandError> {
    Calendar::set_default_calendar(calendar_id).await?;
    Ok(())
}

pub async fn internal_super_sync_calendar(calendar_id: i32) -> Result<(), CommandError> {
    let conn = DbConn::new().await?;
    let (server, calendar) = Calendar::by_id_with_server(conn.clone(), calendar_id).await?;
    let Some(sync_token) = calendar.sync_token.clone() else {
        return Ok(());
    };
    let cal_href = Href(calendar.url.clone());
    let caldav = Caldav::new(server).await?;

    let not_sync_cmp = {
        let not_synced_vevent = VEvent::by_calendar_id_and_not_sync(conn.clone(), calendar_id)
            .await?
            .into_iter()
            .map(VCmp::Event)
            .collect::<Vec<VCmp>>();

        let not_synced_vtodo = VTodo::by_calendar_id_and_not_sync(conn.clone(), calendar_id)
            .await?
            .into_iter()
            .map(VCmp::Todo)
            .collect::<Vec<VCmp>>();

        vec![not_synced_vevent, not_synced_vtodo]
            .into_iter()
            .concat()
    };

    let synced_at = Utc::now();
    for vcmp in not_sync_cmp {
        let uid = vcmp.get_uid();
        let cal = icalendar::Calendar::new().push(vcmp.clone()).done();
        let (_, etag) = caldav
            .create_component(&cal_href, uid.clone(), &cal)
            .await?;

        vcmp.set_synced_at(conn.clone(), etag, synced_at).await?;
    }

    let out_of_sync_cmp = if calendar.synced_at.is_some() {
        let out_of_sync_vtodo = VTodo::by_calendar_id_and_out_of_sync(conn.clone(), calendar_id)
            .await?
            .into_iter()
            .map(VCmp::Todo)
            .collect::<Vec<VCmp>>();
        let out_of_sync_vevent = VEvent::by_calendar_id_and_out_of_sync(conn.clone(), calendar_id)
            .await?
            .into_iter()
            .map(VCmp::Event)
            .collect::<Vec<VCmp>>();

        vec![out_of_sync_vtodo, out_of_sync_vevent]
            .into_iter()
            .concat()
    } else {
        Vec::<VCmp>::new()
    };
    for vcmp in out_of_sync_cmp {
        let Some(href) = vcmp.get_href() else {
            continue;
        };
        let Some(etag) = vcmp.get_etag() else {
            continue;
        };
        // let vcmp: icalendar::CalendarComponent = vevent.clone().into();
        let cal = icalendar::Calendar::new().push(vcmp.clone()).done();
        let etag = caldav
            .update_component(&Href(href), &Etag(etag), &cal)
            .await;

        let etag = etag?;
        vcmp.set_synced_at(conn.clone(), etag, synced_at).await?;
    }

    let GetSyncReportResponse {
        sync_token: new_sync_token,
        report,
    } = caldav
        .get_sync_report(&cal_href, &sync_token.into())
        .await?;

    let _ = join_all(
        report
            .iter()
            .filter_map(|r| match r {
                SyncResult::Upserted(_, _) => None,
                SyncResult::Deleted(href) => Some(href),
            })
            .map(async move |href| {
                let conn = DbConn::new().await?;
                // Try to delete both entries since we don't know which type it is
                let vtodo_del = VTodo::try_delete_by_href(conn.clone(), href).await;
                let vevent_del = VEvent::try_delete_by_href(conn, href).await;
                vevent_del.and(vtodo_del)
            }),
    )
    .await;

    let results = report
        .iter()
        .filter_map(|r| match r {
            SyncResult::Upserted(href, etag) => Some((href, etag)),
            SyncResult::Deleted(_) => None,
        })
        .map(|(href, _)| caldav.fetch_resource(&cal_href, href));

    let r = join_all(results)
        .await
        .into_iter()
        .filter_map(filter_err_and_map)
        .flatten()
        .map(|f| NewVCmp::from_resource(calendar.id, &f))
        .filter_map(filter_err_and_map);

    let _ = join_all(r.map(async |cmp| cmp.upsert_by_href(conn.clone()).await.unwrap()))
        .await
        .into_iter()
        .collect::<Vec<VCmp>>();

    calendar.update_sync_token(&new_sync_token).await?;

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn super_sync_calendar(calendar_id: i32) -> Result<(), CommandError> {
    internal_super_sync_calendar(calendar_id).await
}

#[tauri::command()]
#[specta::specta]
pub async fn fetch_calendars_from_caldav(server_id: i32) -> Result<Vec<Calendar>, CommandError> {
    let conn = DbConn::new().await?;

    let server = Server::by_id(conn, server_id)
        .await?
        .ok_or(anyhow!("Server {server_id} not found"))?;

    let caldav = Caldav::new(server).await?;
    let found_calendars = caldav.list_caldav_calendars().await?;

    let calendars = join_all(found_calendars.into_iter().map(Calendar::create_or_update))
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<Calendar>>();
    Ok(calendars)
}
