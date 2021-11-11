// SPDX-FileCopyrightText: 2021 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;

use clap::{App, Arg, ArgMatches};

pub fn chip_8_subcommand<'a>() -> App<'a> {
    use super::Chip8;
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
            .possible_values(&Chip8::AVAILABLE_DISPLAY_SIZES)
    )
    .arg(
        Arg::new("font")
            .about("font")
            .takes_value(true)
            .long("font")
            .default_value("chip48")
            .possible_values(&Chip8::AVAILABLE_FONTS)
    )
    .arg(
        Arg::new("shift-x")
            .about("shift VX instead of VY in 8XY6 and 8XYE (this is what S-CHIP and many other emulators do")
            .long("shift-x")
    )
}

pub fn run_chip_8_from_subcommand(subcommand: &ArgMatches) {
    let display = subcommand.value_of("display").unwrap();
    let font = subcommand.value_of("font").unwrap();
    let shift_x = subcommand.is_present("shift-x");
    let mut chip8 = super::Chip8::new(display, font, shift_x);
    let filename = subcommand.value_of("rom-file").unwrap();
    println!("loading {}", filename);
    let f = File::open(filename).unwrap();
    chip8.load_rom(f).unwrap();
    chip8.run();
}
