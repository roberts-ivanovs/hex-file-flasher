use rusqlite::{named_params, params, Connection, Result};
use std::time;

pub struct DbInstance {
    conn: Connection,
}

impl DbInstance {
    pub fn new() -> Self {
        let conn = Connection::open("db.sqlite3").unwrap();
        let inst = Self { conn };
        inst.init_tables();
        inst
    }
    fn init_tables(&self) {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS chip (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chip_type TEXT NOT NULL,
                chip_number TEXT NULL UNIQUE
            );",
                params![],
            )
            .unwrap();
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS flash (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chip_fk INTEGER NOT NULL,
                software TEXT NOT NULL,
                flashed_id TEXT NULL,
                flashed_time INTEGER NULL,
                FOREIGN KEY(chip_fk) REFERENCES chip(id)
            );",
                params![],
            )
            .unwrap();
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS test (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                flash_fk INTEGER NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY(flash_fk) REFERENCES flash(id)
            );",
                params![],
            )
            .unwrap();
    }

    pub fn register_chip(&self, chip_type: &str, chip_number: Option<&str>) -> i64 {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM chip WHERE chip_number = :chip_number")
            .unwrap();
        let mut rows = stmt
            .query_named(named_params! { ":chip_number": chip_number })
            .unwrap();

        let mut max_id: i64 = -1;
        let mut max_count = 0;
        while let Some(row) = rows.next().unwrap() {
            let id: i64 = row.get(0).unwrap();
            println!("{}", id);
            max_id = id;
            max_count += 1;
        }

        match max_count {
            1 => {
                let mut stmt = self
                    .conn
                    .prepare(r#"UPDATE chip SET chip_type=? WHERE id=?;"#)
                    .unwrap();
                stmt.execute(params![chip_type, max_id]).unwrap();
                max_id
            }
            _ => {
                // Perform new object insertion
                let mut stmt = self
                    .conn
                    .prepare(
                        "INSERT INTO chip (chip_type, chip_number)
                    VALUES (?1, ?2)
                ",
                    )
                    .unwrap();
                let id = stmt.insert(params![chip_type, chip_number]).unwrap();
                id
            }
        }
    }

    pub fn register_flash(
        &self,
        chip_db_id: i64,
        software: &str,
        flashed_time: time::SystemTime,
        flashed_id: Option<&str>,
    ) -> i64 {
        let timestamp = flashed_time
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let mut stmt = self
            .conn
            .prepare(
                "INSERT INTO flash (chip_fk, software, flashed_id, flashed_time)
            VALUES (?1, ?2, ?3, ?4)",
            )
            .unwrap();
        let id = stmt
            .insert(params![chip_db_id as i64, software, flashed_id, timestamp])
            .unwrap();
        id
    }

    pub fn register_test(&self, flash_db_id: i64, key: &str, value: &str) -> i64 {
        let conn = Connection::open("db.sqlite3").unwrap();
        let mut stmt = conn
            .prepare(
                "INSERT INTO test (flash_fk, key, value)
            VALUES (?1, ?2, ?3)
        ",
            )
            .unwrap();
        let id = stmt
            .insert(params![flash_db_id as f64, key, value])
            .unwrap();
        id
    }

}
