// SPDX-FileCopyrightText: 2021 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::VecDeque;

use super::display;
use super::io::IO;
use super::memory::{LcdControl, MemoryBus};

pub struct PPU {
    display: display::Display,
    tile_buffer: [u8; 21],
    bg_palette: [u8; 4],
}

impl PPU {
    pub fn new() -> Self {
        Self{
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
        if lcdc.is_bg_and_window_enabled() {
            self.paint_background_line(memory, lcdc, ly);
            if lcdc.is_window_enabled() {
                self.paint_window_line(memory, lcdc, ly);
            }
        }
        if lcdc.is_obj_enabled() {
            self.paint_obj_line(memory, ly);
        }
    }

    fn paint_background_line(&mut self, memory: &mut MemoryBus,
                             lcdc: LcdControl, ly: u8) {
        let (y, _) = ly.overflowing_add(memory.scy());
        let tile_y = y / 8;
        let in_tile_y = y % 8;
        let x_offset = memory.scx();
        let tile_x = x_offset / 8;
        let mut tile_pixel_index = 7 - (x_offset % 8);
        self.copy_tile_map_line(memory, lcdc.bg_tilemap_start(),
                                tile_y, tile_x);
        self.read_bg_palette(memory);
        let mut tile_iter = self.tile_buffer.iter();
        let mut tile_data = fetch_bg_tile_line(
            memory, lcdc, *tile_iter.next().unwrap(), in_tile_y);
        for pixel in self.display.line_buffer(ly).iter_mut() {
            let p = ((tile_data >> (tile_pixel_index + 7)) & 0b10)
                    | ((tile_data >> tile_pixel_index) & 1);
            *pixel = self.bg_palette[p as usize];
            if tile_pixel_index > 0 {
                tile_pixel_index -= 1;
            } else {
                tile_data = fetch_bg_tile_line(
                    memory, lcdc, *tile_iter.next().unwrap(), in_tile_y);
                tile_pixel_index = 7;
            }
        }
    }

    fn paint_window_line(&mut self, memory: &mut MemoryBus,
                         lcdc: LcdControl, ly: u8) {
        let wy = memory.wy();
        let wx = memory.wx();
        if ly < wy || wx > 166 {
            return;
        }
        if wx < 7 || wx == 166 {
            eprintln!("Window hardware bugs for WX = {} not implemented.", wx);
        }
        let y = ly - wy;
        let tile_y = y / 8;
        let in_tile_y = y % 8;
        let (x_offset, mut tile_pixel_index) = if wx >= 7 {
            (wx - 7, 7)
        } else {
            (0, wx)
        };
        self.copy_tile_map_line(memory, lcdc.window_tilemap_start(),
                                tile_y, 0);
        self.read_bg_palette(memory);
        let mut tile_iter = self.tile_buffer.iter();
        let mut tile_data = fetch_bg_tile_line(
            memory, lcdc, *tile_iter.next().unwrap(), in_tile_y);
        for pixel in self.display.line_buffer(ly)[x_offset as usize..]
                                 .iter_mut() {
            let p = ((tile_data >> (tile_pixel_index + 7)) & 0b10)
                    | ((tile_data >> tile_pixel_index) & 1);
            *pixel = self.bg_palette[p as usize];
            if tile_pixel_index > 0 {
                tile_pixel_index -= 1;
            } else {
                tile_data = fetch_bg_tile_line(
                    memory, lcdc, *tile_iter.next().unwrap(), in_tile_y);
                tile_pixel_index = 7;
            }
        }
    }

    fn paint_obj_line(&mut self, memory: &mut MemoryBus, ly: u8) {
        let lcdc = memory.lcdc();
        let obj_height = lcdc.obj_height();
        let mut sprites: Vec<Sprite> = Vec::with_capacity(10);
        for address in (0xFE00..0xFEA0).step_by(4) {
            let y = memory.read8(address);
            if ly + 16 < y || ly + 16 >= y + obj_height {
                // OBJ outside ly
                continue;
            }
            let obj_line = ly + 16 - y;
            let x = memory.read8(address+1);
            let tile_index = memory.read8(address+2);
            let attribute_flags = memory.read8(address+3);
            sprites.push(Sprite::new(obj_line, x, tile_index, attribute_flags));
            if sprites.len() == 10 {
                break;
            }
        }
        if sprites.is_empty() {
            return;
        }
        let palettes = [memory.obj_palette0().as_array(),
                        memory.obj_palette1().as_array()];
        // sort sprites by priority
        sprites.sort_by(|a, b| {a.x().cmp(&b.x())});
        let pixels = self.display.line_buffer(ly);
        for sprite in sprites {
            // TODO: Correctly handle overlapping sprites
            let attributes = sprite.attribute_flags();
            let y = if attributes.y_flip() {
                obj_height - 1 - sprite.y()
            } else {
                sprite.y()
            };
            let tile = fetch_obj_tile_line(memory, sprite.tile_index(),
                                           y, obj_height == 16);
            let palette = palettes[attributes.palette()];
            let x = sprite.x();
            let i_min = if x > 160 {
                x - 160
            } else {
                0
            };
            let i_max = std::cmp::min(x, 8);
            for i in i_min..i_max {
                let x = (x - 1 - i) as usize;
                let i = if attributes.x_flip() {
                    7 - i
                } else {
                    i
                };
                let p = ((tile >> (i + 7)) & 0b10)
                        | ((tile >> i) & 1);
                if p > 0 {
                    pixels[x] = palette[p as usize];
                }
            }
        }
    }

    fn copy_tile_map_line(&mut self, memory: &MemoryBus,
                          tilemap_start: u16, y: u8, x: u8) {
        let line_offset = tilemap_start + 32 * y as u16;
        let mut x = x as u16;
        for tile in self.tile_buffer.iter_mut() {
            *tile = memory.read8(line_offset + x);
            x = (x + 1) % 32;
        }
    }

    fn read_bg_palette(&mut self, memory: &MemoryBus) {
        self.bg_palette = memory.bg_palette().as_array();
    }

    pub fn refresh<Window: IO>(&self, window: &mut Window) {
        self.display.refresh(window);
    }
}

fn fetch_bg_tile_line(memory: &MemoryBus, lcdc: LcdControl, tile: u8,
                      in_tile_y: u8) -> u16 {
    let tile = lcdc.get_bg_or_window_tile_address(tile);
    let low = tile + (2 * in_tile_y) as u16;
    memory.read16(low)
}

fn fetch_obj_tile_line(memory: &MemoryBus, tile: u8,
                       in_tile_y: u8, double_sized: bool) -> u16 {
    let (tile, in_tile_y) = if double_sized {
        if in_tile_y < 8 {
            (tile & 0xFE, in_tile_y)
        } else {
            (tile | 1, in_tile_y - 8)
        }
    } else {
        (tile, in_tile_y)
    };
    let low = 0x8000 + 16 * tile as u16 + (2 * in_tile_y) as u16;
    memory.read16(low)
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

pub struct Sprite {
    y: u8,
    x: u8,
    tile_index: u8,
    attribute_flags: ObjAttributeFlags,
}

impl Sprite {
    pub fn new(y: u8, x: u8, tile_index: u8, attribute_flags: u8) -> Self {
        let attribute_flags = ObjAttributeFlags(attribute_flags);
        Sprite{y, x, tile_index, attribute_flags}
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn tile_index(&self) -> u8 {
        self.tile_index
    }

    pub fn attribute_flags(&self) -> ObjAttributeFlags {
        self.attribute_flags
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ObjAttributeFlags(u8);

impl ObjAttributeFlags {
    fn palette(self) -> usize {
        ((self.0 >> 4) & 1) as usize
    }

    fn x_flip(self) -> bool {
        (self.0 & 0x20) != 0
    }

    fn y_flip(self) -> bool {
        (self.0 & 0x40) != 0
    }

    fn bg_and_window_over_obj(self) -> bool {
        (self.0 & 0x80) != 0
    }
}
