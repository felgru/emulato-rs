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
use std::time::Instant;
use std::thread::sleep;

// TODO
const FRAMERATE: usize = 60;
const CPU_CYCLES_PER_SECOND: usize = 4_194_304;
const CPU_CYCLES_PER_FRAME:  usize = CPU_CYCLES_PER_SECOND / FRAMERATE;
const CPU_CYCLES_PER_SCANLINE: usize = CPU_CYCLES_PER_FRAME / 154;

pub struct GameBoy {
    cpu: cpu::CPU,
    ppu: ppu::PPU,
    memory: memory::MemoryBus,
    emulator_window: emulator_window::EmulatorWindow,
}

impl GameBoy {
    pub fn builder() -> GameBoyBuilder {
        GameBoyBuilder::new()
    }

    pub fn new(boot_rom: [u8;0x100], cartridge: cartridge::Cartridge) -> Self {
        let memory = memory::MemoryBus::new(cartridge, boot_rom);
        Self {
            cpu: cpu::CPU::new(),
            ppu: ppu::PPU::new(),
            memory,
            emulator_window: emulator_window::EmulatorWindow::new(),
        }
    }

    pub fn run(&mut self) {
        use std::time::Duration;
        let frame_time = Duration::from_micros((1_000_000. / FRAMERATE as f64)
                                               as u64);
        let mut last_frame_time = Instant::now();
        let mut frame = 0;
        let mut scanline_cycles = 0;
        loop {
            for scanline in 0..154 {
                eprintln!("frame {:>3} scanline {:>3}", frame, scanline);
                self.memory.set_ly(scanline);
                if scanline == 144 {
                    // request VBlank interrupt
                    let requests = self.memory.read8(0xFF0F) | 1;
                    self.memory.write8(0xFF0F, requests);
                }
                self.ppu.paint_line(&mut self.memory);
                while scanline_cycles < CPU_CYCLES_PER_SCANLINE {
                    self.cpu.step(&mut self.memory);
                    scanline_cycles += 4;
                    if self.handle_interrupts() {
                        scanline_cycles += 5 * 4;
                    }
                }
                scanline_cycles %= CPU_CYCLES_PER_SCANLINE;
            }
            frame += 1;
            let current_frame_time = Instant::now();
            let elapsed = current_frame_time.duration_since(last_frame_time);
            if let Some(sleep_duration) = frame_time.checked_sub(elapsed) {
                sleep(sleep_duration);
            }
            last_frame_time = current_frame_time;
            self.ppu.refresh(&mut self.emulator_window);
            if self.emulator_window.is_esc_pressed() {
                break;
            }
        }
    }

    fn handle_interrupts(&mut self) -> bool {
        self.cpu.handle_interrupts(&mut self.memory)
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
