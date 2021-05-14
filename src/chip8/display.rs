use std::cmp::min;
use std::fmt;
use minifb::{Key, Window, WindowOptions};

/// A 64x32 pixel monochrome display
///
/// The corners of the display have the following coordinates:
/// (0, 0) (63, 0)
/// (0,31) (63,31)
pub struct Display {
    pixels: Vec<bool>,
    display_buffer: Vec<u32>,
    window: Window,
    width: usize,
    height: usize,
}

const PIXEL_SIZE: usize = 4;
const SET: u32 = 0xFFFFFF;
const UNSET: u32 = 0;

impl Display {
    pub fn new(width: usize, height: usize, refresh_rate: usize) -> Self {
        let mut window = Window::new(
            "Chip-8 emulator",
            width * PIXEL_SIZE,
            height * PIXEL_SIZE,
            WindowOptions::default(),
        ).unwrap();
        use std::time::Duration;
        let wait_time = Duration::from_micros((1000000. / refresh_rate as f64)
                                              as u64);
        window.limit_update_rate(Some(wait_time));
        Self{
            pixels: vec![false; width * height],
            display_buffer: vec![UNSET; width * height
                                        * PIXEL_SIZE * PIXEL_SIZE],
            window,
            width,
            height,
        }
    }

    pub fn refresh(&mut self) {
        let buffer_width = self.width * PIXEL_SIZE;
        for line in 0..self.height {
            let buffer_line_start = line * PIXEL_SIZE * buffer_width;
            let buffer_line_range
                = buffer_line_start..(buffer_line_start + buffer_width);
            let buffer_line = &mut self.display_buffer[buffer_line_range
                                                       .clone()];
            for col in 0..self.width {
                let color = if self.pixels[line * self.width + col] {
                    SET
                } else {
                    UNSET
                };
                buffer_line[col*PIXEL_SIZE..(col+1)*PIXEL_SIZE].fill(color);
            }
            for i in 1..PIXEL_SIZE {
                self.display_buffer.copy_within(
                    buffer_line_range.clone(),
                    buffer_line_start + i * buffer_width);
            }
        }
        self.window
            .update_with_buffer(&self.display_buffer,
                                self.width * PIXEL_SIZE,
                                self.height * PIXEL_SIZE)
            .unwrap();
    }

    pub fn clear(&mut self) {
        self.pixels.fill(false);
    }

    pub fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> u8 {
        // eprint!("({:#X?}, {:#X?})\n{}", x, y, format_sprite(sprite));
        let x = x as usize;
        let y = y as usize;
        let lines = min(sprite.len(), self.height - y);
        let sprite_width = min(8, self.width - x);
        let mut any_set_pixel_unset = false;
        for i in 0..lines {
            for j in 0..sprite_width {
                let sprite_set = sprite[i] & (1 << 7 - j) != 0;
                let pixel = &mut self.pixels[(y+i) * self.width + x + j];
                *pixel ^= sprite_set;
                if sprite_set && !*pixel {
                    any_set_pixel_unset = true;
                }
            }
        }
        any_set_pixel_unset as u8
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        let key = match key {
            0x0 => Key::Key0,
            0x1 => Key::Key1,
            0x2 => Key::Key2,
            0x3 => Key::Key3,
            0x4 => Key::Key4,
            0x5 => Key::Key5,
            0x6 => Key::Key6,
            0x7 => Key::Key7,
            0x8 => Key::Key8,
            0x9 => Key::Key9,
            0xA => Key::A,
            0xB => Key::B,
            0xC => Key::C,
            0xD => Key::D,
            0xE => Key::E,
            0xF => Key::F,
            k => panic!("{:#X?} is not a valid key.", k),
        };
        self.window.is_key_down(key)
    }

    pub fn is_esc_pressed(&self) -> bool {
        self.window.is_key_down(Key::Escape)
    }

    pub fn get_key_press(&self) -> Option<u8> {
        for key in 0x0..=0xF {
            if self.is_key_pressed(key) {
                return Some(key);
            }
        }
        None
    }
}

pub fn format_sprite(sprite: &[u8]) -> String {
    let mut res = String::with_capacity(sprite.len() * 9);
    for line in sprite.iter() {
        for i in 0..8 {
            res.push(if *line & (1 << 7 -i) != 0 {
                '*'
            } else {
                ' '
            });
        }
        res.push('\n');
    }
    res
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in 0..self.height {
            let offset = line * self.width;
            for pixel in self.pixels[offset..offset+self.width].iter() {
                if *pixel {
                    write!(f, "*")?;
                } else {
                    write!(f, " ")?;
                }
            }
            eprintln!();
        }
        Ok(())
    }
}
