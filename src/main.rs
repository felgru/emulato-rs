use clap::{crate_name, crate_version, App, AppSettings};

use emulato_rs::chip8;

fn main() {
    let matches = App::new(crate_name!())
        .about("A collection of emulators.")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(chip8::commandline::chip_8_subcommand())
        .get_matches();
    match matches.subcommand() {
        Some(("chip8", chip8matches)) => {
            chip8::commandline::run_chip_8_from_subcommand(chip8matches);
        }
        Some((s, _)) => {
            eprintln!("Unknown emulator: {}", s);
        }
        None => {
            eprintln!("Missing emulator argument.");
        }
    }
}
