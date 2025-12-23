use anyhow::anyhow;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use lazy_static::lazy_static;
use log::info;
use specta_typescript::{BigIntExportBehavior, Typescript};
use std::error::Error;
use std::fs::create_dir_all;
use std::sync::Mutex;
use tauri::{Listener, Manager, async_runtime, tray::TrayIconBuilder};
use tauri_specta::{Builder, collect_commands};

use crate::{app_state::AppState, commands::calendar::internal_super_sync_calendar};
pub mod app_state;
pub mod caldav;
pub mod calendar_items;
mod commands;
pub mod db_conn;
pub mod models;
pub mod schema;
pub(crate) mod util;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

lazy_static! {
    static ref CONNECTION_URL: Mutex<String> = Mutex::new(String::new());
}

pub fn establish_connection() -> SqliteConnection {
    let connection_url = CONNECTION_URL.lock().unwrap();
    SqliteConnection::establish(&connection_url).unwrap_or_else(|_| {
        log::error!("Error connecting to {}", *connection_url);
        panic!("Error connecting to {}", *connection_url)
    })
}

pub fn setup_db(connection_url: &str) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    log::info!("setup_db");
    let mut locked_url = CONNECTION_URL.lock().unwrap();
    *locked_url = connection_url.to_owned();
    drop(locked_url); // Release the lock before establishing the connection
    log::info!("connecting to {connection_url}");
    let mut conn = establish_connection();
    log::info!("connected");
    conn.run_pending_migrations(MIGRATIONS)?;
    log::info!("migration ran correctly");
    Ok(())
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncEventPayload {
    calendar_id: i32,
}

impl SyncEventPayload {
    pub fn new(calendar_id: i32) -> Self {
        Self { calendar_id }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            commands::create_server,
            commands::list_servers,
            commands::calendar::list_calendars,
            commands::calendar::fetch_calendars_from_caldav,
            // commands::calendar::sync_calendar,
            // commands::calendar::sync_all_calendars,
            commands::calendar::set_default_calendar,
            commands::calendar::super_sync_calendar,
            commands::components::list_events_for_day,
            commands::components::list_todos_for_day,
            commands::components::parse_event,
            commands::components::create_component,
            commands::components::set_vevent_status,
            commands::components::delete_vcmp,
            commands::components::update_vcmp,
            commands::components::list_unscheduled_todos,
            commands::components::set_vtodo_status,
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            Typescript::new().bigint(BigIntExportBehavior::String),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            let _ = TrayIconBuilder::new().build(app)?;
            let app_path = app.path().app_config_dir().expect("No App path was found!");
            let db_file_name = "mimido.db";
            let conn_url = format!("sqlite://{}/{}", app_path.display(), db_file_name);

            if let Err(e) = create_dir_all(&app_path) {
                println!("Problem creating app directory: {e}");
            }

            println!("Connection URL: {conn_url}");
            if let Err(e) = setup_db(&conn_url) {
                println!("Database setup failed: {e}");
            }

            let Ok(state) = AppState::new(conn_url) else {
                return Err(anyhow!("Could not connect").into_boxed_dyn_error());
            };

            app.manage(state);
            let handle = app.handle().clone();
            app.listen("sync", move |event| {
                let handle = handle.clone();
                async_runtime::spawn(async move {
                    if let Ok(payload) = serde_json::from_str::<SyncEventPayload>(event.payload()) {
                        let state = handle.state::<AppState>();
                        let lock = state.syncing.write().await;
                        let res = internal_super_sync_calendar(payload.calendar_id).await;
                        drop(lock);
                        info!("Sync resulted in {res:?}");
                    }
                });
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
