use crate::{
    caldav::Caldav,
    establish_connection,
    models::{Calendar, NewServer, Server},
};
use diesel::prelude::*;
use futures::TryFutureExt;
use specta::Type;

pub(crate) mod calendar;
pub(crate) mod components;

#[derive(Debug, thiserror::Error)]
pub(crate) enum GeneralCommandError {
    #[error("Diesel error {0:?}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<GeneralCommandError> for String {
    fn from(value: GeneralCommandError) -> Self {
        value.to_string()
    }
}

impl Type for GeneralCommandError {
    fn inline(
        type_map: &mut specta::TypeCollection,
        generics: specta::Generics,
    ) -> specta::datatype::DataType {
        String::inline(type_map, generics)
    }
}

impl serde::Serialize for GeneralCommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command()]
#[specta::specta]
pub async fn create_server(
    server_url: String,
    user: String,
    password: String,
) -> Result<Server, GeneralCommandError> {
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
        .get_result(conn)?;

    let server = Caldav::new(server.clone())
        .map_err(anyhow::Error::new)
        .and_then(|c| c.test())
        .await
        .map(|_| server)?;
    Ok(server)
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn list_servers() -> Result<Vec<(Server, Vec<Calendar>)>, GeneralCommandError> {
    use crate::schema::servers::dsl as server_dsl;

    let conn = &mut establish_connection();
    let servers = server_dsl::servers.select(Server::as_select()).load(conn)?;

    let calendars = Calendar::belonging_to(&servers)
        .select(Calendar::as_select())
        .load(conn)?;
    Ok(calendars
        .grouped_by(&servers)
        .into_iter()
        .zip(servers)
        .map(|(calendars, server)| (server, calendars))
        .collect::<Vec<(Server, Vec<Calendar>)>>())
}
