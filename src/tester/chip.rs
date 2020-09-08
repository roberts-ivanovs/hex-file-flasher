use regex::Regex;
use serial::{core::SerialDevice, unix::TTYPort, SerialPortSettings};
use std::collections::HashMap;
use std::{fmt, thread, time};
use std::{
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SoftTypes {
    Master,
    Relay1,
    Relay1_5,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ChipTypes {
    Green,
    BlueShiny,
    BlueNonShiny,
}

impl fmt::Display for ChipTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl fmt::Display for SoftTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Chip {
    pub serial: TTYPort,
    pub id: Option<i32>,
    leftover_buffer: String,
    chip_type: ChipTypes,
    soft_type: SoftTypes,
}

impl Chip {
    pub fn new(
        port_path: &str,
        id: Option<&str>,
        chip_type: ChipTypes,
        soft_type: SoftTypes,
    ) -> Self {
        let mut serial = TTYPort::open(Path::new(port_path)).unwrap();
        let mut settings = serial.read_settings().unwrap();
        settings.set_baud_rate(serial::Baud57600).unwrap();
        serial.write_settings(&settings).unwrap();
        // Perform string id conversion to int id
        let final_id: Option<i32> = match id {
            Some(val) => Some(val.parse::<i32>().unwrap()),
            None => None,
        };

        let mut chip = Self {
            serial,
            id: final_id,
            leftover_buffer: String::new(),
            chip_type,
            soft_type,
        };
        chip.await_startup();
        chip
    }

    pub fn perform_test(
        &mut self,
        id_to_ping: Option<&str>,
        flashed: bool,
        port_to_simulate: Option<&str>,
    ) -> HashMap<String, String> {
        let mut hm = HashMap::new();
        hm.insert("flashed".to_string(), flashed.to_string());
        if flashed {
            match self.soft_type {
                SoftTypes::Master => {
                    // Check RSSI
                    let id_to_ping = id_to_ping.unwrap().parse::<i32>().unwrap();
                    let rssi = self.check_rssi(4, id_to_ping);
                    let rssi = match rssi {
                        Some(v) => v.to_string(),
                        None => r#"n\a"#.to_owned(),
                    };

                    hm.insert("rssi".to_string(), rssi);
                }
                SoftTypes::Relay1 | SoftTypes::Relay1_5 => {
                    // Create a chip instance to the master
                    // NOTE: the chip type means nothing if you are not
                    // flashing the module
                    let mut attached_master = Chip::new(port_to_simulate.unwrap(), None, ChipTypes::Green, SoftTypes::Master);
                    let id_to_ping = self.id.unwrap();
                    // ping the newly flashed relay with the master instance
                    let rssi = attached_master.check_rssi(4, id_to_ping);
                    // set the rssi value
                    let rssi = match rssi {
                        Some(v) => v.to_string(),
                        None => r#"n\a"#.to_owned(),
                    };

                    hm.insert("rssi".to_string(), rssi);
                }
            }
        }
        hm
    }

    // ----------- Implementation details below ---------------- //
    fn await_startup(&mut self) {
        // Perform 'clean buffer'
        self.serial.flush().unwrap();
        match self.soft_type {
            SoftTypes::Master => {
                self.read("i'm a master");
            }
            SoftTypes::Relay1 | SoftTypes::Relay1_5 => {}
        }
    }

    fn check_rssi(&mut self, times: i32, id_to_ping: i32) -> Option<i32> {
        // Perform 'clean buffer'
        self.serial.flush().unwrap();

        let mut total_rssi = 0;
        let mut total_successful_rssi_pings = 0;
        let re = Regex::new(r"[+-?]\d+").unwrap();
        for _ in 0..times {
            // send ping command
            self.ping(id_to_ping);
            // skip to the part where the device responds back
            self.read("<<0");
            // Check if something responded
            let response_line = self.read("rssi is");
            match response_line {
                Some(line) => {
                    for cap in re.captures_iter(&line) {
                        println!("Ping RSSI: {:?}", &cap[0]);
                        total_rssi += &cap[0].parse().unwrap();
                        total_successful_rssi_pings += 1;
                    }
                }
                None => {
                    println!("Nothing responded! {}", self.leftover_buffer);
                    break;
                }
            }
        }
        if total_successful_rssi_pings == 0 {
            None
        } else {
            let rssi = total_rssi / total_successful_rssi_pings;
            Some(rssi)
        }
    }

    fn ping(&mut self, id_to_ping: i32) {
        let ping_command = format!(">>p:{:x}:4\n", id_to_ping);
        let t = time::Duration::from_secs(1);
        thread::sleep(t);
        self.serial.write(ping_command.as_bytes()).unwrap();
    }

    fn read(&mut self, expected: &str) -> Option<String> {
        let mut empty = 0;
        for _ in 0..30 {
            let mut line = self.readline();
            if line == "" || line == "\r" {
                line = "".to_string();
            }
            if empty > 0 && line != "" {
                empty = 0;
            }
            if line == "" {
                empty += 1;
                let two_empty = empty == 5;
                if two_empty {
                    return None;
                }
            }
            if line.contains(expected) {
                return Some(line);
            }
        }
        return None;
    }

    fn readline(&mut self) -> String {
        for _ in 0..3 {
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
                            let t = time::Duration::from_millis(300);
                            thread::sleep(t);
                        }
                    }
                }
            }
        }
        "".to_owned()
    }

    fn get_line_from_current_buffer(&mut self) -> Option<String> {
        // Search currently saved buffer
        let start = 0;
        let mut end = 0;
        for (ind, val) in self.leftover_buffer.chars().enumerate() {
            if val == '\n' {
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
