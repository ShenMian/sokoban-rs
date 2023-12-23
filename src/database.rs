use crate::level::Level;
use nalgebra::Vector2;
use rusqlite::Connection;
use siphasher::sip::SipHasher24;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn from_memory() -> Self {
        Self {
            connection: Connection::open_in_memory().expect("failed to open database"),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            connection: Connection::open(path).expect("failed to open database"),
        }
    }

    /// Initializes the database by creating the necessary tables.
    pub fn initialize(&self) {
        const CREATE_LEVEL_TABLE: &str = "
            CREATE TABLE IF NOT EXISTS tb_level (
                id       INTEGER PRIMARY KEY AUTOINCREMENT,
                title    TEXT,
                author   TEXT,
                map      TEXT NOT NULL,
                width    INTEGER NOT NULL,
                height   INTEGER NOT NULL,
                hash     INTEGER NOT NULL UNIQUE,
                date     DATE NOT NULL
            )
        ";
        const CREATE_SOLUTION_TABLE: &str = "
            CREATE TABLE IF NOT EXISTS tb_solution (
                level_id INTEGER PRIMARY KEY,
                best_move_solution TEXT,
                best_push_solution TEXT
            )
        ";
        const CREATE_SESSION_TABLE: &str = "
            CREATE TABLE IF NOT EXISTS tb_session (
                level_id INTEGER PRIMARY KEY,
                movement TEXT,
                datetime DATETIME NOT NULL,
                FOREIGN KEY (level_id) REFERENCES tb_level(id)
            )
        ";

        self.connection.execute(CREATE_LEVEL_TABLE, ()).unwrap();
        self.connection.execute(CREATE_SOLUTION_TABLE, ()).unwrap();
        self.connection.execute(CREATE_SESSION_TABLE, ()).unwrap();
    }

    /// Imports multiple levels into the database.
    pub fn import_levels(&self, levels: &[Level]) {
        self.connection.execute("BEGIN TRANSACTION", []).unwrap();
        for level in levels {
            self.import_level(level);
        }
        self.connection.execute("COMMIT", []).unwrap();
    }

    /// Imports a single level into the database.
    pub fn import_level(&self, level: &Level) {
        let title = level.metadata.get("title");
        let author = level.metadata.get("author");

        let mut hasher = SipHasher24::new();
        let mut normalized_level = level.clone();
        normalized_level.normalize();
        normalized_level.hash(&mut hasher);
        let hash = hasher.finish();

        let _ = self.connection.execute(
            "INSERT INTO tb_level(title, author, map, width, height, hash, date) VALUES (?, ?, ?, ?, ?, ?, DATE('now'))",
            (&title, &author, &level.export_map(), level.size.x, level.size.y, hash.to_string()),
        );
    }

    pub fn get_level_id(&self, level: &Level) -> u64 {
        let mut hasher = SipHasher24::new();
        let mut normalized_level = level.clone();
        normalized_level.normalize();
        normalized_level.hash(&mut hasher);
        let hash = hasher.finish();

        self.connection
            .query_row(
                "SELECT id FROM tb_level WHERE hash = ?",
                [hash.to_string()],
                |row| row.get(0),
            )
            .unwrap()
    }

    pub fn get_level_by_id(&self, id: u64) -> Option<Level> {
        let mut statement = self
            .connection
            .prepare("SELECT map, width, height, title, author FROM tb_level WHERE id = ?")
            .unwrap();
        let mut rows = statement.query([id]).unwrap();
        let row = rows.next().unwrap()?;

        let map = row
            .get::<_, String>(0)
            .unwrap()
            .split('\n')
            .map(|x| x.to_string())
            .collect();
        let size = Vector2::new(row.get(1).unwrap(), row.get(2).unwrap());
        let mut metadata = HashMap::new();
        if let Ok(title) = row.get(3) {
            metadata.insert("title".to_string(), title);
        }
        if let Ok(author) = row.get(4) {
            metadata.insert("author".to_string(), author);
        }
        let level = Level::new(map, size, metadata).unwrap();
        Some(level)
    }

    /// Retrieves the maximum level ID.
    pub fn max_level_id(&self) -> Option<u64> {
        self.connection
            .query_row("SELECT MAX(id) FROM tb_level", [], |row| row.get(0))
            .unwrap()
    }

    /// Retrieves the minimum level ID.
    pub fn min_level_id(&self) -> Option<u64> {
        self.connection
            .query_row("SELECT MIN(id) FROM tb_level", [], |row| row.get(0))
            .unwrap()
    }
}
