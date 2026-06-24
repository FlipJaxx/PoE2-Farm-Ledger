use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Manager;

#[derive(Default)]
pub struct AppState {
    db_path: Mutex<Option<PathBuf>>,
}

impl AppState {
    pub fn initialize(&self, app: &tauri::AppHandle) -> Result<PathBuf, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|err| format!("Could not resolve app data directory: {err}"))?;
        std::fs::create_dir_all(&dir)
            .map_err(|err| format!("Could not create app data directory: {err}"))?;
        let path = dir.join("exile-farm-ledger.sqlite");
        let conn = Connection::open(&path).map_err(|err| err.to_string())?;
        crate::schema::migrate_and_seed(&conn).map_err(|err| err.to_string())?;
        *self.db_path.lock().map_err(|err| err.to_string())? = Some(path.clone());
        Ok(path)
    }

    pub fn connection(&self) -> Result<Connection, String> {
        let path = self
            .db_path
            .lock()
            .map_err(|err| err.to_string())?
            .clone()
            .ok_or_else(|| "Database has not been initialized".to_string())?;
        let conn = Connection::open(path).map_err(|err| err.to_string())?;
        conn.busy_timeout(Duration::from_secs(5))
            .map_err(|err| err.to_string())?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
            .map_err(|err| err.to_string())?;
        Ok(conn)
    }
}
