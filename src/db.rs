use rusqlite::{named_params, params, Connection, Result};
use std::collections::{HashSet, HashMap};
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

    pub fn read_data(&self) -> (Vec<HashMap<String, String>>, Vec<String>) {
        // Get test headers
        let mut test_keys = vec![];
        let mut stmt = self
            .conn
            .prepare("SELECT key FROM test GROUP BY key;")
            .unwrap();
        let mut headers = stmt.query(params![]).unwrap();
        while let Some(row) = headers.next().unwrap() {
            let key: String = row.get(0).unwrap();
            test_keys.push(key);
        }

        // Get chips
        let mut stmt = self
            .conn
            .prepare("SELECT chip.id, chip_number, chip_type, flash.software, flash.flashed_id, flash.flashed_time FROM chip JOIN flash ON flash.chip_fk=chip.id ORDER BY chip.chip_number, flash.id;")
            .unwrap();
        let mut chips = stmt.query(params![]).unwrap();

        let mut device_result = vec![];
        let mut viewed: HashSet<i32> = HashSet::new();
        while let Some(row) = chips.next().unwrap() {
            let mut dev_hm: HashMap<String, String> = HashMap::new();
            // Get the max flashed pk (the most recent flash for the given chip)
            let mut stmt = self
                .conn
                .prepare("SELECT MAX(flash.id) FROM flash WHERE chip_fk=?;")
                .unwrap();
            let chip_fk: i32 = row.get(0).unwrap();

            if viewed.contains(&chip_fk) {
                continue;
            } else {
                viewed.insert(chip_fk);
            }

            let mut max_id_rows = stmt.query(params![chip_fk]).unwrap();
            let mut flash_pk = -1;

            while let Some(row_flash_pk) = max_id_rows.next().unwrap() {
                flash_pk = row_flash_pk.get(0).unwrap();
            }


            let chip_number: String = row.get(1).unwrap();

            let chip_type: String = row.get(2).unwrap();
            let software: String = row.get(3).unwrap();
            let flashed_id: Option<String> = row.get(4).unwrap();
            let flashed_time: f64 = row.get(5).unwrap();
            dev_hm.insert("chip_number".to_string(), chip_number);
            dev_hm.insert("chip_type".to_string(), chip_type);
            dev_hm.insert("software".to_string(), software);
            dev_hm.insert(
                "flashed_id".to_string(),
                flashed_id.unwrap_or_else(|| "".to_string()),
            );
            dev_hm.insert("flashed_time".to_string(), flashed_time.to_string());

            test_keys.iter().for_each(|key| {
                let mut stmt = self
                    .conn
                    .prepare(r#"SELECT value FROM test WHERE flash_fk=:flash_fk AND key=:key;"#)
                    .unwrap();
                let mut test_results = stmt
                    .query_named(named_params! { ":flash_fk": flash_pk, ":key": key })
                    .unwrap();
                while let Some(res) = test_results.next().unwrap() {
                    let val = res.get(0).unwrap();
                    dev_hm.insert(key.clone(), val);
                }
            });
            device_result.push(dev_hm);
        }
        (device_result, test_keys)
    }
}
