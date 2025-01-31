// SPDX-FileCopyrightText: 2021–2022, 2025 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;

use clap::{Arg, ArgMatches, Command};

use super::cartridge::CartridgeHeader;
use super::emulator_window::EmulatorWindow;
use super::GameBoy;

pub fn game_boy_subcommand() -> Command {
    Command::new("gameboy")
    .about("A Game Boy emulator")
    .arg(
        Arg::new("cartridge-file")
            .help("a ROM file to load into the emulator")
            .index(1)
            .required(true),
    )
    .arg(
        Arg::new("boot-rom")
            .help("path to boot ROM")
            .num_args(1)
            .long("boot-rom")
    )
    .arg(
        Arg::new("dump-header")
            .help("print cartridge header")
            .long("dump-header")
    )
}

pub fn run_game_boy_from_subcommand(subcommand: &ArgMatches) {
    let mut builder = GameBoy::<EmulatorWindow>::builder();
    let filename: &String = subcommand.get_one("cartridge-file").unwrap();
    let f = File::open(filename).unwrap();
    builder = builder.load_cartridge(f).unwrap();
    if let Some(boot_rom) = subcommand.get_one::<&String>("boot-rom") {
        let f = File::open(boot_rom).unwrap();
        builder = builder.load_boot_rom(f).unwrap();
    } else {
        builder = builder.use_fast_boot_rom();
    }
    if subcommand.contains_id("dump-header") {
        print_cartridge_header(builder.get_cartridge_header().unwrap())
    } else {
        let mut game_boy
            = builder.use_emulator_window(EmulatorWindow::default())
                     .build();
        game_boy.run();
    }
}

fn print_cartridge_header(header: CartridgeHeader) {
    if let Some(title) = std::str::from_utf8(header.title()).ok() {
        println!("Title: {}", title);
    } else {
        println!("Could not decode title: {:#X?}", header.title());
    }
    if let Some(code) = header.manufacturer_code() {
        println!("Manufacturer code: {}", code);
    }
    println!("Cartridge type: {:?}", header.cartridge_type());
    println!("Memory Controller: {:?}", header.cartridge_type()
                                              .memory_controller());

    println!("Color compat: {:?}", header.color_compat());
    println!("Supports SGB function: {}", header.supports_sgb_function());

    println!("ROM banks: {}", header.num_rom_banks());
    println!("RAM banks: {}", header.num_ram_banks());

    println!("ROM version: {}", header.rom_version());
    print!("Licensee code: ");
    if header.uses_new_licensee_code() {
        println!("{}", header.new_licensee_code().unwrap());
    } else {
        println!("{:0>2X}", header.old_licensee_code());
    }
    println!("is Japanese: {}", header.is_japanese());

    println!("Logo is {}.",
             if header.is_logo_correct() { "correct" } else { "wrong" });
    println!("Header checksum is {}.",
             if header.is_header_checksum_correct() {
                 "correct"
             } else {
                 "wrong"
             });
}
