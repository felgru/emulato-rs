use std::collections::VecDeque;

use super::display;
use super::graphics_data;

pub struct PPU {
    mode: LcdMode,
    lyc: u8,
    fifo: PixelFifo,
}

impl PPU {
    pub fn new() -> Self {
        Self{
            mode: LcdMode::VBlank,
            lyc: 0,
            fifo: PixelFifo::new(),
        }
    }

    pub fn update(&mut self) {
        // TODO
    }
}

/// The Pixel FIFO
///
/// https://gbdev.io/pandocs/#pixel-fifo
struct PixelFifo {
    background: VecDeque<u8>,
    sprite: VecDeque<u8>,
}

impl PixelFifo {
    pub fn new() -> Self {
        Self{
            background: VecDeque::new(),
            sprite: VecDeque::new(),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum LcdMode {
    HBlank = 0,
    VBlank = 1,
    SearchingOAM = 2,
    TransferringDataToLcdController = 3,
}

impl From<u8> for LcdMode {
    fn from(v: u8) -> Self {
        use LcdMode::*;
        match v {
            0 => HBlank,
            1 => VBlank,
            2 => SearchingOAM,
            3 => TransferringDataToLcdController,
            _ => panic!("{:X} is not a valid LcdMode.", v),
        }
    }
}
