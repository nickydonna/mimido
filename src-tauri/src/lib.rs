use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use lazy_static::lazy_static;
use std::error::Error;
use std::fs::create_dir_all;
use std::sync::Mutex;
use tauri::Manager;
pub mod caldav;
pub mod calendar_items;
mod commands;
pub mod models;
pub mod schema;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

lazy_static! {
    static ref CONNECTION_URL: Mutex<String> = Mutex::new(String::new());
}

pub fn establish_connection() -> SqliteConnection {
    let connection_url = CONNECTION_URL.lock().unwrap();
    SqliteConnection::establish(&connection_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", *connection_url))
}

pub fn setup_db(connection_url: &str) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    println!("running migrations");
    let mut locked_url = CONNECTION_URL.lock().unwrap();
    *locked_url = connection_url.to_owned();
    drop(locked_url); // Release the lock before establishing the connection
    let mut conn = establish_connection();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::create_server,
            commands::list_servers,
            commands::list_calendars,
            commands::fetch_calendars,
            commands::sync_calendar,
            commands::list_events_for_day,
        ])
        .setup(|app| {
            let app_path = app.path().app_config_dir().expect("No App path was found!");
            let db_file_name = "mimido.db";
            let conn_url = format!("sqlite://{}/{}", app_path.display(), db_file_name);

            if let Err(e) = create_dir_all(&app_path) {
                println!("Problem creating app directory: {:?}", e);
            }

            println!("Connection URL: {}", conn_url);
            if let Err(e) = setup_db(&conn_url) {
                println!("Database setup failed: {:?}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
