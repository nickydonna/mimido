use crate::{
    establish_connection,
    models::{NewServer, Server},
};
use diesel::prelude::*;

pub(crate) mod calendar;
pub(crate) mod components;

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
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
#[specta::specta]
pub async fn list_servers() -> Vec<Server> {
    use crate::schema::servers::dsl::*;

    let conn = &mut establish_connection();
    servers
        .select(Server::as_select())
        .load(conn)
        .expect("To Load servers")
}
