use crate::{
    caldav::Caldav,
    calendar_items::extract_event,
    establish_connection,
    models::{Calendar, Event, NewEvent, NewServer, Server},
};
use chrono::DateTime;
use diesel::{delete, insert_into, prelude::*};
use now::DateTimeNow;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_server(server_url: String, user: String, password: String) -> Server {
    use crate::schema::servers;

    let conn = &mut establish_connection();
    let new_server = NewServer {
        server_url,
        user,
        password,
        last_sync: None,
    };

    diesel::insert_into(servers::table)
        .values(&new_server)
        .returning(Server::as_returning())
        .get_result(conn)
        .expect("Error saving user")
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_servers() -> Vec<Server> {
    use crate::schema::servers::dsl::*;

    let conn = &mut establish_connection();
    servers
        .select(Server::as_select())
        .load(conn)
        .expect("To Load servers")
}

#[tauri::command(rename_all = "snake_case")]
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
pub async fn list_events_for_day(datetime: String) -> Result<Vec<Event>, String> {
    use crate::schema::events::dsl as event_dsl;

    let conn = &mut establish_connection();

    let parsed = DateTime::parse_from_rfc3339(&datetime)
        .map_err(stringify)?
        .to_utc();
    let start = parsed.beginning_of_day();
    let end = parsed.end_of_day();

    event_dsl::events
        .filter(
            event_dsl::has_rrule.eq(true).or(event_dsl::starts_at
                .ge(start)
                .and(event_dsl::ends_at.le(end))),
        )
        .select(Event::as_select())
        .load(conn)
        .map_err(stringify)
}

#[tauri::command(rename_all = "snake_case")]
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
    // let mut calendars: Vec<Calendar> = vec![];

    let calendars = found_calendars
        .into_iter()
        .flat_map(|new_cal| -> Result<Calendar, String> {
            let calendar_record = find_calendar_by_name(conn, &new_cal.name)?;
            if let Some(calendar) = calendar_record {
                println!("update {}", calendar.name);
                diesel::update(calendars_dsl::calendars)
                    .filter(calendars_dsl::id.eq(calendar.id))
                    .set(calendars_dsl::etag.eq(&new_cal.etag))
                    .returning(Calendar::as_select())
                    .get_result(conn)
                    .map_err(|e| e.to_string())
            } else {
                println!("insert {}", new_cal.name);
                diesel::insert_into(calendars_dsl::calendars)
                    .values(&new_cal)
                    .returning(Calendar::as_select())
                    .get_result(conn)
                    .map_err(|e| e.to_string())
            }
        })
        .collect::<Vec<Calendar>>();
    println!("{:#?}", calendars);
    Ok(calendars)

    // let items = caldav
    //     .get_calendar_items(
    //         "/dav/calendars/user/nickydonna@fastmail.com/a43779d6-6b5f-40e9-b3cf-2218e242bbaa/",
    //     )
    //     .await
    //     .map_err(stringify)?;
    //
    // println!("{:#?}", items);
    //
}

fn find_calendar_by_name(
    connection: &mut SqliteConnection,
    name: &str,
) -> Result<Option<Calendar>, String> {
    use crate::schema::calendars::dsl as calendars_dsl;

    calendars_dsl::calendars
        .filter(calendars_dsl::name.eq(name))
        .first::<Calendar>(connection)
        .optional()
        .map_err(|err| err.to_string())
}

fn stringify<T: ToString>(e: T) -> String {
    format!("Error code: {}", e.to_string())
}
