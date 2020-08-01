mod db;
mod tester;

#[macro_use]
extern crate clap;

use clap::{App, ArgMatches};

use tester::chip::{ChipTypes, SoftTypes};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    println!("{:?}", matches);

    // ------------------Arg extracting-------------------- //
    let port_to_flash = matches.value_of("port-to-flash").unwrap();
    let chip_type = get_chip_type(&matches).unwrap();
    let soft_type = get_soft_type(&matches).unwrap();
    let id_to_flash = matches.value_of("id-to-flash");

    // ------------------Validation-------------------- //
    if matches.is_present("port-to-simulate") && soft_type == SoftTypes::Master {
        panic!("No reason to simulate port when flashing Master software!");
    }

    if matches.is_present("id-to-ping")
        && (soft_type == SoftTypes::Relay1 || soft_type == SoftTypes::Relay1_5)
    {
        panic!("When flashing relays there's no need to ping a different chip other than the ID that's getting flashed!");
    }

    if matches.is_present("id-to-flash") && soft_type == SoftTypes::Master {
        panic!("There's no reason for a master soft to contain an ID");
    }

    // ------------------Perform flashing-------------------- //
    if matches.is_present("port-to-flash") && !matches.is_present("only-test") {
        println!(
            "Flashing {} as {:?} chip with {:?} code with ID {:?} ",
            port_to_flash, chip_type, soft_type, id_to_flash
        );
    }

    // ------------------Perform testing-------------------- //
    let mut device_to_test = tester::chip::Chip::new(port_to_flash, id_to_flash, chip_type, soft_type);
    device_to_test.check_rssi(4, 1);
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
