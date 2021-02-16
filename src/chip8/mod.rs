pub mod cpu;
pub mod display;
pub mod fonts;
pub mod memory;

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

    pub fn new(display_size: &str) -> Self {
        let (width, height) = match display_size {
            "64x32" => (64, 32),
            "128x64" => (128, 64),
            "64x128" => (64, 128),
            _ => panic!("Unexpected Chip8 display size: {}", display_size),
        };
        let memory =  memory::Memory::default();
        let display = display::Display::new(width, height, FRAMERATE);
        Self {
            cpu: cpu::CPU::default(),
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
