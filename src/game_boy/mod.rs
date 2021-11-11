// SPDX-FileCopyrightText: 2021 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod boot_rom;
pub mod cartridge;
pub mod commandline;
pub mod cpu;
pub mod display;
pub mod emulator_window;
pub mod graphics_data;
pub mod io;
pub mod memory;
pub mod ppu;
pub mod timer;

use std::fs::File;
use std::time::Instant;
use std::thread::sleep;

// TODO
const FRAMERATE: usize = 60;
const CPU_CYCLES_PER_SECOND: usize = 4_194_304;
const CPU_CYCLES_PER_FRAME:  usize = CPU_CYCLES_PER_SECOND / FRAMERATE;
const CPU_CYCLES_PER_SCANLINE: usize = CPU_CYCLES_PER_FRAME / 154;

pub struct GameBoy<Window: io::IO> {
    cpu: cpu::CPU,
    ppu: ppu::PPU,
    memory: memory::MemoryBus,
    emulator_window: Window,
}

impl<Window: io::IO> GameBoy<Window> {
    pub fn builder() -> GameBoyBuilder<Window> {
        GameBoyBuilder::new()
    }

    pub fn new(boot_rom: [u8;0x100],
               cartridge: cartridge::Cartridge,
               window: Window) -> Self {
        let memory = memory::MemoryBus::new(cartridge, boot_rom);
        Self {
            cpu: cpu::CPU::new(),
            ppu: ppu::PPU::new(),
            memory,
            emulator_window: window,
        }
    }

    pub fn run(&mut self) {
        use std::time::Duration;
        let frame_time = Duration::from_micros((1_000_000. / FRAMERATE as f64)
                                               as u64);
        let mut last_frame_time = Instant::now();
        let mut scanline_cycles = 0;
        loop {
            for scanline in 0..144 {
                self.memory.set_ly(scanline);
                self.memory.set_lcd_mode(ppu::LcdMode::SearchingOAM);
                while scanline_cycles <= 80 {
                    scanline_cycles += self.step();
                }
                self.memory.set_lcd_mode(
                    ppu::LcdMode::TransferringDataToLcdController);
                // Approximate mode duration, it actually depends on number
                // of objects to paint, etc.
                self.ppu.paint_line(&mut self.memory);
                while scanline_cycles <= 280 {
                    scanline_cycles += self.step();
                }
                self.memory.set_lcd_mode(ppu::LcdMode::HBlank);
                // TODO: This does not add up exactly, as we assume 60FPS
                //       here, but it are actually slightly less.
                while scanline_cycles < CPU_CYCLES_PER_SCANLINE {
                    scanline_cycles += self.step();
                }
                scanline_cycles %= CPU_CYCLES_PER_SCANLINE;
            }
            let current_frame_time = Instant::now();
            let elapsed = current_frame_time.duration_since(last_frame_time);
            if let Some(sleep_duration) = frame_time.checked_sub(elapsed) {
                sleep(sleep_duration);
            }
            last_frame_time = current_frame_time;
            self.ppu.refresh(&mut self.emulator_window);
            scanline_cycles += self.check_key_presses();
            if self.emulator_window.is_esc_pressed() {
                break;
            }
            for scanline in 144..154 {
                self.memory.set_ly(scanline);
                if scanline == 144 {
                    self.memory.set_lcd_mode(ppu::LcdMode::VBlank);
                    // request VBlank interrupt
                    let requests = self.memory.read8(0xFF0F) | 1;
                    self.memory.write8(0xFF0F, requests);
                }
                while scanline_cycles < CPU_CYCLES_PER_SCANLINE {
                    scanline_cycles += self.step();
                }
                scanline_cycles %= CPU_CYCLES_PER_SCANLINE;
            }
        }
    }

    fn step(&mut self) -> usize {
        let mut cycles = self.cpu.step(&mut self.memory);
        self.memory.step(cycles);
        if self.handle_interrupts() {
            cycles += 5 * 4;
            self.memory.step(5 * 4);
        }
        cycles
    }

    fn check_key_presses(&mut self) -> usize {
        if self.memory.set_key_presses(
            self.emulator_window.get_key_presses())
           && self.handle_interrupts() {
            self.memory.step(5 * 4);
            5 * 4
        } else {
            0
        }
    }

    fn handle_interrupts(&mut self) -> bool {
        self.cpu.handle_interrupts(&mut self.memory)
    }
}

pub struct GameBoyBuilder<Window: io::IO> {
    boot_rom: Option<[u8;0x100]>,
    cartridge: Option<cartridge::Cartridge>,
    window: Option<Window>,
}

impl<Window: io::IO> GameBoyBuilder<Window> {
    pub fn new() -> Self {
        Self {
            boot_rom: None,
            cartridge: None,
            window: None,
        }
    }

    pub fn build(self) -> GameBoy<Window> {
        GameBoy::new(self.boot_rom.unwrap(),
                     self.cartridge.unwrap(),
                     self.window.unwrap())
    }

    pub fn load_boot_rom(mut self, file: File) -> std::io::Result<Self> {
        let boot_rom = boot_rom::load_boot_rom(file)?;
        self.boot_rom = Some(boot_rom);
        Ok(self)
    }

    pub fn use_fast_boot_rom(mut self) -> Self {
        let boot_rom = boot_rom::fast_boot_rom();
        self.boot_rom = Some(boot_rom);
        self
    }

    pub fn load_cartridge(mut self, file: File) -> std::io::Result<Self> {
        self.cartridge = Some(cartridge::Cartridge::load_from_file(file)?);
        Ok(self)
    }

    pub fn use_emulator_window(mut self, window: Window) -> Self {
        self.window = Some(window);
        self
    }

    pub fn get_cartridge_header(&self) -> Option<cartridge::CartridgeHeader> {
        self.cartridge.as_ref().map(|c| c.header())
    }
}
