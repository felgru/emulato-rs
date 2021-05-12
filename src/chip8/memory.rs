use std::io;
use std::io::Read;
use std::fs::File;
use std::ops::{Index, IndexMut};

use super::fonts::CHIP48_FONT;

const FONT_OFFSET: usize = 0x50;

pub struct Memory([u8; 4096]);

impl Default for Memory {
    fn default() -> Self {
        Self::with_font(&CHIP48_FONT)
    }
}

impl Memory {
    pub fn with_font(font: &[u8; 16 * 5]) -> Self {
        let mut memory = [0u8; 4096];
        memory[FONT_OFFSET..FONT_OFFSET + 16 * 5].copy_from_slice(font);
        Self(memory)
    }

    pub fn load_program_from_file(&mut self, mut f: File) -> io::Result<()> {
        let program_start_address = 0x200;
        f.read_exact(&mut self.0[program_start_address..])?;
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
