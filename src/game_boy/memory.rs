use std::io;
use std::io::Read;
use std::fs::File;

use super::cartridge::Cartridge;
use super::ppu::LcdMode;

/// The memory bus of a Game Boy
///
/// Address layout:
/// 0x0000–0x3FFF  ROM0  Cartridge ROM bank 0
/// (0x0000–0x00FF  boot ROM)
/// 0x4000–0x7FFF  ROMX  Cartridge ROM bank X
/// 0x8000–0x9FFF  VRAM
/// (0x8000–0x97FF  Tile RAM)
/// (0x9800–0x9FFF  Background Map)
/// 0xA000–0xBFFF  SRAM  Cartridge RAM
/// 0xC000–0xCFFF  WRAM0  Working RAM
/// 0xD000–0xDFFF  WRAMX  Working RAM
/// 0xE000–0xFDFF  ECHO  echos Working RAM, discouraged to be used
/// 0xFE00–0xFE9F  OAM  Object Attribute Memory (description of sprites)
/// 0xFEA0–0xFEFF  UNUSED  (reading returns 0, writing does nothing)
/// 0xFF00–0xFF7F  I/O Registers
/// 0xFF80–0xFFFE  HRAM  High RAM Area (targetted by special load instructions)
/// 0xFFFF         IE Register  Interrupt Enabled Register
pub struct MemoryBus {
  memory: [u8; 0x10000],
  cartridge: Cartridge,
  boot_rom: Option<[u8; 0x100]>,
}

impl MemoryBus {
    pub fn new(cartridge: Cartridge, boot_rom: [u8; 0x100]) -> Self {
        Self{
            memory: [0; 0x10000],
            cartridge,
            boot_rom: Some(boot_rom),
        }
    }

    pub fn read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => { // ROM0
                if self.boot_rom.is_some() && address < 0x100 {
                    self.boot_rom.unwrap()[address as usize]
                } else {
                    self.cartridge.rom0_read8(address)
                }
            }
            0x4000..=0x7FFF => { // ROMX
                unimplemented!("reading from ROMX not implemented, yet.");
            }
            // 0xA000–0xBFFF  SRAM  Cartridge RAM
            // 0xC000–0xCFFF  WRAM0  Working RAM
            // 0xD000–0xDFFF  WRAMX  Working RAM
            // 0xE000–0xFDFF  ECHO  echos Working RAM, discouraged to be used
            // 0xFE00–0xFE9F  OAM  Object Attribute Memory (description of sprites)
            // 0xFEA0–0xFEFF  UNUSED  (reading returns 0, writing does nothing)
            // 0xFF00–0xFF7F  I/O Registers
            0x8000..=0x9FFF => {
                // 0x8000–0x9FFF  VRAM
                // (0x8000–0x97FF  Tile RAM)
                // (0x9800–0x9FFF  Background Map)
                self.memory[address as usize]
            }
            0xA000..=0xFF0E => {
                unimplemented!("reading from {:0>4X} not implemented, yet.",
                               address);
            }
            0xFF0F => { // IF – Interrupt Flag
                self.memory[address as usize]
            }
            0xFF10..=0xFF41 => {
                unimplemented!("reading from {:0>4X} not implemented, yet.",
                               address);
            }
            0xFF42..=0xFF4B => { // LCD Position and scrolling
                // FF42 - SCY (Scroll Y) (R/W)
                // FF43 - SCX (Scroll X) (R/W)
                // FF44 - LY (LCDC Y-Coordinate) (R)
                // FF45 - LYC (LY Compare) (R/W)
                // FF4A - WY (Window Y Position) (R/W)
                // FF4B - WX (Window X Position + 7) (R/W)
                self.memory[address as usize]
            }
            0xFF4C..=0xFF7F => { // I/O Registers
                unimplemented!("reading from {:0>4X} not implemented, yet.",
                               address);
            }
            0xFF80..=0xFFFE => { // HRAM
                self.memory[address as usize]
            }
            0xFFFF => { // Interrupt Enabled Register
                self.memory[address as usize]
            }
        }
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => { // ROM
                unimplemented!("writing to ROM not implemented.");
            }
            // 0x8000–0x9FFF  VRAM
            // (0x8000–0x97FF  Tile RAM)
            // (0x9800–0x9FFF  Background Map)
            // 0xA000–0xBFFF  SRAM  Cartridge RAM
            // 0xC000–0xCFFF  WRAM0  Working RAM
            // 0xD000–0xDFFF  WRAMX  Working RAM
            0x8000..=0xDFFF => {
                self.memory[address as usize] = value;
            }
            0xE000..=0xFDFF => { // Echo
                unimplemented!("Writing to Echo not implemented.");
            }
            0xFE00..=0xFE9F => { // OAM
                self.memory[address as usize] = value;
            }
            0xFEA0..=0xFEFF => { // UNUSED
                // write does nothing
            }
            0xFF00..=0xFF7F => { // I/O Registers
                match address {
                    0xFF0F => { // IF – Interrupt Flag
                        self.memory[address as usize] = value;
                    }
                    0xFF10..=0xFF26 => { // Sound
                        // TODO: ignoring sound for now
                    }
                    0xFF40 => {
                        // LCD Control
                        // TODO: handle flags to set LCD state
                        // https://gbdev.io/pandocs/#lcd-control
                        self.memory[address as usize] = value;
                        // unimplemented!("LCDC = {:0>4X}", value);
                    }
                    0xFF41 => { // LCD Status
                        self.memory[address as usize] = value;
                    }
                    0xFF44 => { // LY (LCDC Y-Coordinate) (R)
                        panic!("Trying to write to LY");
                    }
                    0xFF42..=0xFF45 => {
                        // LCD Position and scrolling
                        self.memory[address as usize] = value;
                    }
                    0xFF46 => {
                        // Object Attribute Memory (OAM) DMA Control Register
                        unimplemented!("Writing {:0>4X} to OAM DMA register.",
                                       value);
                    }
                    0xFF47..=0xFF49 => {
                        // 0xFF47: BGP (BG Palette Data)
                        // 0xFF48: OBP0 (Object Palette 0 Data)
                        // 0xFF49: OBP1 (Object Palette 1 Data)
                        self.memory[address as usize] = value;
                    }
                    0xFF4A..=0xFF4B => {
                        // LCD Position and scrolling (continued)
                        self.memory[address as usize] = value;
                    }
                    0xFF50 => { // Disable boot ROM flag
                        if value & 1 != 0 {
                            self.disable_boot_rom();
                        }
                        self.memory[address as usize] = value;
                    }
                    _ => unimplemented!("Writing to I/O register {:0>4X} not implemented.",
                                        address),
                }
            }
            0xFF80..=0xFFFE => { // HRAM
                self.memory[address as usize] = value;
            }
            0xFFFF => { // IE Register
                if value == 1 { // VBLANK
                    self.memory[address as usize] = value;
                } else {
                    unimplemented!(
                        "Writing non-VBlank to IE register not implemented.");
                }
            }
        }
    }

    pub fn read16(&self, address: u16) -> u16 {
        self.read8(address) as u16 + ((self.read8(address+1) as u16) << 8)
    }

    pub fn write16(&mut self, address: u16, value: u16) {
        self.write8(address, value as u8);
        self.write8(address+1, (value >> 8) as u8);
    }

    pub fn lcdc(&self) -> LcdControl {
        LcdControl{flags: self.memory[0xFF40]}
    }

    pub fn lcd_status(&self) -> LcdStatus {
        LcdStatus{flags: self.memory[0xFF41]}
    }

    pub fn scy(&self) -> u8 {
        self.memory[0xFF42]
    }

    pub fn scx(&self) -> u8 {
        self.memory[0xFF43]
    }

    pub fn ly(&self) -> u8 {
        self.memory[0xFF44]
    }

    pub fn set_ly(&mut self, ly: u8) {
        self.memory[0xFF44] = ly;
    }

    pub fn lyc(&self) -> u8 {
        self.memory[0xFF45]
    }

    pub fn wy(&self) -> u8 {
        self.memory[0xFF4A]
    }

    pub fn wx(&self) -> u8 {
        self.memory[0xFF4B]
    }

    pub fn bg_palette(&self) -> u8 {
        self.memory[0xFF47]
    }

    pub fn load_boot_rom(mut file: File) -> io::Result<[u8; 0x100]> {
        let mut rom = [0; 0x100];
        file.read(&mut rom)?;
        Ok(rom)
    }

    pub fn disable_boot_rom(&mut self) {
        self.boot_rom = None;
    }

    pub fn dump_tile_data(&self) {
        let tile_size = 2 * 8;
        let num_tiles = (0x9800 - 0x8000) / tile_size;
        // print tile data in rows of 16 tiles each
        let tiles_per_row = 16;
        let num_rows = num_tiles / tiles_per_row;
        println!("P2");
        println!("{} {}", tiles_per_row*8, num_rows*8);
        println!("3");
        let mut row_start = 0x8000;
        for tile_row in 0..num_rows {
            for row in 0..8 {
                for tile_col in 0..tiles_per_row {
                    let tile = self.read16(row_start + 2 * row
                                           + tile_size * tile_col);
                    let p = ((tile >> 14) & 0b10) | ((tile >> 7) & 1);
                    print!("{}", p);
                    for i in 1..8 {
                        let p = ((tile >> 14-i) & 0b10) | ((tile >> 7-i) & 1);
                        print!(" {}", p);
                    }
                    println!();
                }
            }
            row_start += tiles_per_row * tile_size;
        }
    }

    pub fn dump_bg(&self) {
        let lcdc = self.lcdc();
        let palette = Self::expand_palette(self.bg_palette());
        let (data_start, signed)
            = lcdc.bg_and_window_tile_data_offset_and_addressing();
        let tile_map_start = lcdc.bg_tilemap_start();
        println!("P2");
        println!("256 256");
        println!("3");
        let mut tiles: [u8; 32] = [0; 32];
        let tile_size = 16;
        for tile_row in 0..32 {
            for tile_col in 0..32u16 {
                let tile
                    = self.read8(tile_map_start + 32 * tile_row + tile_col);
                tiles[tile_col as usize] = tile;
                eprintln!("{:0>4X}", tile);
            }
            for row in 0..8 {
                for tile_col in 0..32 {
                    let tile_address = if signed {
                        (data_start as i16
                         + (tiles[tile_col] as i8) as i16 * tile_size as i16) as u16
                    } else {
                        data_start + tiles[tile_col] as u16 * tile_size
                    };
                    let tile = self.read16(tile_address + 2 * row);
                    let p = ((tile >> 14) & 0b10) | ((tile >> 7) & 1);
                    print!("{}", p);
                    for i in 1..8 {
                        let p = ((tile >> 14-i) & 0b10) | ((tile >> 7-i) & 1);
                        print!(" {}", p);
                    }
                    println!();
                }
            }
        }
    }

    fn expand_palette(palette: u8) -> [u8; 4] {
        [palette & 0b11,
         (palette >> 2) & 0b11,
         (palette >> 4) & 0b11,
         (palette >> 6) & 0b11]
    }

    pub fn handle_interrupts(&mut self) -> Option<InterruptAddress> {
        let mut requests = self.read8(0xFF0F);
        let interrupts = self.read8(0xFFFF) & requests & 0x1F;
        if interrupts == 0 {
            return None;
        }
        let next_interrupt = interrupts.trailing_zeros();
        requests ^= 1 << next_interrupt;
        self.write8(0xFF0F, requests);
        use InterruptAddress::*;
        let address = match next_interrupt {
            0 => VBLANK,
            1 => LCD_STAT,
            2 => TIMER,
            3 => SERIAL,
            4 => JOYPAD,
            _ => unreachable!(),
        };
        Some(address)
    }
}

/// LCD Control flags
///
/// https://gbdev.io/pandocs/#lcd-control
/// Bit 	Name 	                Usage notes
///   7 	LCD and PPU enable 	0=Off, 1=On
///   6 	Window tile map area 	0=9800-9BFF, 1=9C00-9FFF
///   5 	Window enable 		0=Off, 1=On
///   4 	BG and Window tile data area 	0=8800-97FF, 1=8000-8FFF
///   3 	BG tile map area 	0=9800-9BFF, 1=9C00-9FFF
///   2 	OBJ size 		0=8x8, 1=8x16
///   1 	OBJ enable 		0=Off, 1=On
///   0 	BG and Window enable/priority 	0=Off, 1=On
#[derive(Copy, Clone)]
pub struct LcdControl {
    flags: u8,
}

impl LcdControl {
    pub fn are_lcd_and_ppu_enabled(&self) -> bool {
        self.flags & 128 != 0
    }

    pub fn window_tilemap_start(&self) -> u16 {
        if self.flags & 64 == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    pub fn is_window_enabled(&self) -> bool {
        self.flags & 32 != 0
    }

    /// Tile data offset and signedness of addressing
    ///
    /// https://gbdev.io/pandocs/#vram-tile-data
    pub fn bg_and_window_tile_data_offset_and_addressing(&self) -> (u16, bool) {
        if self.flags & 16 == 0 {
            (0x8800, true)
        } else {
            (0x8000, false)
        }
    }

    pub fn bg_tilemap_start(&self) -> u16 {
        if self.flags & 8 == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    pub fn obj_height(&self) -> u16 {
        // OBJ width is always 8
        if self.flags & 4 == 0 {
            8
        } else {
            16
        }
    }

    pub fn is_obj_enabled(&self) -> bool {
        self.flags & 2 != 0
    }

    pub fn is_bg_and_window_enabled(&self) -> bool {
        self.flags & 1 != 0
    }
}

/// LCD Status flags
///
/// https://gbdev.io/pandocs/#lcd-status-register
/// Bit - description
///   7 - -unused-
///   6 - LYC=LY Interrupt             (1=Enable) (Read/Write)
///   5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
///   4 - Mode 1 VBlank Interrupt      (1=Enable) (Read/Write)
///   3 - Mode 0 HBlank Interrupt      (1=Enable) (Read/Write)
///   2 - LYC=LY Flag      (0=Different, 1=Equal) (Read Only)
///   1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
///         0: In HBlank
///         1: In VBlank
///         2: Searching OAM
///         3: Transferring Data to LCD Controller
pub struct LcdStatus {
    flags: u8,
}

impl LcdStatus {
    fn lyc_eq_ly_interrupt_set(&self) -> bool {
        self.flags & (1 << 6) != 0
    }

    fn mode2_oam_interrupt_set(&self) -> bool {
        self.flags & (1 << 5) != 0
    }

    fn mode1_vblank_interrupt_set(&self) -> bool {
        self.flags & (1 << 4) != 0
    }

    fn mode0_hblank_interrupt_set(&self) -> bool {
        self.flags & (1 << 3) != 0
    }

    fn lyc_eq_ly(&self) -> bool {
        self.flags & (1 << 2) != 0
    }

    fn mode(&self) -> LcdMode {
        (self.flags & 3).into()
    }
}

#[repr(u16)]
#[derive(Debug)]
pub enum InterruptAddress {
    VBLANK = 0x40,
    LCD_STAT = 0x48,
    TIMER = 0x50,
    SERIAL = 0x58,
    JOYPAD = 0x60,
}
