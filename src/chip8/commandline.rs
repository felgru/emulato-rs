// SPDX-FileCopyrightText: 2021–2022, 2025 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;

use clap::{Arg, ArgMatches, Command};

pub fn chip_8_subcommand() -> Command {
    use super::Chip8;
    Command::new("chip8")
    .about("A Chip-8 emulator")
    .arg(
        Arg::new("rom-file")
            .help("a ROM file to load into the emulator")
            .index(1)
            .required(true),
    )
    .arg(
        Arg::new("display")
            .help("display dimensions")
            .num_args(1)
            .long("display")
            .default_value("64x32")
            .value_parser(Chip8::AVAILABLE_DISPLAY_SIZES)
    )
    .arg(
        Arg::new("font")
            .help("font")
            .num_args(1)
            .long("font")
            .default_value("chip48")
            .value_parser(Chip8::AVAILABLE_FONTS)
    )
    .arg(
        Arg::new("shift-x")
            .help("shift VX instead of VY in 8XY6 and 8XYE (this is what S-CHIP and many other emulators do")
            .long("shift-x")
    )
}

pub fn run_chip_8_from_subcommand(subcommand: &ArgMatches) {
    let display: &String = subcommand.get_one("display").unwrap();
    let font: &String = subcommand.get_one("font").unwrap();
    let shift_x = subcommand.contains_id("shift-x");
    let mut chip8 = super::Chip8::new(display, font, shift_x);
    let filename: &String = subcommand.get_one("rom-file").unwrap();
    println!("loading {}", filename);
    let f = File::open(filename).unwrap();
    chip8.load_rom(f).unwrap();
    chip8.run();
}
