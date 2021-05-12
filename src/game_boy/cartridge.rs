use std::io;
use std::io::Read;
use std::fs::File;
use std::str;

pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    memory_controller: MemoryController,
}

impl Cartridge {
    pub fn load_from_file(mut file: File) -> io::Result<Self> {
        let mut rom = Vec::new();
        file.read_to_end(&mut rom)?;
        let header = CartridgeHeader{rom: &rom};
        let memory_controller
            = MemoryController::from_cartridge_header(&header);
        let ram = vec![0; header.num_ram_banks() as usize * 8 * 1024];
        Ok(Self{
            rom,
            ram,
            memory_controller,
        })
    }

    pub fn read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => { // ROM Bank 0
                self.rom[address as usize]
            }
            0x4000..=0x7FFF => { // ROM X (switchable via Memory Controller)
                self.romx_read8(address)
            }
            0xA000..=0xBFFF => { // SRAM  Cartridge RAM
                self.ram_read8(address)
            }
            _ => panic!("Trying to read non-Cartridge address {:0>4X}.",
                        address),
        }
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {
                self.memory_controller.register_write8(address, value);
            }
            0xA000..=0xBFFF => {
                self.memory_controller.ram_write8(&mut self.ram, address,
                                                  value);
            }
            _ => panic!("Trying to write non-Cartridge address {:0>4X}.",
                        address),
        }
    }

    pub fn rom0_read8(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    pub fn romx_read8(&self, address: u16) -> u8 {
        self.memory_controller.romx_read8(&self.rom, address)
    }

    pub fn ram_read8(&self, address: u16) -> u8 {
        self.memory_controller.ram_read8(&self.ram, address)
    }

    pub fn header(&self) -> CartridgeHeader {
        CartridgeHeader{rom: &self.rom}
    }
}

/// The type of a cartridge
///
/// Possible values:
/// 0x00  ROM ONLY
/// 0x01  MBC1
/// 0x02  MBC1+RAM
/// 0x03  MBC1+RAM+BATTERY
/// 0x05  MBC2
/// 0x06  MBC2+BATTERY
/// 0x08  ROM+RAM *
/// 0x09  ROM+RAM+BATTERY *
/// 0x0B  MMM01
/// 0x0C  MMM01+RAM
/// 0x0D  MMM01+RAM+BATTERY
/// 0x0F  MBC3+TIMER+BATTERY
/// 0x10  MBC3+TIMER+RAM+BATTERY **
/// 0x11  MBC3
/// 0x12  MBC3+RAM **
/// 0x13  MBC3+RAM+BATTERY **
/// 0x19  MBC5
/// 0x1A  MBC5+RAM
/// 0x1B  MBC5+RAM+BATTERY
/// 0x1C  MBC5+RUMBLE
/// 0x1D  MBC5+RUMBLE+RAM
/// 0x1E  MBC5+RUMBLE+RAM+BATTERY
/// 0x20  MBC6
/// 0x22  MBC7+SENSOR+RUMBLE+RAM+BATTERY
/// 0xFC  POCKET CAMERA
/// 0xFD  BANDAI TAMA5
/// 0xFE  HuC3
/// 0xFF  HuC1+RAM+BATTERY
#[derive(Debug)]
pub struct CartridgeType(u8);

impl CartridgeType {
    pub fn memory_controller(self) -> MemoryControllerModel {
        use MemoryControllerModel::*;
        match self.0 {
            0x00 => NoController,
            0x01..=0x03 => MBC1,
            0x05..=0x06 => MBC2,
            0x08..=0x09 => NoController,
            0x0B..=0x0D => MMM01,
            0x0F..=0x13 => MBC3,
            0x19..=0x1E => MBC5,
            0x20 => MBC6,
            0x22 => MBC7,
            0xFC => PocketCamera,
            0xFD => BandaiTAMA5,
            0xFE => HuC3,
            0xFF => HuC1,
            cartridge_type => {
                unimplemented!("Unknown cartridge type: {:0>2X}.",
                               cartridge_type);
            }
        }
    }
}

#[derive(Debug)]
pub enum MemoryControllerModel {
    NoController,
    MBC1,
    MBC2,
    MBC3, // or MBC30 if RAM size is 64KB
    MBC5,
    MBC6,
    MBC7,
    MMM01,
    HuC1,
    HuC3,
    PocketCamera,
    BandaiTAMA5,
}

enum MemoryController {
    NoController,
    MBC1(MBC1),
    MBC3(MBC3),
}

impl MemoryController {
    fn from_cartridge_header(header: &CartridgeHeader) -> Self {
        let controller_model = header.cartridge_type().memory_controller();
        use MemoryControllerModel as Model;
        match controller_model {
            Model::NoController => Self::NoController,
            Model::MBC1 => Self::MBC1(MBC1::from_cartridge_header(header)),
            Model::MBC3 => Self::MBC3(MBC3::from_cartridge_header(header)),
            _ => unimplemented!("Memory controller {:?} not handled yet.",
                                controller_model),
        }
    }

    fn romx_read8(&self, rom: &[u8], address: u16) -> u8 {
        use MemoryController::*;
        match self {
            NoController => rom[address as usize],
            MBC1(mbc1) => {
                rom[address as usize - 0x4000 + mbc1.rom_bank_offset()]
            }
            MBC3(mbc3) => {
                rom[address as usize - 0x4000 + mbc3.rom_bank_offset()]
            }
        }
    }

    fn ram_read8(&self, ram: &[u8], address: u16) -> u8 {
        use MemoryController::*;
        match self {
            NoController => ram[address as usize],
            MBC1(mbc1) => {
                ram[address as usize - 0xA000 + mbc1.ram_bank_offset()]
            }
            MBC3(mbc3) => {
                ram[address as usize - 0xA000 + mbc3.ram_bank_offset()]
            }
        }
    }

    fn register_write8(&mut self, address: u16, value: u8) {
        use MemoryController::*;
        match self {
            NoController => unimplemented!(
                "Writing {:0>2X} to {:0>4X} without memory controller.",
                value, address),
            MBC1(mbc1) => mbc1.register_write8(address, value),
            MBC3(mbc3) => mbc3.register_write8(address, value),
        }
    }

    fn ram_write8(&mut self, ram: &mut [u8], address: u16, value: u8) {
        use MemoryController::*;
        match self {
            NoController => unimplemented!(
                "Writing {:0>2X} to {:0>4X} without memory controller.",
                value, address),
            MBC1(mbc1) => {
                ram[address as usize - 0xA000 + mbc1.ram_bank_offset()]
                    = value;
            }
            MBC3(mbc3) => {
                ram[address as usize - 0xA000 + mbc3.ram_bank_offset()]
                    = value;
            }
        }
    }
}

trait MemoryControllerRegisters {
    fn register_write8(&mut self, address: u16, value: u8);

    fn rom_bank_offset(&self) -> usize;

    fn ram_bank_offset(&self) -> usize;
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
enum MBC1BankingMode {
    Simple = 0,
    Advanced = 1,
}

impl From<u8> for MBC1BankingMode {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Simple,
            0x01 => Self::Advanced,
            _ => panic!("Unknown MBC1 banking mode: {:0>2X}.", value),
        }
    }
}

struct MBC1 {
    rom_bank: u8,
    ram_bank: u8,
    num_rom_banks: u16,
    num_ram_banks: u8,
    banking_mode: MBC1BankingMode,
}

impl MBC1 {
    fn from_cartridge_header(header: &CartridgeHeader) -> Self {
        let num_rom_banks = header.num_rom_banks();
        let num_ram_banks = header.num_ram_banks();
        if num_rom_banks > 64 || num_ram_banks > 1 {
            unimplemented!(
                "MBC1 with {} ROM banks and {} RAM banks not implemented yet.",
                num_rom_banks, num_ram_banks);

        }
        Self{
            rom_bank: 1,
            ram_bank: 0,
            num_rom_banks,
            num_ram_banks,
            banking_mode: MBC1BankingMode::Simple,
        }
    }
}

impl MemoryControllerRegisters for MBC1 {
    fn register_write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => { // RAM Enable
                // 0x00  Disable RAM (default)
                // 0x0A  Enable RAM
                match value {
                    0x00 => eprintln!("disable cartridge RAM."),
                    0x0A => eprintln!("enable cartridge RAM."),
                    _ => eprintln!("Unexpected RAM enable/disable value {}",
                                   value),
                }
            }
            0x2000..=0x3FFF => { // ROM Bank Number
                // TODO: mask with max bits required for num_rom_banks
                let mut bank = value & 0x1F;
                if bank == 0 {
                    bank += 1;
                }
                eprintln!("switching to ROM bank {}", bank);
                self.rom_bank &= 0xE0;
                self.rom_bank |= bank;
            }
            0x4000..=0x5FFF => {
                // RAM Bank Number | Upper Bits of ROM Bank Number
                let value = value & 0x03;
                match self.banking_mode {
                    MBC1BankingMode::Simple => {
                        // Upper Bits of ROM Bank Number
                        self.rom_bank &= 0x1F;
                        self.rom_bank |= value << 5;
                    }
                    MBC1BankingMode::Advanced => {
                        // RAM Bank Number
                        self.ram_bank = value;
                    }
                }
            }
            0x6000..=0x7FFF => { // Banking Mode Select
                eprintln!("Select banking mode 0x{:0>2X}", value);
                self.banking_mode = value.into();
            }
            _ => unreachable!("{:0>4X} is not a cartridge register.", address),
        }
    }

    fn rom_bank_offset(&self) -> usize {
        0x4000 * self.rom_bank as usize
    }

    fn ram_bank_offset(&self) -> usize {
        0x2000 * self.ram_bank as usize
    }
}

struct MBC3 {
    rom_bank: u8,
    ram_bank: u8,
    num_rom_banks: u16,
    num_ram_banks: u8,
}

impl MBC3 {
    fn from_cartridge_header(header: &CartridgeHeader) -> Self {
        let num_rom_banks = header.num_rom_banks();
        let num_ram_banks = header.num_ram_banks();
        Self{
            rom_bank: 1,
            ram_bank: 0,
            num_rom_banks,
            num_ram_banks,
        }
    }
}

impl MemoryControllerRegisters for MBC3 {
    fn register_write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => { // RAM and Timer Enable
                // 0x00  Disable RAM (default)
                // 0x0A  Enable RAM
                match value {
                    0x00 => eprintln!("disable cartridge RAM."),
                    0x0A => eprintln!("enable cartridge RAM."),
                    _ => eprintln!("Unexpected RAM enable/disable value {}",
                                   value),
                }
            }
            0x2000..=0x3FFF => { // ROM Bank Number
                // TODO: mask with max bits required for num_rom_banks
                let mut bank = value & 0x7F;
                if bank == 0 {
                    bank += 1;
                }
                eprintln!("switching to ROM bank {}", bank);
                self.rom_bank = bank;
            }
            0x4000..=0x5FFF => {
                // RAM Bank Number | RTC Register Select
                match value {
                    0x00..=0x03 => { // RAM Bank Number
                        self.ram_bank = value;
                    }
                    0x08..=0x0C => {
                        unimplemented!("Mapping RTC register {:0>2X}.",
                                       value);
                    }
                    _ => panic!("Unexpected RAM Bank/RTC Register: {:0>2X}.",
                                value),
                }
            }
            0x6000..=0x7FFF => { // Latch Clock Data
                unimplemented!("Latching clock data.");
            }
            _ => unreachable!("{:0>4X} is not a cartridge register.", address),
        }
    }

    fn rom_bank_offset(&self) -> usize {
        0x4000 * self.rom_bank as usize
    }

    fn ram_bank_offset(&self) -> usize {
        0x2000 * self.ram_bank as usize
    }
}

const LOGO: [u8; 0x30] = [
     0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
     0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
     0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
     0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
     0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
     0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

pub struct CartridgeHeader<'a> {
    rom: &'a [u8],
}

impl<'a> CartridgeHeader<'a> {
    pub fn is_logo_correct(&self) -> bool {
        self.rom[0x104..=0x133] == LOGO
    }

    pub fn is_header_checksum_correct(&self) -> bool {
        let mut x: u8 = 0;
        for y in &self.rom[0x0134..=0x014C] {
            x = x.overflowing_sub(*y).0.overflowing_sub(1).0;
        }
        self.header_checksum() == x
    }

    pub fn title(&self) -> &[u8] {
        &self.rom[0x134..=0x143]
    }

    pub fn manufacturer_code(&self) -> Option<&str> {
        str::from_utf8(&self.rom[0x013F..=0x0142]).ok()
    }

    pub fn color_compat(&self) -> ColorCompat {
        let cgb = self.rom[0x0143];
        match cgb {
            0x80 => ColorCompat::CGBcompat,
            0xC0 => ColorCompat::CGBonly,
            0x84 | 0x88 => ColorCompat::PGM,
            _ => ColorCompat::DGM,
        }
    }

    pub fn supports_sgb_function(&self) -> bool {
        self.rom[0x0146] == 0x03
    }

    pub fn cartridge_type(&self) -> CartridgeType {
        CartridgeType(self.rom[0x0147])
    }

    /// Number of ROM banks of 16KB each
    pub fn num_rom_banks(&self) -> u16 {
        2 << self.rom[0x0148]
    }

    /// Number of RAM banks of 8KB each
    pub fn num_ram_banks(&self) -> u8 {
        match self.rom[0x0149] {
            0x00 => 0,
            // 0x01 is used by some public domain ROMs
            // https://gbdev.io/pandocs/#_0149-ram-size
            0x01 => unimplemented!("Unknown RAM size 0x01"),
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            num => unimplemented!("Unknown RAM size 0x{:0>2X}", num),
        }
    }

    pub fn rom_version(&self) -> u8 {
        self.rom[0x14C]
    }

    pub fn header_checksum(&self) -> u8 {
        self.rom[0x14D]
    }

    pub fn global_checksum(&self) -> u16 {
        // This is probably the only big-endian word used by the GameBoy.
        ((self.rom[0x14E] as u16) << 8) + self.rom[0x14F] as u16
    }

    pub fn uses_new_licensee_code(&self) -> bool {
        self.old_licensee_code() == 0x33
    }

    pub fn old_licensee_code(&self) -> u8 {
        self.rom[0x014B]
    }

    pub fn new_licensee_code(&self) -> Option<&str> {
        str::from_utf8(&self.rom[0x0144..=0x0145]).ok()
    }

    pub fn is_japanese(&self) -> bool {
        self.rom[0x014A] == 0
    }
}

pub enum ColorCompat {
    DGM,
    CGBcompat,
    CGBonly,
    PGM,
}
