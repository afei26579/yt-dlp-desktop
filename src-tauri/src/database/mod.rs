pub mod models;

use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use std::sync::Mutex;
use models::{DownloadTask, DownloadStatus};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let conn = Connection::open(app_dir.join("history.db"))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS download_history (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                thumbnail TEXT,
                status TEXT NOT NULL,
                output_path TEXT,
                format_id TEXT,
                quality_label TEXT NOT NULL DEFAULT '',
                audio_only INTEGER NOT NULL DEFAULT 0,
                total_size TEXT,
                error_message TEXT,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                playlist_title TEXT,
                playlist_index INTEGER,
                playlist_total INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_created_at ON download_history(created_at DESC);"
        )?;

        // 数据库迁移：添加新列（如果不存在）
        let columns = Self::get_column_names(&conn, "download_history")?;
        if !columns.contains(&"playlist_title".to_string()) {
            conn.execute("ALTER TABLE download_history ADD COLUMN playlist_title TEXT", [])?;
        }
        if !columns.contains(&"playlist_index".to_string()) {
            conn.execute("ALTER TABLE download_history ADD COLUMN playlist_index INTEGER", [])?;
        }
        if !columns.contains(&"playlist_total".to_string()) {
            conn.execute("ALTER TABLE download_history ADD COLUMN playlist_total INTEGER", [])?;
        }

        Ok(Self { conn: Mutex::new(conn) })
    }

    fn get_column_names(conn: &Connection, table: &str) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
        let names = stmt.query_map([], |row| {
            row.get::<_, String>(1)
        })?.collect::<Result<Vec<_>>>()?;
        Ok(names)
    }

    pub fn insert_task(&self, task: &DownloadTask) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO download_history
            (id, url, title, thumbnail, status, output_path, format_id, quality_label,
             audio_only, total_size, error_message, created_at, completed_at,
             playlist_title, playlist_index, playlist_total)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                task.id, task.url, task.title, task.thumbnail,
                serde_json::to_string(&task.status).unwrap(),
                task.output_path, task.format_id, task.quality_label,
                task.audio_only as i32, task.total_size, task.error_message,
                task.created_at, task.completed_at,
                task.playlist_title, task.playlist_index, task.playlist_total,
            ],
        )?;
        Ok(())
    }

    pub fn update_task_status(
        &self, id: &str, status: &str, output_path: Option<&str>,
        error_message: Option<&str>, completed_at: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE download_history SET status=?2, output_path=COALESCE(?3,output_path),
             error_message=?4, completed_at=?5 WHERE id=?1",
            params![id, status, output_path, error_message, completed_at],
        )?;
        Ok(())
    }

    pub fn get_history(&self, limit: u32, offset: u32) -> Result<Vec<DownloadTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, title, thumbnail, status, output_path, format_id,
             quality_label, audio_only, total_size, error_message, created_at,
             completed_at, playlist_title, playlist_index, playlist_total
             FROM download_history ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        )?;
        let tasks = stmt.query_map(params![limit, offset], |row| {
            let status_str: String = row.get(4)?;
            let ao: i32 = row.get(8)?;
            Ok(DownloadTask {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                thumbnail: row.get(3)?,
                status: serde_json::from_str(&status_str).unwrap_or(DownloadStatus::Failed),
                progress: 0.0,
                speed: None,
                eta: None,
                total_size: row.get(9)?,
                downloaded_size: None,
                output_path: row.get(5)?,
                format_id: row.get(6)?,
                quality_label: row.get(7)?,
                audio_only: ao != 0,
                download_subtitle: false,
                subtitle_lang: None,
                error_message: row.get(10)?,
                created_at: row.get(11)?,
                completed_at: row.get(12)?,
                playlist_title: row.get(13)?,
                playlist_index: row.get::<_, Option<i32>>(14)?.map(|v| v as u32),
                playlist_total: row.get::<_, Option<i32>>(15)?.map(|v| v as u32),
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(tasks)
    }

    pub fn delete_history(&self, id: &str) -> Result<()> {
        self.conn.lock().unwrap().execute(
            "DELETE FROM download_history WHERE id=?1", params![id],
        )?;
        Ok(())
    }

    pub fn clear_history(&self) -> Result<()> {
        self.conn.lock().unwrap().execute("DELETE FROM download_history", [])?;
        Ok(())
    }

    pub fn get_pending_count(&self) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM download_history WHERE status IN ('\"Pending\"', '\"Queued\"', '\"Downloading\"', '\"Merging\"')",
            [], |row| row.get(0),
        )?;
        Ok(count)
    }
}