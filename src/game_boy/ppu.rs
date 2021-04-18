use std::collections::VecDeque;

use super::display;
use super::emulator_window::EmulatorWindow;
use super::graphics_data;
use super::memory::{LcdControl, MemoryBus};

pub struct PPU {
    mode: LcdMode,
    display: display::Display,
    tile_buffer: [u8; 21],
    bg_palette: [u8; 4],
}

impl PPU {
    pub fn new() -> Self {
        Self{
            mode: LcdMode::VBlank,
            display: display::Display::new(),
            tile_buffer: [0; 21],
            bg_palette: [0; 4],
        }
    }

    pub fn paint_line(&mut self, memory: &mut MemoryBus) {
        let ly = memory.ly();
        if ly >= 144 {
            // VBLANK line
            return;
        }
        let lcdc = memory.lcdc();
        // TODO: This only draws background, handle window and sprites
        if lcdc.is_window_enabled() {
            unimplemented!("Window drawing not implemented, yet!");
        }
        let (y, _) = ly.overflowing_add(memory.scy());
        let tile_y = y / 8;
        let in_tile_y = y % 8;
        let x_offset = memory.scx();
        let tile_x = x_offset / 8;
        let mut tile_pixel_index = 7 - (x_offset % 8);
        self.copy_tile_map_line(memory, lcdc, tile_y, tile_x);
        self.read_bg_palette(memory);
        let mut tile_iter = self.tile_buffer.iter();
        let mut tile_data = fetch_bg_tile_line(
            memory, lcdc, *tile_iter.next().unwrap(), in_tile_y);
        for pixel in self.display.line_buffer(ly).iter_mut() {
            let p = ((tile_data >> tile_pixel_index + 7) & 1)
                    | ((tile_data >> tile_pixel_index) & 1);
            *pixel = self.bg_palette[p as usize];
            if tile_pixel_index > 1 {
                tile_pixel_index -= 1;
            } else {
                tile_data = fetch_bg_tile_line(
                    memory, lcdc, *tile_iter.next().unwrap(), in_tile_y);
                tile_pixel_index = 7;
            }
        }
    }

    fn copy_tile_map_line(&mut self, memory: &MemoryBus,
                          lcdc: LcdControl, y: u8, x: u8) {
        let line_offset = lcdc.bg_tilemap_start() + 32 * y as u16;
        let mut x = x as u16;
        for tile in self.tile_buffer.iter_mut() {
            *tile = memory.read8(line_offset + x);
            x = (x + 1) % 32;
        }
    }

    fn read_bg_palette(&mut self, memory: &MemoryBus) {
        let palette = memory.bg_palette();
        self.bg_palette[0] = palette & 0b11;
        self.bg_palette[1] = (palette >> 2) & 0b11;
        self.bg_palette[2] = (palette >> 4) & 0b11;
        self.bg_palette[3] = (palette >> 6) & 0b11;
    }

    pub fn refresh(&self, window: &mut EmulatorWindow) {
        self.display.refresh(window);
    }
}

fn fetch_bg_tile_line(memory: &MemoryBus, lcdc: LcdControl, tile: u8,
                      in_tile_y: u8) -> u16 {
    let (offset, signed)
        = lcdc.bg_and_window_tile_data_offset_and_addressing();
    let orig_tile = tile;
    let tile = if signed {
        (offset as i16 + (tile as i8) as i16) as u16
    } else {
        offset + tile as u16
    };
    let low = tile + (2 * in_tile_y) as u16;
    let res = memory.read16(low);
    if orig_tile != 0 {
        eprintln!("drawing non-zero tile {:0>2X}: {:0>4X}@{:0>4X}",
               orig_tile, res, low);
    }
    res
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
