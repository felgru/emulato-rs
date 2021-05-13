use std::io;
use std::io::Read;
use std::fs::File;

pub fn load_boot_rom(mut file: File) -> io::Result<[u8; 0x100]> {
    let mut rom = [0; 0x100];
    file.read_exact(&mut rom)?;
    Ok(rom)
}

pub fn fast_boot_rom() -> [u8; 0x100] {
    let mut rom = [0; 0x100];
    // LD SP, 0xFFFE
    rom[0x00] = 0x31;
    rom[0x01] = 0xFE;
    rom[0x02] = 0xFF;

    // JP 0x00FC
    rom[0x03] = 0xC3;
    rom[0x04] = 0xFC;
    rom[0x05] = 0x00;

    // Disable boot ROM by writing 0x01 to 0xFF50
    // LD A, 0x01
    rom[0xFC] = 0x3E;
    rom[0xFD] = 0x01;
    // LD (0x50), A
    rom[0xFE] = 0xE0;
    rom[0xFF] = 0x50;

    rom
}
