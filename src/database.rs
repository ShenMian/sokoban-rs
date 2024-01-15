use nalgebra::Vector2;
use rusqlite::Connection;
use siphasher::sip::SipHasher24;

use crate::level::Level;
use crate::movement::Movement;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub struct Database {
    connection: Connection,
}

impl Database {
    #[allow(dead_code)]
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
                level_id           INTEGER PRIMARY KEY,
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
        let hash = Database::normalized_hash(level);

        let _ = self.connection.execute(
            "INSERT INTO tb_level(title, author, map, width, height, hash, date) VALUES (?, ?, ?, ?, ?, ?, DATE('now'))",
            (&title, &author, &level.export_map(), level.dimensions.x, level.dimensions.y, hash.to_string()),
        );
    }

    pub fn get_level_id(&self, level: &Level) -> Option<u64> {
        let hash = Database::normalized_hash(level);
        match self.connection.query_row(
            "SELECT id FROM tb_level WHERE hash = ?",
            [hash.to_string()],
            |row| row.get(0),
        ) {
            Ok(level_id) => level_id,
            Err(_) => None,
        }
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

    pub fn update_solution(&self, level_id: u64, solution: &[Movement]) {
        let move_count = solution.len();
        let push_count = solution.iter().filter(|x| x.is_push).count();
        let lurd: String = solution
            .iter()
            .map(|x| Into::<char>::into(x.clone()))
            .collect();

        if let Some(best_move_count) = self.get_best_move_count(level_id) {
            if move_count < best_move_count {
                self.connection
                    .execute(
                        "UPDATE tb_solution SET best_move_solution = ? WHERE level_id = ?",
                        (lurd.clone(), level_id),
                    )
                    .unwrap();
            }
        }

        if let Some(best_push_count) = self.get_best_push_count(level_id) {
            if push_count < best_push_count {
                self.connection
                    .execute(
                        "UPDATE tb_solution SET best_push_solution = ? WHERE level_id = ?",
                        (lurd.clone(), level_id),
                    )
                    .unwrap();
            }
        }

        let _ = self.connection.execute(
            "INSERT INTO tb_solution (level_id, best_move_solution, best_push_solution) VALUES (?, ?, ?)",
            (level_id, lurd.clone(), lurd.clone()),
        );
    }

    pub fn get_best_move_count(&self, level_id: u64) -> Option<usize> {
        Some(self.get_best_move_solution(level_id)?.len())
    }

    pub fn get_best_push_count(&self, level_id: u64) -> Option<usize> {
        Some(
            self.get_best_push_solution(level_id)?
                .chars()
                .filter(|x| x.is_ascii_uppercase())
                .count(),
        )
    }

    pub fn get_best_move_solution(&self, level_id: u64) -> Option<String> {
        let mut statement = self
            .connection
            .prepare("SELECT best_move_solution FROM tb_solution WHERE level_id = ?")
            .unwrap();
        let mut rows = statement.query([level_id]).unwrap();
        let row = rows.next().unwrap()?;
        let best_move: String = row.get(0).unwrap();
        Some(best_move)
    }

    pub fn get_best_push_solution(&self, level_id: u64) -> Option<String> {
        let mut statement = self
            .connection
            .prepare("SELECT best_push_solution FROM tb_solution WHERE level_id = ?")
            .unwrap();
        let mut rows = statement.query([level_id]).unwrap();
        let row = rows.next().unwrap()?;
        let best_push: String = row.get(0).unwrap();
        Some(best_push)
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

    fn normalized_hash(level: &Level) -> String {
        let mut hasher = SipHasher24::new();
        let mut normalized_level = level.clone();
        normalized_level.normalize();
        normalized_level.hash(&mut hasher);
        let hash = hasher.finish();
        // 必须先将 hash 转为字符串, 否则 rusqlite 可能报错
        hash.to_string()
    }
}
