use serial::{
    core::SerialDevice,
    unix::{TTYPort, TTYSettings},
    SerialPortSettings,
};
use std::{
    io::{Read, Write},
    path::Path,
};
use std::{thread, time};

pub struct Chip {
    pub serial: TTYPort,
    pub id: Option<i32>,
    pub rssi: Option<i32>,
    leftover_buffer: String,
}

impl Chip {
    pub fn new(port_path: &str, id: Option<&str>) -> Self {
        let mut serial = TTYPort::open(Path::new(port_path)).unwrap();
        let mut settings = serial.read_settings().unwrap();
        settings.set_baud_rate(serial::Baud57600).unwrap();
        serial.write_settings(&settings);
        // Perform string id conversion to int id
        let final_id: Option<i32> = match id {
            Some(val) => Some(val.parse::<i32>().unwrap()),
            None => None,
        };

        Self {
            serial,
            id: final_id,
            rssi: None,
            leftover_buffer: String::new(),
        }
    }

    pub fn check_rssi(&mut self, times: i32, id_to_ping: i32) {
        // Perform 'clean buffer'
        self.serial.flush().unwrap();
        self.read("i'm a master");
        // send ping command
        self.ping(id_to_ping);
        // skip to the part where the device responds back
        self.read("<<0");
        // Check if something responded
        if !self.read("rssi is") {
            println!("Nothing responded!");
            self.rssi = None;
        }
    }

    fn ping(&mut self, id_to_ping: i32) {
        let ping_command = format!(">>p:{:x}:4\n", id_to_ping);
        println!("{}", ping_command);
        let t = time::Duration::from_secs(1);
        thread::sleep(t);
        self.serial.write(ping_command.as_bytes()).unwrap();
    }

    fn read(&mut self, expected: &str) -> bool {
        let mut empty = 0;
        for i in 0..15 {
            let mut line = self.readline();
            if line == "" || line == "\r" {
                line = "".to_string();
            }
            if empty > 0 && line != "" {
                empty = 0;
            }
            if line == "" {
                empty += 1;
                let two_empty = empty == 2;
                if two_empty {
                    return false;
                }
            }
            if line.contains(expected) {
                return true;
            }
        }
        return false;
    }

    fn readline(&mut self) -> String {
        let line = self.get_line_from_current_buffer();

        match line {
            Some(val) => return val,
            None => {
                // Perform buffer reading
                let mut res = [0; 10000];
                let elems = self.serial.read(&mut res);

                match elems {
                    Ok(val) => {
                        if val == 0 {
                            return "".to_string();
                        } else {
                            let response = &res[0..val];
                            let lines = std::str::from_utf8(&response).unwrap();
                            self.leftover_buffer += lines;
                        }
                    }
                    Err(_) => {
                        // sleep for a bit
                        let t = time::Duration::from_millis(100);
                        thread::sleep(t);
                    }
                }
            }
        }
        self.readline()
    }

    fn get_line_from_current_buffer(&mut self) -> Option<String> {
        // Search currently saved buffer
        let start = 0;
        let mut end = 0;
        for (ind, val) in self.leftover_buffer.chars().enumerate() {
            if val == '\r' {
                end = end + 1;
                let returnable = self.leftover_buffer[start..end].to_string();
                self.leftover_buffer =
                    self.leftover_buffer[end..self.leftover_buffer.len()].to_string();
                return Some(returnable);
            }
            end = ind;
        }
        None
    }
}
