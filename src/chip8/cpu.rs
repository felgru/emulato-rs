use std::fmt;
use std::ops::{Index, IndexMut};
use super::display::Display;
use super::memory::Memory;

pub struct CPU {
    pc: u16,
    registers: Registers,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            pc: 0x200,
            registers: Registers::default(),
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

impl CPU {
    pub fn tick(&mut self, memory: &mut Memory, display: &mut Display) {
        let pc = &mut self.pc;
        let opcode: u16 = ((memory[*pc] as u16) << 8) + memory[*pc + 1] as u16;
        // eprintln!("{:0>3X}: {:0>4X}", pc, opcode);
        // eprintln!("{}", self.registers);
        *pc += 2;
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    0x00E0 => display.clear(),
                    0x00EE => {
                        *pc = match self.stack.pop() {
                            Some(nnn) => nnn,
                            None => panic!("Stack underflow at {:X}", *pc - 2),
                        }
                    }
                    _ => panic!("Unknown opcode: {:X}", opcode),
                }
            }
            0x1000 => {
                let nnn = opcode & 0xFFF;
                *pc = nnn;
            }
            0x2000 => {
                let nnn = opcode & 0xFFF;
                self.stack.push(*pc);
                *pc = nnn;
            }
            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let nn = opcode as u8;
                if self.registers[x] == nn {
                    *pc += 2;
                }
            }
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let nn = opcode as u8;
                if self.registers[x] != nn {
                    *pc += 2;
                }
            }
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                let n = (opcode & 0x000F) as u8;
                if n != 0 {
                    panic!("Unknown opcode: {:X}", opcode);
                }
                if self.registers[x] == self.registers[y] {
                    *pc += 2;
                }
            }
            0x6000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let nn = opcode as u8;
                self.registers[x] = nn;
            }
            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let nn = opcode as u8;
                let (new, _carry) = self.registers[x].overflowing_add(nn);
                self.registers[x] = new;
            }
            0x8000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                let n = (opcode & 0x000F) as u8;
                let y = self.registers[y];
                match n {
                    0x0000 => {
                        self.registers[x] = y;
                    }
                    0x0001 => {
                        self.registers[x] |= y;
                    }
                    0x0002 => {
                        self.registers[x] &= y;
                    }
                    0x0003 => {
                        self.registers[x] ^= y;
                    }
                    0x0004 => {
                        let (new, carry) = self.registers[x].overflowing_add(y);
                        self.registers[x] = new;
                        self.registers[0xF] = carry as u8;
                    }
                    0x0005 => {
                        let (new, carry) = self.registers[x].overflowing_sub(y);
                        self.registers[x] = new;
                        self.registers[0xF] = !carry as u8;
                    }
                    0x0006 => {
                        // TODO: What is the expected behavior when X, Y = F?
                        self.registers[0xF] = y & 1;
                        self.registers[x] = y >> 1;
                    }
                    0x0007 => {
                        let x_old = self.registers[x];
                        let (new, carry) = y.overflowing_sub(x_old);
                        self.registers[x] = new;
                        self.registers[0xF] = !carry as u8;
                    }
                    0x000E => {
                        // TODO: What is the expected behavior when X, Y = F?
                        self.registers[0xF] = y & (1 << 7);
                        self.registers[x] = y << 1;
                    }
                    _ => {
                        panic!("Unknown opcode: {:X}", opcode);
                    }
                }
            }
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                let n = (opcode & 0x000F) as u8;
                if n != 0 {
                    panic!("Unknown opcode: {:X}", opcode);
                }
                if self.registers[x] != self.registers[y] {
                    *pc += 2;
                }
            }
            0xA000 => {
                let nnn = opcode & 0xFFF;
                self.registers.write_i(nnn);
            }
            0xB000 => {
                let nnn = opcode & 0xFFF;
                *pc = nnn + self.registers[0] as u16;
            }
            0xD000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                let n = (opcode & 0x000F) as u8;
                let sprite = memory.read_slice(self.registers.i, n);
                let x = self.registers[x];
                let y = self.registers[y];
                // eprintln!("draw {:X} {:X} = {:#X?} ({})", x, y, sprite, n);
                self.registers[0xF] = display.draw_sprite(x, y, sprite);
            }
            0xF000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                match opcode & 0x00FF {
                    0x001E => {
                        let i = self.registers.read_i();
                        self.registers.write_i(i + self.registers[x] as u16);
                    }
                    0x0033 => {
                        let i = self.registers.read_i();
                        use std::io::Write;
                        let mut buf = [0u8; 3];
                        write!(&mut buf[..], "{:0>3}", self.registers[x])
                            .unwrap();
                        for j in 0..3 {
                            memory[i + j] = buf[j as usize] - 0x30;
                        }
                    }
                    0x0055 => {
                        let mut i = self.registers.read_i();
                        for r in 0..x+1 {
                            memory[i] = self.registers[r];
                            i += 1;
                        }
                        self.registers.write_i(i);
                    }
                    0x0065 => {
                        let mut i = self.registers.read_i();
                        for r in 0..x+1 {
                            self.registers[r] = memory[i];
                            i += 1;
                        }
                        self.registers.write_i(i);
                    }
                    _ => {
                        panic!("Unknown opcode: {:X}", opcode);
                    }
                }
            }
            _ => panic!("Unknown opcode: {:X}", opcode),
        }
    }
}

pub struct Opcode([u8; 2]);

struct Registers {
    i: u16,
    v: [u8; 16],
}

impl Default for Registers {
    fn default() -> Self {
        Self{
            i: 0,
            v: [0; 16],
        }
    }
}

impl Registers {
    pub fn read_i(&self) -> u16 {
        self.i
    }

    pub fn write_i(&mut self, i: u16) {
        self.i = i
    }
}

impl Index<u8> for Registers {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.v[index as usize]
    }
}

impl IndexMut<u8> for Registers {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.v[index as usize]
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " 0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F   I\n")?;
        for i in 0..0xF {
          write!(f, "{:0>2X} ", self[i])?;
        }
        write!(f, "{:0>2X} {:0>3X}", self[0xF], self.read_i())?;
        Ok(())
    }
}
