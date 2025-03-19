use crate::{
    caldav::Caldav,
    calendar_items::extract_event,
    establish_connection,
    models::{Calendar, NewEvent, Server},
    util::stringify,
};
use diesel::prelude::*;
use diesel::{delete, insert_into};

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn list_calendars() -> Vec<Calendar> {
    use crate::schema::calendars::dsl as calendars_dsl;
    use crate::schema::servers::dsl as server_dsl;

    let conn = &mut establish_connection();

    server_dsl::servers
        .inner_join(calendars_dsl::calendars)
        .select(Calendar::as_select())
        .load(conn)
        .unwrap()
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn sync_calendar(calendar_id: i32) -> Result<(), String> {
    use crate::schema::calendars::dsl as calendars_dsl;
    use crate::schema::events::dsl as event_dsl;
    use crate::schema::servers::dsl as server_dsl;

    let conn = &mut establish_connection();

    let (server, calendar) = server_dsl::servers
        .inner_join(calendars_dsl::calendars)
        .filter(calendars_dsl::id.eq(calendar_id))
        .select((Server::as_select(), Calendar::as_select()))
        .first::<(Server, Calendar)>(conn)
        .map_err(|e| e.to_string())?;

    let caldav = Caldav::new(server).await?;

    let items = caldav
        .get_calendar_items(&calendar.url)
        .await
        .map_err(|e| e.to_string())?;

    // Clean the events from that calendar
    delete(event_dsl::events)
        .filter(event_dsl::calendar_id.eq(calendar_id))
        .execute(conn)
        .map_err(stringify)?;

    let events = items
        .into_iter()
        .flat_map(|fetched_resource| extract_event(calendar_id, fetched_resource))
        .flatten()
        .collect::<Vec<NewEvent>>();

    insert_into(event_dsl::events)
        .values(events)
        .execute(conn)
        .map(|_| ())
        .map_err(stringify)
}

#[tauri::command(rename_all = "snake_case")]
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
            let calendar_record = find_calendar_by_name(conn, &new_cal.name)?;
            if let Some(calendar) = calendar_record {
                diesel::update(calendars_dsl::calendars)
                    .filter(calendars_dsl::id.eq(calendar.id))
                    .set(calendars_dsl::etag.eq(&new_cal.etag))
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

fn find_calendar_by_name(
    connection: &mut SqliteConnection,
    name: &str,
) -> anyhow::Result<Option<Calendar>> {
    use crate::schema::calendars::dsl as calendars_dsl;

    calendars_dsl::calendars
        .filter(calendars_dsl::name.eq(name))
        .first::<Calendar>(connection)
        .optional()
        .map_err(anyhow::Error::new)
}
