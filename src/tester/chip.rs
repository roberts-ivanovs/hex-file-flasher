use std::{io::Write, path::Path};
use serial::unix::TTYPort;

pub struct Chip {
    pub serial: TTYPort,
    pub id: Option<i32>,
    pub rssi: Option<i32>,
}

impl Chip {
    pub fn new(port_path: &str, id: Option<&str>) -> Self {
        let serial = TTYPort::open(Path::new(port_path)).unwrap();

        // Perform string id conversion to int id
        let final_id: Option<i32> = match id {
            Some(val) => {
                Some(val.parse::<i32>().unwrap())
            }
            None => {
                None
            }
        };


        Self { serial, id: final_id, rssi: None }
    }


    pub fn check_rssi(&mut self, times: i32, id_to_ping: i32) {
        // Perform 'clean buffer'
        self.serial.flush().unwrap();
        // send ping command
        let ping_command = format!(">>p:{:x}:4", id_to_ping);
        println!("{}", ping_command);
        ping_command.as_bytes();
        self.serial.write(ping_command.as_bytes()).unwrap();
        // skip to the part where the device responds back

    }
}

