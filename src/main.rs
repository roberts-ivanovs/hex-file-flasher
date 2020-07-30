mod tester;

#[macro_use]
extern crate clap;

use clap::{App, ArgMatches};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Perform flashing
    let port_to_flash = matches.value_of("port-to-flash").unwrap();
    let chip_type = get_chip_type(&matches).unwrap_or_else(|| {
        panic!("Invalid chip type!",);
    });
    let soft_type = get_soft_type(&matches).unwrap_or_else(|| {
        panic!("Invalid soft type!",);
    });
    let id_to_flash = matches.value_of("id-to-flash");
    if matches.is_present("port-to-flash") && !matches.is_present("only-test") {

        println!(
            "Flashing {} as {} chip with {} code with ID {:?} ",
            port_to_flash, chip_type, soft_type, id_to_flash
        );
    }

    // Perform testing
    let mut device_to_test = tester::chip::Chip::new(port_to_flash, id_to_flash);
    device_to_test.check_rssi(4, 1);
}

fn get_chip_type(matches: &ArgMatches) -> Option<String> {
    for possible_type in vec!["green", "blue-shiny", "blue-non-shiny"] {
        if matches.is_present(possible_type) {
            return Some(possible_type.to_string());
        }
    }
    None
}

fn get_soft_type(matches: &ArgMatches) -> Option<String> {
    for possible_type in vec!["rel-mk1", "rel-mk1.5", "master"] {
        if matches.is_present(possible_type) {
            return Some(possible_type.to_string());
        }
    }
    None
}
