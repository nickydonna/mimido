use anyhow::anyhow;
use tauri::async_runtime::RwLock;

use diesel::{Connection, SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub struct AppState {
    pub syncing: RwLock<()>,
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
        })
    }
}
