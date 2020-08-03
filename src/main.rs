mod db;
mod hex;
mod tester;
mod utils;

#[macro_use]
extern crate clap;

use crate::hex::flash;
use clap::{App, ArgMatches};

use promptly::{prompt, prompt_default};
use std::process;
use std::time;
use tester::chip::{ChipTypes, SoftTypes};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // ------------------Arg extracting-------------------- //
    let port_to_flash = matches.value_of("port-to-flash").unwrap();
    let chip_type = get_chip_type(&matches).unwrap();
    let soft_type = get_soft_type(&matches).unwrap();
    let id_to_flash = matches.value_of("id-to-flash");
    let factory_number = matches.value_of("chip-factory-number");
    let id_to_ping = matches.value_of("id-to-ping");

    // ------------------Validation-------------------- //
    if matches.is_present("port-to-simulate") && soft_type == SoftTypes::Master {
        utils::print_error_and_exit("No reason to simulate port when flashing Master software!");
        process::exit(1);
    }

    if !matches.is_present("id-to-ping") && soft_type == SoftTypes::Master {
        utils::print_error_and_exit("Master needs an ID to ping!");
        process::exit(1);
    }

    if matches.is_present("id-to-flash") && soft_type == SoftTypes::Master {
        utils::print_error_and_exit("There's no reason for a master soft to contain an ID");
        process::exit(1);
    }

    if chip_type == ChipTypes::Green && !matches.is_present("chip-factory-number") {
        utils::print_error_and_exit(
            "When flashing green chips be sure to pass the chips factory number.
        It usually can be found on the bottom of the chip with a QR code attached",
        );
    }

    // ------------------Perform flashing-------------------- //
    let mut flashed = false;
    if matches.is_present("port-to-flash") && !matches.is_present("only-test") {
        println!(
            "Flashing {} as {:?} chip with {:?} code with ID {:?} ",
            port_to_flash, chip_type, soft_type, id_to_flash
        );

        loop {
            flashed = flash(&chip_type, &soft_type, &port_to_flash, id_to_flash);
            if flashed {
                println!("Flashing went OK!");
            } else {
                println!("Something went wrong while flashing!");
            }
            let reflash = prompt_default("Flash again?", true).unwrap();
            if !reflash {
                break;
            }
        }
    }

    // ------------------Perform testing-------------------- //
    let mut device_to_test =
        tester::chip::Chip::new(port_to_flash, id_to_flash, chip_type, soft_type);
    let test_results = device_to_test.perform_test(id_to_ping, flashed);

    // ------------------Save to database-------------------- //
    let db_instance = db::DbInstance::new();
    let chip_id = db_instance.register_chip(&chip_type.to_string(), factory_number);
    let flash_id = db_instance.register_flash(
        chip_id,
        &soft_type.to_string(),
        time::SystemTime::now(),
        id_to_flash,
    );
    for (key, val) in test_results.iter() {
        db_instance.register_test(flash_id, key, val);
    }
    // ------------------Save to csv file-------------------- //
    // TODO Call external Python code
}

fn get_chip_type(matches: &ArgMatches) -> Result<ChipTypes, String> {
    let chip_t = matches.value_of("chip-type").unwrap();
    match chip_t {
        "green" => Ok(ChipTypes::Green),
        "blue-shiny" => Ok(ChipTypes::BlueShiny),
        "blue-non-shiny" => Ok(ChipTypes::BlueNonShiny),
        _ => Err("Unsupported chip type".to_string()),
    }
}

fn get_soft_type(matches: &ArgMatches) -> Result<SoftTypes, String> {
    let chip_t = matches.value_of("soft-type").unwrap();
    match chip_t {
        "rel-mk1" => Ok(SoftTypes::Relay1),
        "rel-mk1.5" => Ok(SoftTypes::Relay1_5),
        "master" => Ok(SoftTypes::Master),
        _ => Err("Unsupported software".to_string()),
    }
}
