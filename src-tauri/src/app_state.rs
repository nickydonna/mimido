use anyhow::anyhow;
use std::borrow::Cow;
use tauri::async_runtime::RwLock;

use diesel::{Connection, SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub struct AppState {
    pub syncing: RwLock<()>,
    pub db_conn_url: Cow<'static, str>,
}

impl AppState {
    pub fn new(connection_url: String) -> Result<Self, anyhow::Error> {
        log::info!("Creating AppState");
        log::info!("connecting to {connection_url}");
        let mut conn = SqliteConnection::establish(&connection_url)?;
        log::info!("connected");
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow!("{e:?}"))?;
        log::info!("migration ran correctly");
        Ok(Self {
            syncing: RwLock::new(()),
            db_conn_url: connection_url.into(),
        })
    }

    pub fn get_connection(&self) -> Result<SqliteConnection, anyhow::Error> {
        let conn = SqliteConnection::establish(&self.db_conn_url)?;
        Ok(conn)
    }
}
