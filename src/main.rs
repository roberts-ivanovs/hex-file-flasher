#[macro_use]
extern crate clap;

use clap::App;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Perform flashing
    if matches.is_present("port-to-flash") && !matches.is_present("only-test") {
        let port_to_flash = matches.value_of("port-to-flash").unwrap();

        let mut chip_type = None;
        for possible_type in vec!["green", "blue-shiny", "blue-non-shiny"] {
            if matches.is_present(possible_type) {
                chip_type = Some(possible_type);
                break;
            }
        }
        let mut soft_type = None;
        for possible_type in vec!["rel-mk1", "rel-mk1.5", "master"] {
            if matches.is_present(possible_type) {
                soft_type = Some(possible_type);
                break;
            }
        }
        if soft_type == None || chip_type == None {
            panic!(
                "Invalid soft_type {:?} or chip_type {:?}",
                soft_type, chip_type
            );
        }

        let id_to_flash = matches.value_of("id-to-flash");

        println!(
            "Flashing {} as {} chip with {} code with ID {:?} ",
            port_to_flash,
            chip_type.unwrap(),
            soft_type.unwrap(),
            id_to_flash
        );
    }

    // Perform testing
}
