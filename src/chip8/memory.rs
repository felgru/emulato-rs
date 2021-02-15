use std::io;
use std::io::Read;
use std::fs::File;
use std::ops::{Index, IndexMut};

const FONT_OFFSET: usize = 0x50;

pub struct Memory([u8; 4096]);

impl Default for Memory {
    fn default() -> Self {
        let mut memory = [0u8; 4096];
        memory[FONT_OFFSET..FONT_OFFSET + 16 * 5]
            .copy_from_slice(&FONT_SPRITES);
        Self(memory)
    }
}

impl Memory {
    pub fn load_program_from_file(&mut self, mut f: File) -> io::Result<()> {
        let program_start_address = 0x200;
        f.read(&mut self.0[program_start_address..])?;
        Ok(())
    }

    pub fn read_slice(&self, from: u16, for_: u8) -> &[u8] {
        let a = from as usize;
        let b = a + for_ as usize;
        &self.0[a..b]
    }

    pub fn font_sprite_address(&self, character: u8) -> u16 {
        FONT_OFFSET as u16 + character as u16 * 5
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

const FONT_SPRITES: [u8; 16 * 5] = [
    // 0
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    // 1
    0x20, 0x60, 0x20, 0x20, 0x70,
    // 2
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    // 3
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    // 4
    0x90, 0x90, 0xF0, 0x10, 0x10,
    // 5
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    // 6
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    // 7
    0xF0, 0x10, 0x20, 0x40, 0x40,
    // 8
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    // 9
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    // A
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    // B
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    // C
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    // D
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    // E
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    // F
    0xF0, 0x80, 0xF0, 0x80, 0x80,
    ];
