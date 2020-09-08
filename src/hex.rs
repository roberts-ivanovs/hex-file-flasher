use crate::{
    tester::chip::{ChipTypes, SoftTypes},
    utils::print_error_and_exit,
};
use subprocess::{Popen, PopenConfig};
use regex::Regex;

pub fn flash(
    chip: &ChipTypes,
    soft: &SoftTypes,
    port_to_flash: &str,
    id_to_flash: Option<&str>,
) -> bool {
    // Get code directory
    let code_directory_path = match chip {
        ChipTypes::Green => "green".to_owned(),
        ChipTypes::BlueShiny => "blue/shiny".to_owned(),
        ChipTypes::BlueNonShiny => "blue/nonshiny".to_owned(),
    };
    let code_directory_hex = match soft {
        SoftTypes::Master => format!("master.{}.hex", get_avr_device(&chip)),
        SoftTypes::Relay1 => format!("relay_mk1.{}.hex", get_avr_device(&chip)),
        SoftTypes::Relay1_5 => format!("relay_mk1_5.{}.hex", get_avr_device(&chip)),
    };
    let code_directory = format!("hex/{}/{}", code_directory_path, code_directory_hex);
    let programmer_type = get_programmer_device(&chip);
    let factual_avrdude_port = get_proper_port(&chip, &port_to_flash);
    let prcs = match soft {
        SoftTypes::Master => Popen::create(
            &[
                "avrdude",
                "-p",
                get_avr_device(&chip),
                "-P",
                factual_avrdude_port.as_ref(),
                "-c",
                programmer_type,
                "-b",
                "57600",
                "-U",
                &format!("flash:w:{}", code_directory),
            ],
            PopenConfig::default(),
        ),
        SoftTypes::Relay1 | SoftTypes::Relay1_5 => Popen::create(
            &[
                "avrdude",
                "-p",
                get_avr_device(&chip),
                "-c",
                programmer_type,
                "-U",
                &format!("flash:w:{}", code_directory),
                "-U",
                &format!("eeprom:w:hex/eeproms/eeprom_{}.hex", id_to_flash.unwrap()),
                "-P",
                factual_avrdude_port.as_ref(),
            ],
            PopenConfig::default(),
        ),
    };

    let prcs_status = prcs.unwrap().wait();

    match prcs_status {
        Ok(p) => p.success() == true,
        Err(_) => false,
    }
}

fn get_avr_device(chip: &ChipTypes) -> &str {
    match chip {
        ChipTypes::Green => "x16e5",
        ChipTypes::BlueShiny => "m328p",
        ChipTypes::BlueNonShiny => "m328p",
    }
}

fn get_programmer_device(chip: &ChipTypes) -> &str {
    match chip {
        ChipTypes::Green => "atmelice_pdi",
        ChipTypes::BlueShiny => "arduino",
        ChipTypes::BlueNonShiny => "arduino",
    }
}
fn get_proper_port(chip: &ChipTypes, initial_port: &str) -> String {
    match chip {
        ChipTypes::Green => extract_for_atmelice(initial_port).unwrap(),
        // the arduino programmer is fine with "/dev/ttyUSB0" port specifier
        ChipTypes::BlueShiny => initial_port.to_owned(),
        ChipTypes::BlueNonShiny => initial_port.to_owned(),
    }
}

fn extract_for_atmelice(initial_port: &str) -> Option<String> {
    let re = Regex::new(r"d+").unwrap();
    for cap in re.captures_iter(&initial_port) {
        return Some(format!("usb{}", &cap[0]));
    }
    None
}
