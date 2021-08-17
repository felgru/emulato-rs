use std::fs::File;
use std::io;

use emulato_rs::game_boy;
use game_boy::io::{HEIGHT, WIDTH};

const MOONEYE_DIR: &'static str = "/home/felix/games/roms/gameboy/test_roms/mooneye";
const TEST_OK: &'static str = "88413F205E3822FF44203C4338422049224124FF44214422463C4A224128FF44213E4330422049224138FF44214B202049224124FF44203C423843304A3822FF";
const TEST_DE_OK: &'static str = "BFA1513C543822523E543822FF512253224124522053224124FF5122452048224128523C452048224128FF512253224138522053224138FF512253224124522053224124FF513C4520493822523E4520493822FF";
const TEST_BCD_OK: &'static str = "BF99513C5438225338533822FF512253224124522253224124FF513C4520482241285120462048224128FF512253224138512054224138FF512253224124522253224124FF513C452049382253384420493822FF82513C543822FF512253224124FF5122452048224128FF512253224138FF512253224124FF513C4520493822FF";
const TEST_CE_OK: &'static str = "BF997F4B38533822FF7F4A2253224124FF7F4920462048224128FF7F492054224138FF7F4A2253224124FF7F4B384420493822FF827F4A3E543822FF7F4A2053224124FF7F4A3C452048224128FF7F4A2053224138FF7F4A2053224124FF7F4A3E4520493822FF";
const TEST_B_OK: &'static str = "BF99513C543822FF512253224124FF513C452048224128FF512253224138FF512253224124FF513C4520493822FF";
const TEST_BC_OK_FULL: &'static str = "88413C5520FF4122433C53384220413C4B38FF413C42214334442020423C4121432C4220FF4128423F42264520413042203F43324330FF41244220422245204320202043204520FF4122433C42344420384330413C42204238FF5520FF5238FF88533052304230503E54304230FF533051244224512053244224FF5224442047214221523C452047214221FF523C51214221522052214221FF512153244224512053244224FF51214520493042305020452049304230FF82513C523E423E5338513C433CFF512252204220522252224221FF513C4520473C423C5120462047224221FF51225220422051205322433EFF512252204220522252224720FF513C45204720422053384420473C433CFF82513C54304230503E54304230FF512253244224512053244224FF5122452047214221523C452047214221FF512252214221522052214221FF512253244224512053244224FF513C452049304230503E452049304230FF82512154304230502054304230FF512153244224512053244224FF513F4520472142215220452047214221FF512152214221522052214221FF512153244224512053244224FF5121452049304230503F452049304230FF8A43306B20FF433042384238413C4B205A38FF4224422042204121432C423C44203C422E4220FF423C433042303F4332432043214423422120FF412146204222472043204321442242204120FF412143384238423C4220433043203C42224238FF92513C5438225338533822FF512253224124522253224124FF513C4520482241285120462048224128FF512253224138512054224138FF512253224124522253224124FF513C452049382253384420493822FF";
const TEST_BCDE_OK: &'static str = "TODO";
const TEST_BCDEHL_OK: &'static str = "TODO";
const TEST_CDEL_OK: &'static str = "TODO";

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(2 * bytes.len());
    for b in bytes {
        hex += &format!("{:0>2X}", b);
    }
    hex
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let num_bytes = hex.len() / 2;
    let mut bytes = Vec::with_capacity(num_bytes);
    for h in hex.as_bytes().chunks_exact(2) {
        let high = if h[0] >= 65 { h[0] - 55 } else { h[0] - '0' as u8 };
        let low = if h[1] >= 65 { h[1] - 55 } else { h[1] - '0' as u8 };
        let b = (high << 4) | low;
        bytes.push(b);
    }
    bytes
}

struct BrailleDisplay {
    pixels: Vec<u8>,
}

impl Default for BrailleDisplay {
    fn default() -> Self {
        Self{
            pixels: vec![0; WIDTH * HEIGHT],
        }
    }
}

impl BrailleDisplay {
    fn from_hex(hex: &str) -> Self {
        Self::from_compressed_1bpp_dump(&hex_to_bytes(hex))
    }

    fn to_hex(&self) -> String {
        bytes_to_hex(&self.compressed_1bpp_dump())
    }

    fn from_compressed_1bpp_dump(dump: &[u8]) -> Self {
        let mut pixels = vec![0; WIDTH * HEIGHT];
        let mut cur_pixel: usize = 0;
        for byte in dump {
            match byte & 0xC0 {
                0 => {
                    // TODO: handle bits that protrude over the line ending
                    for i in 0..6 {
                        if (byte >> (5 - i)) & 1 != 0 {
                            pixels[cur_pixel] = 3;
                        }
                        cur_pixel += 1;
                    }
                }
                0x80 => {
                    let num_lines = byte & 0x3F;
                    cur_pixel += num_lines as usize * WIDTH;
                }
                0x40 => {
                    let num_pixels = byte & 0x3F;
                    cur_pixel += num_pixels as usize;
                }
                0xC0 => {
                    let cur_line = cur_pixel / WIDTH;
                    let start_of_line = cur_line * WIDTH;
                    if cur_pixel != start_of_line {
                        cur_pixel = start_of_line + WIDTH;
                    }
                }
                _ => unreachable!(),
            }
        }
        Self{pixels}
    }

    fn is_blank(&self) -> bool {
        self.pixels.iter().all(|x| *x == 0)
    }

    fn refresh(&mut self, pixels: &[u8]) {
        self.pixels.copy_from_slice(pixels);
    }

    /// Run-length coded monochrome display dump.
    ///
    /// First two bits of each byte will encode the meaning of the following 6
    /// bytes:
    /// 00: bits are interpreted literally.
    /// 01: bits give length of a run of zero bits.
    /// 10: bits give number of zero lines.
    /// 11: rest of line is filled with zeroes.
    ///
    /// Once the compressed bit stream ends, the remaining bits are assumed
    /// to be zero.
    fn compressed_1bpp_dump(&self) -> Vec<u8> {
        let mut compressed = Vec::<u8>::with_capacity(self.pixels.len() / 8);
        let mut empty_lines: u16 = 0;
        for pixels in (&self.pixels).chunks_exact(WIDTH) {
            let mut pixels = pixels.iter();
            let mut leading_zeroes: u16 = 0;
            while let Some(pixel) = pixels.next() {
                if pixel & 3 == 0 {
                    leading_zeroes += 1;
                } else {
                    while empty_lines > 63 {
                        compressed.push(0b1011_1111);
                        empty_lines -= 63;
                    }
                    if empty_lines > 0 {
                        compressed.push(0x80 + empty_lines as u8);
                        empty_lines = 0;
                    }
                    while leading_zeroes > 63 {
                        compressed.push(0b0111_1111);
                        leading_zeroes -= 63;
                    }
                    if leading_zeroes > 0 {
                        compressed.push(0x40 + leading_zeroes as u8);
                        leading_zeroes = 0;
                    }
                    break;
                }
            }
            if leading_zeroes == 0 {
                'line_loop: loop {
                    // Here we have already read a non-zero bit
                    // and we will continue filling cur_byte with pixel bits.
                    let mut cur_byte: u8 = 0x20;
                    let mut pos: i8 = 4;
                    while pos >= 0 {
                        // TODO: How should we handle less than 8 non-zero bits
                        //       at the end of a line?
                        let pixel = *pixels.next().unwrap();
                        if pixel != 0 {
                            cur_byte |= 1 << pos;
                        }
                        pos -= 1;
                    }
                    compressed.push(cur_byte);
                    while let Some(&pixel) = pixels.next() {
                        if pixel == 0 {
                            leading_zeroes += 1;
                        } else {
                            while leading_zeroes > 63 {
                                compressed.push(0b0111_1111);
                                leading_zeroes -= 63;
                            }
                            if leading_zeroes > 0 {
                                compressed.push(0x40 + leading_zeroes as u8);
                                leading_zeroes = 0;
                            }
                            continue 'line_loop;
                        }
                    }
                    break 'line_loop;
                }
                compressed.push(0xFF);
            } else { // reached end of line full of zeroes
                empty_lines += 1;
            }
        }
        compressed
    }

    /// This is useful for printing to a terminal for debugging.
    fn format_display_as_braille(&self) -> String {
        let mut braille_bits = vec![0; (WIDTH / 2) * (HEIGHT / 4)];
        for h in 0..(HEIGHT / 4) {
            for h2 in 0..4 {
                let input_offset = (4*h + h2) * WIDTH;
                let output_offset = h * WIDTH / 2;
                let bit_indices = match h2 {
                    0 => [0, 3],
                    1 => [1, 4],
                    2 => [2, 5],
                    3 => [6, 7],
                    _ => unreachable!(),
                };
                for w in 0..(WIDTH / 2) {
                    for w2 in 0..2 {
                        if self.pixels[input_offset + 2*w + w2] != 0 {
                            let bit = bit_indices[w2];
                            braille_bits[output_offset+w] |= 1 << bit;
                        }
                    }
                }
            }
        }
        let mut res = String::with_capacity(3 * braille_bits.len() + (HEIGHT / 4));
        for (i, b) in braille_bits.into_iter().enumerate() {
            res.push(char::from_u32(0x2800 + b as u32).unwrap());
            if i % (WIDTH / 2) == (WIDTH / 2) - 1 {
                res.push('\n');
            }
        }
        res
    }
}

struct TestEmulatorWindow {
    display: BrailleDisplay,
    frame: usize,
    min_wait_frames: usize,
    reference: String,
}

impl TestEmulatorWindow {
    fn with_reference(reference: &str) -> Self {
        Self {
            display: BrailleDisplay::default(),
            frame: 0,
            min_wait_frames: 0,
            reference: reference.to_string(),
        }
    }

    fn with_reference_and_min_frames(reference: &str,
                                     min_frames: usize) -> Self {
        Self {
            display: BrailleDisplay::default(),
            frame: 0,
            min_wait_frames: min_frames,
            reference: reference.to_string(),
        }
    }
}

impl Default for TestEmulatorWindow {
    fn default() -> Self {
        Self{
            display: BrailleDisplay::default(),
            frame: 0,
            min_wait_frames: 0,
            reference: String::new(),
        }
    }
}

impl game_boy::io::IO for TestEmulatorWindow {
    fn refresh(&mut self, pixels: &[u8]) {
        self.display.refresh(pixels);
        self.frame += 1;
    }

    fn is_esc_pressed(&self) -> bool {
        if self.display.is_blank() || self.frame < self.min_wait_frames {
            false
        } else {
            println!("After {} frames.", self.frame);
            print!("{}", self.display.format_display_as_braille());
            let hex = self.display.to_hex();
            println!("{:X?}", hex);
            assert_eq!(self.reference, hex);
            true
        }
    }

    fn get_key_presses(&self) -> u8 {
        0
    }
}

fn mooneye_test_rom(dir: &str) -> io::Result<File> {
    File::open(MOONEYE_DIR.to_owned() + dir)
}

#[test]
fn mooneye_bits_mem_oam() {
    let f = mooneye_test_rom("/acceptance/bits/mem_oam.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_bits_reg_f() {
    let f = mooneye_test_rom("/acceptance/bits/reg_f.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_CE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_bits_unused_hwio_gs() {
    let f = mooneye_test_rom("/acceptance/bits/unused_hwio-GS.gb").unwrap();
    // TODO: Not sure if this is correct, as this test even fails on Sameboy
    //       when writing to register 0xFF4F.
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_halt_ime0_ei() {
    let f = mooneye_test_rom("/acceptance/halt_ime0_ei.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_halt_ime0_nointr_timing() {
    let f = mooneye_test_rom("/acceptance/halt_ime0_nointr_timing.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_halt_ime1_timing() {
    let f = mooneye_test_rom("/acceptance/halt_ime1_timing.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_B_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_halt_ime1_timing2_gs() {
    let f = mooneye_test_rom("/acceptance/halt_ime1_timing2-GS.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_BCDE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_instr_daa() {
    let f = mooneye_test_rom("/acceptance/instr/daa.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_interrupts_ie_push() {
    let f = mooneye_test_rom("/acceptance/interrupts/ie_push.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_oam_dma_restart() {
    let f = mooneye_test_rom("/acceptance/oam_dma_restart.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

// This test currently never finishes.
// #[test]
// fn mooneye_oam_dma_start() {
//     let f = mooneye_test_rom("/acceptance/oam_dma_start.gb").unwrap();
//     let window = TestEmulatorWindow::with_reference(TEST_BCDE_OK);
//     let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
//         .use_emulator_window(window)
//         .use_fast_boot_rom()
//         .load_cartridge(f).unwrap()
//         .build();
//     gameboy.run();
// }

#[test]
fn mooneye_oam_dma_timing() {
    let f = mooneye_test_rom("/acceptance/oam_dma_timing.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_oam_dma_basic() {
    let f = mooneye_test_rom("/acceptance/oam_dma/basic.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_oam_dma_reg_read() {
    let f = mooneye_test_rom("/acceptance/oam_dma/reg_read.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_oam_dma_sources_gs() {
    let f = mooneye_test_rom("/acceptance/oam_dma/sources-GS.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_div_timing() {
    let f = mooneye_test_rom("/acceptance/div_timing.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_BCD_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_div_write() {
    let f = mooneye_test_rom("/acceptance/timer/div_write.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_rapid_toggle() {
    let f = mooneye_test_rom("/acceptance/timer/rapid_toggle.gb").unwrap();
    let window = TestEmulatorWindow::with_reference_and_min_frames(
        TEST_BC_OK_FULL, 9);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tim00() {
    let f = mooneye_test_rom("/acceptance/timer/tim00.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tim00_div_trigger() {
    let f = mooneye_test_rom("/acceptance/timer/tim00_div_trigger.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tim01() {
    let f = mooneye_test_rom("/acceptance/timer/tim01.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tim01_div_trigger() {
    let f = mooneye_test_rom("/acceptance/timer/tim01_div_trigger.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tim10() {
    let f = mooneye_test_rom("/acceptance/timer/tim10.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tim10_div_trigger() {
    let f = mooneye_test_rom("/acceptance/timer/tim10_div_trigger.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tim11() {
    let f = mooneye_test_rom("/acceptance/timer/tim11.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tim11_div_trigger() {
    let f = mooneye_test_rom("/acceptance/timer/tim11_div_trigger.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_DE_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tima_reload() {
    let f = mooneye_test_rom("/acceptance/timer/tima_reload.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_BCDEHL_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tima_write_reloading() {
    let f = mooneye_test_rom("/acceptance/timer/tima_write_reloading.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_CDEL_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_timer_tma_write_reloading() {
    let f = mooneye_test_rom("/acceptance/timer/tma_write_reloading.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_CDEL_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_bits_bank1() {
    let f = mooneye_test_rom("/emulator-only/mbc1/bits_bank1.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_bits_bank2() {
    let f = mooneye_test_rom("/emulator-only/mbc1/bits_bank2.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_bits_mode() {
    let f = mooneye_test_rom("/emulator-only/mbc1/bits_mode.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_bits_ramg() {
    let f = mooneye_test_rom("/emulator-only/mbc1/bits_ramg.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_multicart_rom_8mb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/multicart_rom_8Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_ram_64kb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/ram_64kb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference_and_min_frames(TEST_OK, 55);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_ram_256kb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/ram_256kb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_rom_512kb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/rom_512kb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_rom_1mb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/rom_1Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_rom_2mb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/rom_2Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_rom_4mb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/rom_4Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_rom_8mb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/rom_8Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc1_rom_16mb() {
    let f = mooneye_test_rom("/emulator-only/mbc1/rom_16Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc2_bits_ramg() {
    let f = mooneye_test_rom("/emulator-only/mbc2/bits_ramg.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc2_bits_romb() {
    let f = mooneye_test_rom("/emulator-only/mbc2/bits_romb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc2_bits_unused() {
    let f = mooneye_test_rom("/emulator-only/mbc2/bits_unused.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc2_ram() {
    let f = mooneye_test_rom("/emulator-only/mbc2/ram.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc2_rom_512kb() {
    let f = mooneye_test_rom("/emulator-only/mbc2/rom_512kb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc2_rom_1mb() {
    let f = mooneye_test_rom("/emulator-only/mbc2/rom_1Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc2_rom_2mb() {
    let f = mooneye_test_rom("/emulator-only/mbc2/rom_2Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc5_rom_512kb() {
    let f = mooneye_test_rom("/emulator-only/mbc5/rom_512kb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc5_rom_1mb() {
    let f = mooneye_test_rom("/emulator-only/mbc5/rom_1Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc5_rom_2mb() {
    let f = mooneye_test_rom("/emulator-only/mbc5/rom_2Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc5_rom_4mb() {
    let f = mooneye_test_rom("/emulator-only/mbc5/rom_4Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc5_rom_8mb() {
    let f = mooneye_test_rom("/emulator-only/mbc5/rom_8Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc5_rom_16mb() {
    let f = mooneye_test_rom("/emulator-only/mbc5/rom_16Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc5_rom_32mb() {
    let f = mooneye_test_rom("/emulator-only/mbc5/rom_32Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}

#[test]
fn mooneye_mbc5_rom_64mb() {
    let f = mooneye_test_rom("/emulator-only/mbc5/rom_64Mb.gb").unwrap();
    let window = TestEmulatorWindow::with_reference(TEST_OK);
    let mut gameboy = game_boy::GameBoy::<TestEmulatorWindow>::builder()
        .use_emulator_window(window)
        .use_fast_boot_rom()
        .load_cartridge(f).unwrap()
        .build();
    gameboy.run();
}
