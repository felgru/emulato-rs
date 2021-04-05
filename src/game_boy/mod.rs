pub mod cartridge;
pub mod commandline;
pub mod cpu;
pub mod display;
pub mod emulator_window;
pub mod graphics_data;
pub mod memory;
pub mod ppu;

use std::io;
use std::fs::File;

// TODO
const FRAMERATE:  usize = 60;
const CPU_CYCLES_PER_SECOND: usize = 4_194_304;
const CPU_CYCLES_PER_FRAME:  usize = CPU_CYCLES_PER_SECOND / FRAMERATE;

pub struct GameBoy {
    cpu: cpu::CPU,
    ppu: ppu::PPU,
    // display: display::Display,
}

impl GameBoy {
    pub fn builder() -> GameBoyBuilder {
        GameBoyBuilder::new()
    }

    pub fn new(boot_rom: [u8;0x100], cartridge: cartridge::Cartridge) -> Self {
        let bus = memory::MemoryBus::new(cartridge, boot_rom);
        Self {
            cpu: cpu::CPU::new(bus),
            ppu: ppu::PPU::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut frame_cycles = 0;
            while frame_cycles < CPU_CYCLES_PER_FRAME {
                self.cpu.step();
                frame_cycles += 4;
                self.ppu.update();
            }
            // self.display.refresh();
            // if self.display.is_esc_pressed() {
                // break;
            // }
        }
    }
}

pub struct GameBoyBuilder {
    boot_rom: Option<[u8;0x100]>,
    cartridge: Option<cartridge::Cartridge>,
}

impl GameBoyBuilder {
    pub fn new() -> Self {
        Self {
            boot_rom: None,
            cartridge: None,
        }
    }

    pub fn build(self) -> GameBoy {
        GameBoy::new(self.boot_rom.unwrap(), self.cartridge.unwrap())
    }

    pub fn load_boot_rom(mut self, file: File) -> io::Result<Self> {
        let boot_rom = memory::MemoryBus::load_boot_rom(file)?;
        self.boot_rom = Some(boot_rom);
        Ok(self)
    }

    pub fn load_cartridge(mut self, file: File) -> io::Result<Self> {
        self.cartridge = Some(cartridge::Cartridge::load_from_file(file)?);
        Ok(self)
    }
}
