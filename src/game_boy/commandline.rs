use std::fs::File;

use clap::{App, Arg, ArgMatches};

use super::emulator_window::EmulatorWindow;
use super::GameBoy;

pub fn game_boy_subcommand<'a>() -> App<'a> {
    App::new("gameboy")
    .about("A Game Boy emulator")
    .arg(
        Arg::new("cartridge-file")
            .about("a ROM file to load into the emulator")
            .index(1)
            .required(true),
    )
    .arg(
        Arg::new("boot-rom")
            .about("path to boot ROM")
            .takes_value(true)
            .long("boot-rom")
    )
}

pub fn run_game_boy_from_subcommand(subcommand: &ArgMatches) {
    let mut builder = GameBoy::<EmulatorWindow>::builder()
        .use_emulator_window(EmulatorWindow::default());
    let filename = subcommand.value_of("cartridge-file").unwrap();
    let f = File::open(filename).unwrap();
    builder = builder.load_cartridge(f).unwrap();
    if let Some(boot_rom) = subcommand.value_of("boot-rom") {
        let f = File::open(boot_rom).unwrap();
        builder = builder.load_boot_rom(f).unwrap();
    } else {
        builder = builder.use_fast_boot_rom();
    }
    let mut game_boy = builder.build();
    game_boy.run();
}
