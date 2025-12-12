use std::sync::{Arc, Mutex};

use diesel::SqliteConnection;
use tauri::async_runtime::spawn_blocking;

use crate::establish_connection;

#[derive(Clone)]
pub struct DbConn(pub Arc<Mutex<SqliteConnection>>);

impl DbConn {
    pub async fn new() -> anyhow::Result<Self> {
        let res = spawn_blocking(|| {
            let conn = establish_connection();
            Self(Arc::new(Mutex::new(conn)))
        })
        .await?;
        Ok(res)
    }
}
