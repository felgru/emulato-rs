<!--
SPDX-FileCopyrightText: 2021 Felix Gruber

SPDX-License-Identifier: GPL-3.0-or-later
-->

# Emulato.rs

> A collection of emulators that I've created to learn the internals of old gaming consoles.

## Emulated Systems

* [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
  * 8-bit virtual machine for very simple games.
  * Complete and functioning implementation, except for sound.
* [Game Boy](https://en.wikipedia.org/wiki/Game_Boy)
  * Everyone's favorite 8-bit handheld console from the 90's.
  * Implementation is still work in progress. While some games work well enough
    to be playable, others suffer from graphical glitches or even random crashes
    on unimplemented features.
    Already passes a large part of the
    [mooneye](https://github.com/wilbertpol/mooneye-gb/tree/master/tests)
    test ROMs, but notably those tests related to the Game Boy's timers are
    still failing.
    The audio processing unit is not emulated yet, so no sound for now.

## Usage

You can use Rust's `cargo` build tool to run the emulators.
See
```
cargo run --release -- --help
```
for an overview of available command line options. Each emulator sub-command
has its own `--help` message with the arguments it accepts.

To run a given ROM file in the Game Boy emulator, you can run
```
cargo run --release -- gameboy <path_to_rom_file>
```

## License

This program is licensed under the GPL version 3 or (at your option)
any later version.

The text of the GPL version 3 can be found in the LICENSES directory.
