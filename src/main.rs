// SPDX-FileCopyrightText: 2021–2022 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::{crate_name, crate_version, Command};

use emulato_rs::chip8;
use emulato_rs::game_boy;

fn main() {
    let matches = Command::new(crate_name!())
        .about("A collection of emulators.")
        .version(crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(chip8::commandline::chip_8_subcommand())
        .subcommand(game_boy::commandline::game_boy_subcommand())
        .get_matches();
    match matches.subcommand() {
        Some(("chip8", matches)) => {
            chip8::commandline::run_chip_8_from_subcommand(matches);
        }
        Some(("gameboy", matches)) => {
            game_boy::commandline::run_game_boy_from_subcommand(matches);
        }
        Some((s, _)) => {
            eprintln!("Unknown emulator: {}", s);
        }
        None => {
            eprintln!("Missing emulator argument.");
        }
    }
}
