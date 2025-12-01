use crate::{
    commands::errors::CommandError,
    models::{
        Calendar,
        server::{NewServer, Server},
    },
};

pub(crate) mod calendar;
pub(crate) mod components;
pub mod errors;
mod extended_event;
mod extended_todo;

#[tauri::command()]
#[specta::specta]
pub async fn create_server(
    server_url: String,
    user: String,
    password: String,
) -> Result<Server, CommandError> {
    let new_server = NewServer {
        server_url,
        user,
        password,
        last_sync: None,
    };

    let server = new_server.persist().await?;
    Ok(server)
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub async fn list_servers() -> Result<Vec<(Server, Vec<Calendar>)>, CommandError> {
    let res = Server::list_all_with_calendars().await?;
    Ok(res)
}
