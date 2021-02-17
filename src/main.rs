use std::fs::File;

use clap::{crate_name, crate_version, App, AppSettings, Arg};

use emulato_rs::chip8;

fn main() {
    let matches = App::new(crate_name!())
        .about("A collection of emulators.")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("chip8")
            .about("A Chip-8 emulator")
            .arg(
                Arg::new("rom-file")
                    .about("a ROM file to load into the emulator")
                    .index(1)
                    .required(true),
            )
            .arg(
                Arg::new("display")
                    .about("display dimensions")
                    .takes_value(true)
                    .long("display")
                    .default_value("64x32")
                    .possible_values(&chip8::Chip8::AVAILABLE_DISPLAY_SIZES)
            )
            .arg(
                Arg::new("font")
                    .about("font")
                    .takes_value(true)
                    .long("font")
                    .default_value("chip48")
                    .possible_values(&chip8::Chip8::AVAILABLE_FONTS)
            ),
        )
        .get_matches();
    match matches.subcommand_name() {
        Some("chip8") => {
            let subcommand = matches.subcommand_matches("chip8").unwrap();
            let display = subcommand.value_of("display").unwrap();
            let font = subcommand.value_of("font").unwrap();
            let mut chip8 = chip8::Chip8::new(display, font);
            let filename = subcommand.value_of("rom-file").unwrap();
            println!("loading {}", filename);
            let f = File::open(filename).unwrap();
            chip8.load_rom(f).unwrap();
            chip8.run();
        }
        Some(s) => {
            eprintln!("Unknown emulator: {}", s);
        }
        None => {
            eprintln!("Missing emulator argument.");
        }
    }
}
