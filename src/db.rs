use rusqlite::{params, Connection, Result};
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

    pub fn register_chip(&self, chip_type: &str, chip_number: Option<&str>) -> usize {
        // TODO search if chip number exists in the DB already, if so, don't create new entry
        let id = self
            .conn
            .execute(
                "INSERT INTO chip (chip_type, chip_number)
            VALUES (?1, ?2)
        ",
                params![chip_type, chip_number],
            )
            .unwrap();
        id
    }

    pub fn register_flash(
        &self,
        chip_db_id: usize,
        software: &str,
        flashed_time: time::SystemTime,
        flashed_id: Option<&str>,
    ) -> usize {
        let timestamp = flashed_time
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let id = self
            .conn
            .execute(
                "INSERT INTO flash (chip_fk, software, flashed_id, flashed_time)
            VALUES (?1, ?2, ?3, ?4)
        ",
                params![chip_db_id as f64, software, flashed_id, timestamp],
            )
            .unwrap();
        id
    }

    pub fn register_test(&self, flash_db_id: usize, key: &str, value: &str) -> usize {
        let conn = Connection::open("db.sqlite3").unwrap();
        let id = conn
            .execute(
                "INSERT INTO test (flash_fk, key, value)
            VALUES (?1, ?2, ?3)
        ",
                params![flash_db_id as f64, key, value],
            )
            .unwrap();
        id
    }
}
