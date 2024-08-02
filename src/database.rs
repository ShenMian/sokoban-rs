use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
    str::FromStr,
};

use rusqlite::Connection;
use soukoban::{Actions, Level};

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
        let title = level.metadata().get("title");
        let author = level.metadata().get("author");
        let comments = level.metadata().get("comments");
        let hash = Database::normalized_hash(level);

        let _ = self.connection.execute(
            "INSERT INTO tb_level(title, author, comments, map, width, height, hash, date) VALUES (?, ?, ?, ?, ?, ?, ?, DATE('now'))",
            (title, author, comments, level.map().to_string(), level.map().dimensions().x, level.map().dimensions().y, hash),
        );
    }

    /// Returns the level ID by the provided level.
    pub fn get_level_id(&self, level: &Level) -> Option<u64> {
        let hash = Database::normalized_hash(level);
        self.connection
            .query_row("SELECT id FROM tb_level WHERE hash = ?", [hash], |row| {
                row.get(0)
            })
            .ok()
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

        let map = row.get::<_, String>(0).unwrap();
        let mut metadata = String::new();
        if let Ok(title) = row.get::<_, String>(3) {
            metadata.push_str(&format!("title: {title}\n"));
        }
        if let Ok(author) = row.get::<_, String>(4) {
            metadata.push_str(&format!("author: {author}\n"));
        }
        if let Ok(comments) = row.get::<_, String>(5) {
            metadata.push_str(&format!("comments: {comments}\n"));
        }
        let level = Level::from_str(&(map + &metadata)).unwrap();
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

    pub fn best_move_solution(&self, level_id: u64) -> Option<Actions> {
        let mut statement = self
            .connection
            .prepare("SELECT movements FROM tb_snapshot WHERE level_id = ? AND best_move = 1")
            .unwrap();
        let mut rows = statement.query([level_id]).unwrap();
        let row = rows.next().unwrap()?;
        let best_move: String = row.get(0).unwrap();
        Some(Actions::from_str(&best_move).unwrap())
    }

    pub fn best_push_solution(&self, level_id: u64) -> Option<Actions> {
        let mut statement = self
            .connection
            .prepare("SELECT movements FROM tb_snapshot WHERE level_id = ? AND best_push = 1")
            .unwrap();
        let mut rows = statement.query([level_id]).unwrap();
        let row = rows.next().unwrap()?;
        let best_push: String = row.get(0).unwrap();
        Some(Actions::from_str(&best_push).unwrap())
    }

    pub fn update_solution(&self, level_id: u64, solution: &Actions) {
        let lurd = solution.to_string();

        if let Some(best_move_solution) = self.best_move_solution(level_id) {
            dbg!();
            if solution.moves() < best_move_solution.moves() {
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
            if solution.pushes() < best_push_solution.pushes() {
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
        let mut hasher = DefaultHasher::new();
        let mut normalized_level = level.clone();
        normalized_level.map_mut().normalize();
        normalized_level.map_mut().hash(&mut hasher);
        let hash = hasher.finish();
        // Must convert the hash to a string first, otherwise rusqlite may throw an error.
        hash.to_string()
    }
}
