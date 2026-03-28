use std::{path::PathBuf, sync::Arc};

use rusqlite::{Connection, OptionalExtension, params};

use crate::model::{NewPhoto, PhotoRecord};

#[derive(Clone)]
pub struct Database {
    path: Arc<PathBuf>,
}

impl Database {
    pub async fn new(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = Arc::new(path.into());
        let db = Self { path };
        db.init().await?;
        Ok(db)
    }

    pub async fn list_photos(&self) -> Result<Vec<PhotoRecord>, String> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = open_connection(&path)?;
            let mut stmt = conn
                .prepare(
                    "
                    SELECT id, title, description, tags, filename, mime_type, image_data, created_at
                    FROM photos
                    ORDER BY id DESC
                    ",
                )
                .map_err(|err| err.to_string())?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(PhotoRecord {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        tags: split_tags(row.get::<_, String>(3)?),
                        filename: row.get(4)?,
                        mime_type: row.get(5)?,
                        data: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                })
                .map_err(|err| err.to_string())?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|err| err.to_string())
        })
        .await
        .map_err(|err| err.to_string())?
    }

    pub async fn insert_photo(&self, new_photo: NewPhoto) -> Result<PhotoRecord, String> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = open_connection(&path)?;
            conn.execute(
                "
                INSERT INTO photos (title, description, tags, filename, mime_type, image_data)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ",
                params![
                    new_photo.title,
                    new_photo.description,
                    join_tags(&new_photo.tags),
                    new_photo.filename,
                    new_photo.mime_type,
                    new_photo.data
                ],
            )
            .map_err(|err| err.to_string())?;

            let id = conn.last_insert_rowid();
            read_photo(&conn, id)?.ok_or_else(|| "photo inserted but not found".to_string())
        })
        .await
        .map_err(|err| err.to_string())?
    }

    pub async fn get_photo(&self, id: i64) -> Result<Option<PhotoRecord>, String> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = open_connection(&path)?;
            read_photo(&conn, id)
        })
        .await
        .map_err(|err| err.to_string())?
    }

    pub async fn delete_photo(&self, id: i64) -> Result<bool, String> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = open_connection(&path)?;
            let affected = conn
                .execute("DELETE FROM photos WHERE id = ?1", [id])
                .map_err(|err| err.to_string())?;
            Ok(affected > 0)
        })
        .await
        .map_err(|err| err.to_string())?
    }

    pub async fn update_photo(
        &self,
        id: i64,
        title: String,
        description: String,
        tags: Vec<String>,
    ) -> Result<Option<PhotoRecord>, String> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = open_connection(&path)?;
            let affected = conn
                .execute(
                    "
                    UPDATE photos
                    SET title = ?1, description = ?2, tags = ?3
                    WHERE id = ?4
                    ",
                    params![title, description, join_tags(&tags), id],
                )
                .map_err(|err| err.to_string())?;

            if affected == 0 {
                return Ok(None);
            }

            read_photo(&conn, id)
        })
        .await
        .map_err(|err| err.to_string())?
    }

    async fn init(&self) -> Result<(), String> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = open_connection(&path)?;
            conn.execute_batch(
                "
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS photos (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    description TEXT NOT NULL DEFAULT '',
                    tags TEXT NOT NULL DEFAULT '',
                    filename TEXT,
                    mime_type TEXT NOT NULL,
                    image_data BLOB NOT NULL,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS app_settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
                ",
            )
            .map_err(|err| err.to_string())?;

            ensure_tags_column(&conn)
        })
        .await
        .map_err(|err| err.to_string())?
    }

    pub async fn get_admin_password(&self) -> Result<Option<String>, String> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = open_connection(&path)?;
            conn.query_row(
                "SELECT value FROM app_settings WHERE key = 'admin_password'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| err.to_string())
        })
        .await
        .map_err(|err| err.to_string())?
    }

    pub async fn set_admin_password_if_unset(&self, password: String) -> Result<bool, String> {
        let path = self.path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = open_connection(&path)?;
            let affected = conn
                .execute(
                    "
                    INSERT INTO app_settings (key, value)
                    VALUES ('admin_password', ?1)
                    ON CONFLICT(key) DO NOTHING
                    ",
                    [password],
                )
                .map_err(|err| err.to_string())?;

            Ok(affected > 0)
        })
        .await
        .map_err(|err| err.to_string())?
    }
}

fn open_connection(path: &PathBuf) -> Result<Connection, String> {
    Connection::open(path).map_err(|err| err.to_string())
}

fn read_photo(conn: &Connection, id: i64) -> Result<Option<PhotoRecord>, String> {
    conn.query_row(
        "
        SELECT id, title, description, tags, filename, mime_type, image_data, created_at
        FROM photos
        WHERE id = ?1
        ",
        [id],
        |row| {
            Ok(PhotoRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                tags: split_tags(row.get::<_, String>(3)?),
                filename: row.get(4)?,
                mime_type: row.get(5)?,
                data: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )
    .optional()
    .map_err(|err| err.to_string())
}

fn split_tags(raw: String) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn join_tags(tags: &[String]) -> String {
    tags.iter()
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

fn ensure_tags_column(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(photos)")
        .map_err(|err| err.to_string())?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|err| err.to_string())?;

    let has_tags = columns
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?
        .iter()
        .any(|column| column == "tags");

    if has_tags {
        return Ok(());
    }

    conn.execute(
        "ALTER TABLE photos ADD COLUMN tags TEXT NOT NULL DEFAULT ''",
        [],
    )
    .map_err(|err| err.to_string())?;

    Ok(())
}
