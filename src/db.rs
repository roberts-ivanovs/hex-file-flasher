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
                software TEXT NOT NULL,
                chip_number TEXT NULL,
                flashed_time INTEGER NULL
            );",
                params![],
            )
            .unwrap();
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS test (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chip_fk INTEGER NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY(chip_fk) REFERENCES chip(id)
            );",
                params![],
            )
            .unwrap();
    }
    pub fn register_chip(&self, chip_type: &str, software: &str, chip_number: Option<&str>, flashed_time: time::SystemTime) -> usize {
        let timestamp  = flashed_time.duration_since(time::UNIX_EPOCH).unwrap();
        let id = self.conn.execute(
            "INSERT INTO chip (chip_type, software, chip_number, flashed_time)
            VALUES (?1, ?2, ?3, ?4)
        ", params![chip_type, software, chip_number, timestamp.as_secs_f64()]).unwrap();
        id
    }

    pub fn register_test(&self, chip_db_id: usize, key: &str, value: &str) -> usize {
        let conn = Connection::open("db.sqlite3").unwrap();
        let id = conn.execute(
            "INSERT INTO test (chip_fk, key, value)
            VALUES (?1, ?2, ?2)
        ", params![chip_db_id as f64, key, value]).unwrap();
        id
    }
}

