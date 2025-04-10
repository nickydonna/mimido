use crate::{
    caldav::Caldav,
    calendar_items::extract_todo,
    establish_connection,
    models::{event::NewEvent, Calendar, NewTodo, Server},
    util::stringify,
};
use diesel::{delete, insert_into};
use diesel::{prelude::*, update};
use futures::future::join_all;

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn sync_all_calendars() -> Result<(), String> {
    use crate::schema::servers::dsl as server_dsl;

    // Get all servers and fetch their calendars
    let servers = list_servers()?;
    let syncs = servers.iter().map(|server| fetch_calendars(server.id));
    let _ = join_all(syncs)
        .await
        .into_iter()
        .collect::<Result<Vec<Vec<Calendar>>, String>>()?;

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
    use crate::schema::calendars::dsl as calendars_dsl;
    use crate::schema::events::dsl as event_dsl;
    use crate::schema::servers::dsl as server_dsl;
    use crate::schema::todos::dsl as todo_dsl;

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
    // Clean the todos from that calendar
    delete(todo_dsl::todos)
        .filter(todo_dsl::calendar_id.eq(calendar_id))
        .execute(conn)
        .map_err(stringify)?;

    let events = items
        .iter()
        .flat_map(|fetched_resource| NewEvent::new_from_resource(calendar_id, fetched_resource))
        .flatten()
        .collect::<Vec<NewEvent>>();

    insert_into(event_dsl::events)
        .values(events)
        .execute(conn)
        .map(|_| ())
        .map_err(stringify)?;

    let todos = items
        .iter()
        .flat_map(|fetched_resource| extract_todo(calendar_id, fetched_resource))
        .flatten()
        .collect::<Vec<NewTodo>>();

    insert_into(todo_dsl::todos)
        .values(todos)
        .execute(conn)
        .map(|_| ())
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
