pub mod cpu;
pub mod display;
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
    pub fn new() -> Self {
        let memory =  memory::Memory::default();
        let display = display::Display::with_refresh_rate(FRAMERATE);
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
