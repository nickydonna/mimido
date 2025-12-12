use diesel::prelude::*;
use tauri::async_runtime::spawn_blocking;

use crate::db_conn::DbConn;
use crate::models::Calendar;
use crate::models::model_traits::{ById, ListAll};
use crate::schema::servers::dsl as server_dsl;
use crate::{caldav::Caldav, establish_connection, schema::servers};

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    Debug,
    serde::Serialize,
    specta::Type,
    Clone,
    PartialEq,
)]
#[diesel(table_name = servers)]
pub struct Server {
    pub id: i32,
    pub server_url: String,
    pub user: String,
    pub password: String,
    pub last_sync: Option<i64>,
}

#[derive(Clone, Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = servers)]
pub struct NewServer {
    pub server_url: String,
    pub user: String,
    pub password: String,
    pub last_sync: Option<i64>,
}

impl NewServer {
    pub async fn save(self: NewServer) -> anyhow::Result<Server> {
        let server = spawn_blocking(|| {
            let conn = &mut establish_connection();
            let res: Result<Server, anyhow::Error> = diesel::insert_into(servers::table)
                .values(self)
                .returning(Server::as_returning())
                .get_result(conn)
                .map_err(anyhow::Error::new);
            res
        })
        .await??;

        let caldav = Caldav::new(server.clone()).await?;
        caldav.test().await?;
        Ok(server)
    }
}

impl ById for Server {
    async fn by_id(conn: DbConn, server_id: i32) -> anyhow::Result<Option<Server>> {
        use crate::schema::servers::dsl as server_dsl;
        let res = spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();

            server_dsl::servers
                .filter(server_dsl::id.eq(server_id))
                .select(Self::as_select())
                .first(conn)
                .optional()
        })
        .await??;
        Ok(res)
    }
}

impl ListAll for Server {
    async fn list_all(conn: DbConn) -> anyhow::Result<Vec<Server>> {
        use crate::schema::servers::dsl::*;
        spawn_blocking(move || {
            let conn = &mut *conn.0.lock().unwrap();
            let s = servers.select(Server::as_select()).load(conn)?;
            Ok(s)
        })
        .await?
    }
}

impl Server {
    pub async fn list_all_with_calendars(
        conn: DbConn,
    ) -> anyhow::Result<Vec<(Server, Vec<Calendar>)>> {
        let res: Result<Vec<(Server, Vec<Calendar>)>, diesel::result::Error> =
            spawn_blocking(move || {
                let conn = &mut *conn.0.lock().unwrap();

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
            })
            .await?;
        let res = res?;
        Ok(res)
    }
}
