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
  joypad: u8,
}

impl MemoryBus {
    pub fn new(cartridge: Cartridge, boot_rom: [u8; 0x100]) -> Self {
        Self{
            memory: [0; 0x10000],
            cartridge,
            boot_rom: Some(boot_rom),
            joypad: 0,
        }
    }

    pub fn read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x00FF => { // Boot ROM / ROM Bank 0
                if let Some(ref boot_rom) = self.boot_rom {
                    boot_rom[address as usize]
                } else {
                    self.cartridge.rom0_read8(address)
                }
            }
            0x0100..=0x7FFF | 0xA000..=0xBFFF => { // Cartridge
                // 0x0000–0x3FFF  ROM Bank 0
                // 0x4000–0x7FFF  ROM X (switchable via Memory Controller)
                // 0xA000–0xBFFF  SRAM  Cartridge RAM
                self.cartridge.read8(address)
            }
            0x8000..=0x9FFF => {
                // 0x8000–0x9FFF  VRAM
                // (0x8000–0x97FF  Tile RAM)
                // (0x9800–0x9FFF  Background Map)
                self.memory[address as usize]
            }
            0xC000..=0xDFFF => { // Working RAM
                // 0xC000–0xCFFF  WRAM0  Working RAM
                // 0xD000–0xDFFF  WRAMX  Working RAM (switchable banks on GBC)
                self.memory[address as usize]
            }
            // 0xE000–0xFDFF  ECHO  echos Working RAM, discouraged to be used
            0xE000..=0xFDFF => {
                unimplemented!("reading from ECHO {:0>4X} not implemented, yet.",
                               address);
            }
            // 0xFE00–0xFE9F  OAM  Object Attribute Memory (description of sprites)
            0xFE00..=0xFE9F => {
                self.memory[address as usize]
            }
            // 0xFEA0–0xFEFF  UNUSED  (reading returns 0, writing does nothing)
            0xFEA0..=0xFEFF => {
                unimplemented!("reading from UNUSED {:0>4X} not implemented, yet.",
                               address);
            }
            // 0xFF00–0xFF7F  I/O Registers
            0xFF00 => { // Joypad
                self.memory[address as usize]
            }
            0xFF01..=0xFF03 => {
                unimplemented!("reading from {:0>4X} not implemented, yet.",
                               address);
            }
            0xFF04..=0xFF07 => { // Timer and Divider Registers
                // 0xFF04  DIV – Divider Register
                // 0xFF05  TIMA – Timer Counter
                // 0xFF06  TMA – Timer Modulo
                // 0xFF07  TAC – Timer Control
                self.memory[address as usize]
            }
            0xFF08..=0xFF0E => {
                unimplemented!("reading from {:0>4X} not implemented, yet.",
                               address);
            }
            0xFF0F => { // IF – Interrupt Flag
                self.memory[address as usize]
            }
            0xFF10..=0xFF26 => { // Sound
                // TODO: ignoring sound for now
                0x00
            }
            0xFF27..=0xFF2F => {
                unimplemented!("reading from {:0>4X} not implemented, yet.",
                               address);
            }
            0xFF30..=0xFF3F => { // Wave Form RAM
                // TODO: ignoring sound for now
                0x00
            }
            0xFF40..=0xFF4B => { // LCD Status
                // FF40 - LCD Control (R/W)
                // FF41 - LCD Status (R/W)
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
            0x0000..=0x7FFF => { // Cartridge ROM
                self.cartridge.write8(address, value);
            }
            0x8000..=0x9FFF => { // VRAM
                // (0x8000–0x97FF  Tile RAM)
                // (0x9800–0x9FFF  Background Map)
                self.memory[address as usize] = value;
            }
            0xA000..=0xBFFF => { // SRAM  Cartridge RAM
                self.cartridge.write8(address, value);
            }
            0xC000..=0xDFFF => { // Working RAM
                // 0xC000–0xCFFF  WRAM0  Working RAM
                // 0xD000–0xDFFF  WRAMX  Working RAM (switchable banks on GBC)
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
                    0xFF00 => { // Joypad
                        self.memory[address as usize] = value;
                        self.update_joypad_register();
                    }
                    0xFF01..=0xFF02 => { // Serial Transfer
                        // 0xFF01  SB – Serial Transfer Data
                        // 0xFF02  SC – Serial Transfer Control
                        self.memory[address as usize] = value;
                    }
                    0xFF04 => { // DIV – Divider Register
                        // Writing any value to DIV register resets it to 0.
                        // https://gbdev.io/pandocs/#ff04-div-divider-register-r-w
                        self.memory[address as usize] = 0;
                    }
                    0xFF05..=0xFF07 => {
                        // 0xFF05 TIMA – Timer Counter
                        // 0xFF06 TMA – Timer Modulo
                        // 0xFF07 TAC – Timer Control
                        self.memory[address as usize] = value;
                    }
                    0xFF0F => { // IF – Interrupt Flag
                        self.memory[address as usize] = value;
                    }
                    0xFF10..=0xFF26 => { // Sound
                        // TODO: ignoring sound for now
                    }
                    0xFF30..=0xFF3F => { // Wave Form RAM
                        // TODO: ignoring sound for now
                    }
                    0xFF40 => {
                        // LCD Control
                        // https://gbdev.io/pandocs/#lcd-control
                        // TODO: Swith display on/off according to bit 7.
                        // assert!(value & 0x80 != 0,
                        //         "Switching off LCD not handled.");
                        self.memory[address as usize] = value;
                        // unimplemented!("LCDC = {:0>4X}", value);
                    }
                    0xFF41 => { // LCD Status
                        // lowest 3 bits are read-only
                        let value = value & !0x07;
                        self.memory[address as usize] &= 0x07;
                        self.memory[address as usize] |= value;
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
                        // TODO: This is a simplification of the DMA transfer.
                        //       Normally it would take 160 cycles during which
                        //       the CPU continues execution but only has
                        //       access to HRAM.
                        eprintln!("OAM transfer from {0:0>2X}00–{0:0>2X}9F.",
                                  value);
                        let source_start = (value as u16) << 8;
                        for (source, target)
                            in (source_start..=source_start+0x9F)
                                .zip(0xFE00..=0xFE9F) {
                            let value = self.read8(source);
                            self.write8(target, value);
                        }
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
                    0xFF72..=0xFF7F => { // Undocumented I/O registers
                        // TODO: Improve handling of undocumented I/O registers
                        eprintln!("Writing {:0>2X} to undocumented I/O register {:0>4X}.",
                                  value, address);
                    }
                    _ => unimplemented!("Writing {:0>2X} to I/O register {:0>4X} not implemented.",
                                        value, address),
                }
            }
            0xFF80..=0xFFFE => { // HRAM
                self.memory[address as usize] = value;
            }
            0xFFFF => { // IE Register
                if value & !0x13 != 0 {
                    unimplemented!(
                        "Writing {:0>2X} to IE register not implemented.",
                        value);
                }
                self.memory[address as usize] = value;
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

    pub fn lcd_status(&mut self) -> LcdStatus {
        LcdStatus{flags: &mut self.memory[0xFF41]}
    }

    pub fn set_lcd_mode(&mut self, mode: LcdMode) {
        if self.lcd_status().set_mode(mode) {
            // Request Stat interrupt.
            self.memory[0xFFFF] |= 2;
        }
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
        let equal = ly == self.lyc();
        self.lcd_status().update_lyc_eq_ly(equal);
        if equal && self.lcd_status().lyc_eq_ly_interrupt_set() {
            // Request Stat interrupt.
            self.memory[0xFFFF] |= 2;
        }
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

    pub fn obj_palette0(&self) -> u8 {
        self.memory[0xFF48]
    }

    pub fn obj_palette1(&self) -> u8 {
        self.memory[0xFF49]
    }

    /// Set pressed JoyPad keys
    ///
    /// Keypresses are given as a bitmap with 1 bit per button,
    /// which is 1 if pressed and 0 if unpressed.
    ///
    /// Bit  Button
    /// ---  -------
    /// 0    Right
    /// 1    Left
    /// 2    Up
    /// 3    Down
    /// 4    A
    /// 5    B
    /// 6    Select
    /// 7    Start
    pub fn set_key_presses(&mut self, presses: u8) -> bool {
        self.joypad = presses;
        self.update_joypad_register()
    }

    fn update_joypad_register(&mut self) -> bool {
        let joypad_register = self.memory[0xFF00];
        // Careful: joypad_register stores pressed buttons as 0,
        //          but joypad stores them as 1.
        let mut joypad = 0;
        if (joypad_register & 0x10) == 0 { // Direction keys
            joypad |= self.joypad & 0x0F;
        }
        if (joypad_register & 0x20) == 0 { // Action keys
            joypad |= (self.joypad >> 4) & 0x0F;
        }
        self.memory[0xFF00] = joypad_register & 0x30 | (!joypad & 0x0F);
        if joypad != 0 {
            eprintln!("Joypad register: {:0>2X}", self.memory[0xFF00]);
        }
        // Raise interrup when "unpressed button" bits become
        // "pressed button bits"
        if ((joypad_register & 0x0F) & joypad) != 0 {
            // Request Joypad interrupt
            self.memory[0xFFFF] |= 1 << 4;
            true
        } else {
            false
        }
    }

    pub fn disable_boot_rom(&mut self) {
        self.boot_rom = None;
    }

    pub fn dump_tile_data<W: std::io::Write>(
            &self, buffer: &mut W) -> std::io::Result<()> {
        let tile_size = 2 * 8;
        let num_tiles = (0x9800 - 0x8000) / tile_size;
        // print tile data in rows of 16 tiles each
        let tiles_per_row = 16;
        let num_rows = num_tiles / tiles_per_row;
        writeln!(buffer, "P2")?;
        writeln!(buffer, "{} {}", tiles_per_row*8, num_rows*8)?;
        writeln!(buffer, "3")?;
        let mut row_start = 0x8000;
        for _tile_row in 0..num_rows {
            for row in 0..8 {
                for tile_col in 0..tiles_per_row {
                    let tile = self.read16(row_start + 2 * row
                                           + tile_size * tile_col);
                    let p = ((tile >> 14) & 0b10) | ((tile >> 7) & 1);
                    write!(buffer, "{}", 3 - p)?;
                    for i in 1..8 {
                        let p = ((tile >> 14-i) & 0b10) | ((tile >> 7-i) & 1);
                        write!(buffer, " {}", 3 - p)?;
                    }
                    writeln!(buffer)?;
                }
            }
            row_start += tiles_per_row * tile_size;
        }
        Ok(())
    }

    pub fn dump_bg<W: std::io::Write>(&self,
                                      buffer: &mut W) -> std::io::Result<()> {
        let lcdc = self.lcdc();
        let palette = Self::expand_palette(self.bg_palette());
        let tile_map_start = lcdc.bg_tilemap_start();
        writeln!(buffer, "P2")?;
        writeln!(buffer, "256 256")?;
        writeln!(buffer, "3")?;
        let mut tiles: [u8; 32] = [0; 32];
        for tile_row in 0..32 {
            for tile_col in 0..32u16 {
                let tile
                    = self.read8(tile_map_start + 32 * tile_row + tile_col);
                tiles[tile_col as usize] = tile;
            }
            for row in 0..8 {
                for tile_col in 0..32 {
                    let tile_address
                        = lcdc.get_bg_or_window_tile_address(tiles[tile_col]);
                    let tile = self.read16(tile_address + 2 * row);
                    let index = ((tile >> 14) & 0b10)
                              | ((tile >> 7) & 1);
                    let p = palette[index as usize];
                    write!(buffer, "{}", 3 - p)?;
                    for i in 1..8 {
                        let index = ((tile >> 14-i) & 0b10)
                                  | ((tile >> 7-i) & 1);
                        let p = palette[index as usize];
                        write!(buffer, " {}", 3 - p)?;
                    }
                    writeln!(buffer)?;
                }
            }
        }
        Ok(())
    }

    fn expand_palette(palette: u8) -> [u8; 4] {
        [palette & 0b11,
         (palette >> 2) & 0b11,
         (palette >> 4) & 0b11,
         (palette >> 6) & 0b11]
    }

    pub fn get_requested_interrupts(&self) -> u8 {
        let requests = self.read8(0xFF0F);
        self.read8(0xFFFF) & requests & 0x1F
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
    pub fn are_lcd_and_ppu_enabled(self) -> bool {
        self.flags & 128 != 0
    }

    pub fn window_tilemap_start(self) -> u16 {
        if self.flags & 64 == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    pub fn is_window_enabled(self) -> bool {
        self.flags & 32 != 0
    }

    /// Tile data offset and signedness of addressing
    ///
    /// https://gbdev.io/pandocs/#vram-tile-data
    fn bg_and_window_tile_data_offset_and_addressing(self) -> (u16, bool) {
        if self.flags & 16 == 0 {
            (0x9000, true)
        } else {
            (0x8000, false)
        }
    }

    pub fn get_bg_or_window_tile_address(self, tile: u8) -> u16 {
        let (offset, signed)
            = self.bg_and_window_tile_data_offset_and_addressing();
        let tile_size = 16; // 8 lines with 2 bytes each
        if signed {
            (offset as i16 + (tile as i8) as i16 * tile_size as i16) as u16
        } else {
            offset + tile as u16 * tile_size
        }
    }

    pub fn bg_tilemap_start(self) -> u16 {
        if self.flags & 8 == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    pub fn obj_height(self) -> u8 {
        // OBJ width is always 8
        if self.flags & 4 == 0 {
            8
        } else {
            16
        }
    }

    pub fn is_obj_enabled(self) -> bool {
        self.flags & 2 != 0
    }

    pub fn is_bg_and_window_enabled(self) -> bool {
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
pub struct LcdStatus<'a> {
    flags: &'a mut u8,
}

impl LcdStatus<'_> {
    fn lyc_eq_ly_interrupt_set(&self) -> bool {
        *self.flags & (1 << 6) != 0
    }

    fn mode2_oam_interrupt_set(&self) -> bool {
        *self.flags & (1 << 5) != 0
    }

    fn mode1_vblank_interrupt_set(&self) -> bool {
        *self.flags & (1 << 4) != 0
    }

    fn mode0_hblank_interrupt_set(&self) -> bool {
        *self.flags & (1 << 3) != 0
    }

    fn update_lyc_eq_ly(&mut self, set: bool) {
        if set {
            *self.flags |= 1 << 2;
        } else {
            *self.flags &= !(1 << 2);
        }
    }

    fn mode(&self) -> LcdMode {
        (*self.flags & 3).into()
    }

    pub fn set_mode(&mut self, mode: LcdMode) -> bool {
        *self.flags &= !0x03;
        *self.flags |= mode as u8;
        match mode {
            LcdMode::HBlank => self.mode0_hblank_interrupt_set(),
            LcdMode::VBlank => self.mode1_vblank_interrupt_set(),
            LcdMode::SearchingOAM => self.mode2_oam_interrupt_set(),
            LcdMode::TransferringDataToLcdController => false,
        }
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
