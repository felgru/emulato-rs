// SPDX-FileCopyrightText: 2021 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod cpu;
pub mod display;
pub mod fonts;
pub mod memory;
pub mod commandline;

use std::io;
use std::fs::File;

pub struct Chip8 {
    cpu: cpu::CPU,
    memory: memory::Memory,
    display: display::Display,
}

const FRAMERATE:  usize = 60;
const CPU_CYCLES_PER_FRAME:  usize = 10;

impl Chip8 {
    pub const AVAILABLE_DISPLAY_SIZES: [&'static str; 3] = [
        "64x32",  //< CHIP-8
        "128x64",  //< CHIP-10
        "64x128",  //< HI-RES CHIP-8
    ];

    pub const AVAILABLE_FONTS: [&'static str; 5] = [
        "chip48",
        "cosmacvip",
        "dream6800",
        "eti660",
        "fishnchips",
    ];

    pub fn new(display_size: &str, font: &str, shift_x: bool) -> Self {
        let (width, height) = match display_size {
            "64x32" => (64, 32),
            "128x64" => (128, 64),
            "64x128" => (64, 128),
            _ => panic!("Unexpected CHIP-8 display size: {}", display_size),
        };
        let font = match font {
            "chip48" => &fonts::CHIP48_FONT,
            "cosmacvip" => &fonts::COSMAC_VIP_FONT,
            "dream6800" => &fonts::DREAM6800_FONT,
            "eti660" => &fonts::ETI660_FONT,
            "fishnchips" => &fonts::FISH_N_CHIPS_FONT,
            _ => panic!("Unknown CHIP-8 font: {}", font),
        };
        let mut cpu = cpu::CPU::default();
        if shift_x {
            cpu.activate_shift_quirk();
        }
        let memory = memory::Memory::with_font(font);
        let display = display::Display::new(width, height, FRAMERATE);
        Self {
            cpu,
            memory,
            display,
        }
    }

    pub fn load_rom(&mut self, file: File) -> io::Result<()> {
        self.memory.load_program_from_file(file)
    }

    pub fn run(&mut self) {
        loop {
            for _ in 0..CPU_CYCLES_PER_FRAME {
                self.cpu.tick(&mut self.memory, &mut self.display);
            }
            self.cpu.decrement_timers();
            self.display.refresh();
            if self.display.is_esc_pressed() {
                break;
            }
        }
    }
}
