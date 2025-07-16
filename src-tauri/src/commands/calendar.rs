use crate::{
    caldav::{Caldav, CmpSyncResult, Href},
    establish_connection,
    models::{
        vevent::{NewVEvent, VEvent},
        vtodo::{NewVTodo, VTodo},
        Calendar, NewVCmp, Server, VCmp,
    },
    util::{filter_err_and_map, stringify},
};
use diesel::insert_into;
use diesel::{prelude::*, update};
use futures::future::join_all;
use log::{info, warn};

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn sync_all_calendars() -> Result<(), String> {
    use crate::schema::servers::dsl as server_dsl;

    // Get all servers and fetch their calendars
    let servers = list_servers()?;
    let syncs = servers.iter().map(|server| fetch_calendars(server.id));
    let r = join_all(syncs)
        .await
        .into_iter()
        .collect::<Result<Vec<Vec<Calendar>>, String>>()?;
    info!("{r:?}");

    let calendars = list_calendars()?;
    let syncs = calendars.iter().map(|cal| sync_calendar(cal.id));

    // Sync all the calendar events
    let _ = join_all(syncs)
        .await
        .into_iter()
        .collect::<Result<Vec<()>, String>>()?;

    let now = chrono::Utc::now().timestamp();
    update(server_dsl::servers)
        .filter(server_dsl::id.eq_any(servers.iter().map(|s| s.id)))
        .set(server_dsl::last_sync.eq(now))
        .execute(&mut establish_connection())
        .map_err(stringify)?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn list_calendars() -> Result<Vec<Calendar>, String> {
    use crate::schema::calendars::dsl as calendars_dsl;
    use crate::schema::servers::dsl as server_dsl;

    let conn = &mut establish_connection();

    server_dsl::servers
        .inner_join(calendars_dsl::calendars)
        .select(Calendar::as_select())
        .load(conn)
        .map_err(stringify)
}

#[tauri::command()]
#[specta::specta]
pub fn set_default_calendar(calendar_id: i32) -> Result<(), String> {
    use crate::schema::calendars::dsl as calendars_dsl;

    let conn = &mut establish_connection();
    conn.transaction::<(), diesel::result::Error, _>(|conn| {
        diesel::update(calendars_dsl::calendars)
            .set(calendars_dsl::is_default.eq(false))
            .execute(conn)?;

        diesel::update(calendars_dsl::calendars)
            .filter(calendars_dsl::id.eq(calendar_id))
            .set(calendars_dsl::is_default.eq(true))
            .execute(conn)?;
        Ok(())
    })
    .map_err(|_| "Could not update default".to_string())
}

fn list_servers() -> Result<Vec<Server>, String> {
    use crate::schema::servers::dsl::*;

    let conn = &mut establish_connection();
    servers
        .select(Server::as_select())
        .load(conn)
        .map_err(stringify)
}

#[tauri::command()]
#[specta::specta]
pub async fn sync_calendar(calendar_id: i32) -> Result<(), String> {
    let conn = &mut establish_connection();

    let (server, calendar) =
        Calendar::by_id_with_server(conn, calendar_id).map_err(|e| e.to_string())?;

    let caldav = Caldav::new(server).await?;

    let items = caldav
        .get_calendar_items(&calendar.url)
        .await
        .map_err(|e| e.to_string())?;

    // Clean the events from that calendar
    // VEvent::delete_all(conn, calendar_id).map_err(|e| e.to_string())?;
    // VTodo::delete_all(conn, calendar_id).map_err(|e| e.to_string())?;

    let events = items
        .iter()
        .map(|fetched_resource| NewVCmp::from_resource(calendar_id, fetched_resource))
        .filter_map(|new_v| match new_v {
            Ok(new_v) => Some(new_v),
            Err(e) => {
                warn!("{e}");
                None
            }
        })
        .flat_map(|cmp| cmp.upsert_by_href(conn))
        .collect::<Vec<VCmp>>();

    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn super_sync_calendar(calendar_id: i32) -> Result<(), String> {
    let conn = &mut establish_connection();

    let (server, calendar) = Calendar::by_id_with_server(conn, calendar_id).map_err(stringify)?;

    let Some(sync_token) = calendar.sync_token.clone() else {
        return Ok(());
    };
    let caldav = Caldav::new(server).await?;
    let (new_sync_token, results) = caldav
        .get_sync_report(&calendar.url, &sync_token)
        .await
        .map_err(stringify)?;

    let calendar_href = Href(calendar.url.clone());
    let del_res = results
        .iter()
        .filter_map(|r| match r {
            CmpSyncResult::Upserted(_, _) => None,
            CmpSyncResult::Deleted(href) => Some(href),
        })
        .map(|href| {
            // Try to delete both entries since we don't know which type it is
            let vtodo_del = VTodo::try_delete_by_href(conn, href);
            let vevent_del = VEvent::try_delete_by_href(conn, href);
            vevent_del.and(vtodo_del)
        })
        .collect::<Vec<anyhow::Result<bool>>>();
    info!("Del {del_res:?}");

    let results = results
        .iter()
        .filter_map(|r| match r {
            CmpSyncResult::Upserted(href, etag) => Some((href, etag)),
            CmpSyncResult::Deleted(_) => None,
        })
        .map(|(href, _)| caldav.fetch_resource(&calendar_href, href));

    let r = join_all(results)
        .await
        .into_iter()
        .filter_map(filter_err_and_map)
        .flatten()
        .map(|f| NewVCmp::from_resource(calendar.id, &f))
        .filter_map(filter_err_and_map)
        .map(|cmp| {
            match &cmp {
                NewVCmp::Todo(new_vtodo) => info!("{}", new_vtodo.summary),
                NewVCmp::Event(new_vevent) => info!("{}", new_vevent.summary),
            }
            cmp.upsert_by_href(conn).unwrap()
        })
        .collect::<Vec<VCmp>>();
    calendar
        .update_sync_token(conn, &new_sync_token)
        .map_err(stringify)?;
    Ok(())
}

#[tauri::command()]
#[specta::specta]
pub async fn fetch_calendars(server_id: i32) -> Result<Vec<Calendar>, String> {
    use crate::schema::calendars::dsl as calendars_dsl;
    use crate::schema::servers::dsl as server_dsl;

    let conn = &mut establish_connection();

    let server = server_dsl::servers
        .filter(server_dsl::id.eq(server_id))
        .select(Server::as_select())
        .first(conn)
        .unwrap();

    let caldav = Caldav::new(server).await?;
    let found_calendars = caldav.list_calendars().await.map_err(stringify)?;

    let calendars = found_calendars
        .into_iter()
        .flat_map(|new_cal| -> anyhow::Result<Calendar> {
            let calendar_record = Calendar::by_name(conn, &new_cal.name)?;
            if let Some(calendar) = calendar_record {
                diesel::update(calendars_dsl::calendars)
                    .filter(calendars_dsl::id.eq(calendar.id))
                    .set((
                        calendars_dsl::etag.eq(&new_cal.etag),
                        calendars_dsl::sync_token.eq(&new_cal.sync_token),
                    ))
                    .returning(Calendar::as_select())
                    .get_result(conn)
                    .map_err(anyhow::Error::new)
            } else {
                diesel::insert_into(calendars_dsl::calendars)
                    .values(&new_cal)
                    .returning(Calendar::as_select())
                    .get_result(conn)
                    .map_err(anyhow::Error::new)
            }
        })
        .collect::<Vec<Calendar>>();
    Ok(calendars)
}
