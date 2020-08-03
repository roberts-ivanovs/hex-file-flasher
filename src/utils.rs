use std::process;

pub fn print_error_and_exit(error: &str) {
    eprintln!("{}", error);
    process::exit(1);
}
