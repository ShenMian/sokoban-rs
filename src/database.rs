use nalgebra::Vector2;
use rusqlite::Connection;
use siphasher::sip::SipHasher24;

use crate::level::Level;
use crate::movement::Movements;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub struct Database {
    connection: Connection,
}

impl Database {
    /// Creates a new Database instance with a connection to a file-based database.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            connection: Connection::open(path).expect("failed to open database"),
        }
    }

    /// Creates a new Database instance with an in-memory connection.
    #[allow(dead_code)]
    pub fn from_memory() -> Self {
        Self {
            connection: Connection::open_in_memory().expect("failed to open database"),
        }
    }

    /// Initializes the database by creating the necessary tables.
    pub fn initialize(&self) {
        const CREATE_LEVEL_TABLE: &str = "
            CREATE TABLE IF NOT EXISTS tb_level (
                id       INTEGER PRIMARY KEY AUTOINCREMENT,
                title    TEXT,
                author   TEXT,
                comments TEXT,
                map      TEXT NOT NULL,
                width    INTEGER NOT NULL,
                height   INTEGER NOT NULL,
                hash     INTEGER NOT NULL UNIQUE,
                date     DATE NOT NULL
            )
        ";
        const CREATE_LEVEL_INDICES: &str =
            "CREATE UNIQUE INDEX IF NOT EXISTS ux_level_hash ON tb_level(hash)";
        const CREATE_SNAPSHOT_TABLE: &str = "
            CREATE TABLE IF NOT EXISTS tb_snapshot (
                level_id  INTEGER,
                movements TEXT,
                datetime  DATETIME NOT NULL,
                best_move BOOLEAN NOT NULL DEFAULT 0 CHECK (best_move IN (0, 1)),
                best_push BOOLEAN NOT NULL DEFAULT 0 CHECK (best_push IN (0, 1)),
                PRIMARY KEY (level_id, best_move, best_push),
                FOREIGN KEY (level_id) REFERENCES tb_level(id)
            )
        ";

        self.connection.execute(CREATE_LEVEL_TABLE, ()).unwrap();
        self.connection.execute(CREATE_LEVEL_INDICES, ()).unwrap();
        self.connection.execute(CREATE_SNAPSHOT_TABLE, ()).unwrap();
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
        let comments = level.metadata.get("comments");
        let hash = Database::normalized_hash(level);

        let _ = self.connection.execute(
            "INSERT INTO tb_level(title, author, comments, map, width, height, hash, date) VALUES (?, ?, ?, ?, ?, ?, ?, DATE('now'))",
            (title, author, comments, level.export_map(), level.dimensions().x, level.dimensions().y, hash),
        );
    }

    /// Returns the level ID by the provided level.
    pub fn get_level_id(&self, level: &Level) -> Option<u64> {
        let hash = Database::normalized_hash(level);
        match self
            .connection
            .query_row("SELECT id FROM tb_level WHERE hash = ?", [hash], |row| {
                row.get(0)
            }) {
            Ok(level_id) => level_id,
            Err(_) => None,
        }
    }

    /// Returns a level based by ID.
    pub fn get_level_by_id(&self, id: u64) -> Option<Level> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT map, width, height, title, author, comments FROM tb_level WHERE id = ?",
            )
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
        if let Ok(comments) = row.get(5) {
            metadata.insert("comments".to_string(), comments);
        }
        let level = Level::new(map, size, metadata).unwrap();
        Some(level)
    }

    /// Returns the ID of the next unsolved level after the provided ID.
    pub fn next_unsolved_level_id(&self, id: u64) -> Option<u64> {
        let mut statement = self.connection.prepare(
            "SELECT id FROM tb_level WHERE id > ? AND id NOT IN (SELECT DISTINCT level_id FROM tb_snapshot) ORDER BY id ASC LIMIT 1",
        ).unwrap();
        let mut rows = statement.query([id]).unwrap();

        let row = rows.next().unwrap()?;
        Some(row.get(0).unwrap())
    }

    /// Returns the ID of the previous unsolved level before the provided ID.
    pub fn previous_unsolved_level_id(&self, id: u64) -> Option<u64> {
        let mut statement = self.connection.prepare(
            "SELECT id FROM tb_level WHERE id < ? AND id NOT IN (SELECT DISTINCT level_id FROM tb_snapshot) ORDER BY id ASC LIMIT 1",
        ).unwrap();
        let mut rows = statement.query([id]).unwrap();

        let row = rows.next().unwrap()?;
        Some(row.get(0).unwrap())
    }

    pub fn best_move_solution(&self, level_id: u64) -> Option<Movements> {
        let mut statement = self
            .connection
            .prepare("SELECT movements FROM tb_snapshot WHERE level_id = ? AND best_move = 1")
            .unwrap();
        let mut rows = statement.query([level_id]).unwrap();
        let row = rows.next().unwrap()?;
        let best_move: String = row.get(0).unwrap();
        Some(Movements::from_lurd(&best_move))
    }

    pub fn best_push_solution(&self, level_id: u64) -> Option<Movements> {
        let mut statement = self
            .connection
            .prepare("SELECT movements FROM tb_snapshot WHERE level_id = ? AND best_push = 1")
            .unwrap();
        let mut rows = statement.query([level_id]).unwrap();
        let row = rows.next().unwrap()?;
        let best_push: String = row.get(0).unwrap();
        Some(Movements::from_lurd(&best_push))
    }

    pub fn update_solution(&self, level_id: u64, solution: &Movements) {
        let lurd: String = solution.lurd();

        if let Some(best_move_solution) = self.best_move_solution(level_id) {
            dbg!();
            if solution.move_count() < best_move_solution.move_count() {
                self.connection
                    .execute(
                        "UPDATE tb_snapshot SET movements = ? WHERE level_id = ?",
                        (lurd.clone(), level_id),
                    )
                    .unwrap();
            }
        } else {
            self.connection
                .execute(
                    "INSERT INTO tb_snapshot (level_id, movements, best_move, datetime) VALUES (?, ?, 1, DATE('now'))",
                    (level_id, lurd.clone()),
                )
                .unwrap();
        }

        if let Some(best_push_solution) = self.best_push_solution(level_id) {
            dbg!();
            if solution.push_count() < best_push_solution.push_count() {
                self.connection
                    .execute(
                        "UPDATE tb_snapshot SET movements = ? WHERE level_id = ?",
                        (lurd.clone(), level_id),
                    )
                    .unwrap();
            }
        } else {
            self.connection
                .execute(
                    "INSERT INTO tb_snapshot (level_id, movements, best_push, datetime) VALUES (?, ?, 1, DATE('now'))",
                    (level_id, lurd.clone()),
                )
                .unwrap();
        }
    }

    /// Returns the maximum level ID.
    pub fn max_level_id(&self) -> Option<u64> {
        self.connection
            .query_row("SELECT MAX(id) FROM tb_level", [], |row| row.get(0))
            .unwrap()
    }

    /// Returns the minimum level ID.
    pub fn min_level_id(&self) -> Option<u64> {
        self.connection
            .query_row("SELECT MIN(id) FROM tb_level", [], |row| row.get(0))
            .unwrap()
    }

    /// Computes a normalized hash for the provided level.
    fn normalized_hash(level: &Level) -> String {
        let mut hasher = SipHasher24::new();
        let mut normalized_level = level.clone();
        normalized_level.normalize();
        normalized_level.hash(&mut hasher);
        let hash = hasher.finish();
        // Must convert the hash to a string first, otherwise rusqlite may throw an error.
        hash.to_string()
    }
}
