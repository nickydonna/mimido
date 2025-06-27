use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use lazy_static::lazy_static;
use specta_typescript::{BigIntExportBehavior, Typescript};
use std::error::Error;
use std::fs::create_dir_all;
use std::sync::Mutex;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};
pub mod caldav;
pub mod calendar_items;
mod commands;
pub mod models;
pub mod schema;
pub(crate) mod util;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            commands::create_server,
            commands::list_servers,
            commands::calendar::list_calendars,
            commands::calendar::fetch_calendars,
            commands::calendar::sync_calendar,
            commands::calendar::sync_all_calendars,
            commands::components::list_events_for_day,
            commands::components::parse_event,
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            Typescript::new().bigint(BigIntExportBehavior::String),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
