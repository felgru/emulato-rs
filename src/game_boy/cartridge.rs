use std::io;
use std::io::Read;
use std::fs::File;
use std::str;

pub struct Cartridge {
    rom: Vec<u8>,
}

impl Cartridge {
    pub fn load_from_file(mut file: File) -> io::Result<Self> {
        let mut rom = Vec::new();
        file.read_to_end(&mut rom)?;
        Ok(Self{
            rom,
        })
    }

    pub fn rom0_read8(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    pub fn header<'a>(&'a self) -> CartridgeHeader<'a> {
        CartridgeHeader{rom: &self.rom}
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

    pub fn cartridge_type(&self) -> u8 {
        self.rom[0x0147]
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
