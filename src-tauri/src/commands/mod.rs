use crate::{
    caldav::Caldav,
    establish_connection,
    models::{Calendar, NewServer, Server},
    util::stringify,
};
use diesel::prelude::*;
use futures::TryFutureExt;

pub(crate) mod calendar;
pub(crate) mod components;

#[tauri::command()]
#[specta::specta]
pub async fn create_server(
    server_url: String,
    user: String,
    password: String,
) -> Result<Server, String> {
    use crate::schema::servers;

    let conn = &mut establish_connection();
    let new_server = NewServer {
        server_url,
        user,
        password,
        last_sync: None,
    };

    let server = diesel::insert_into(servers::table)
        .values(&new_server)
        .returning(Server::as_returning())
        .get_result(conn)
        .map_err(|e| e.to_string())?;

    Caldav::new(server.clone())
        .and_then(|c| c.test())
        .await
        .map(|_| server)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn list_servers() -> Result<Vec<(Server, Vec<Calendar>)>, String> {
    use crate::schema::servers::dsl as server_dsl;

    let conn = &mut establish_connection();
    let servers = server_dsl::servers
        .select(Server::as_select())
        .load(conn)
        .map_err(stringify)?;

    let calendars = Calendar::belonging_to(&servers)
        .select(Calendar::as_select())
        .load(conn)
        .map_err(stringify)?;
    Ok(calendars
        .grouped_by(&servers)
        .into_iter()
        .zip(servers)
        .map(|(calendars, server)| (server, calendars))
        .collect::<Vec<(Server, Vec<Calendar>)>>())
}
