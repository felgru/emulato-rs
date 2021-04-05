use minifb::{Key, Window, WindowOptions};

/// A 160x144 pixel display with 4 shades of gray
pub struct EmulatorWindow {
    display_buffer: Vec<u32>,
    window: Window,
}

const PIXEL_SIZE: usize = 4;
pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

const COLORS: [u32; 4] = [0xFFFFFF, 0x808080, 0x404040, 0];

impl EmulatorWindow {
    pub fn new(refresh_rate: usize) -> Self {
        let mut window = Window::new(
            "Game Boy emulator",
            WIDTH * PIXEL_SIZE,
            HEIGHT * PIXEL_SIZE,
            WindowOptions::default(),
        ).unwrap();
        use std::time::Duration;
        let wait_time = Duration::from_micros((1000000. / refresh_rate as f64)
                                              as u64);
        // TODO: disable limit of update rate
        window.limit_update_rate(Some(wait_time));
        Self{
            display_buffer: vec![0; WIDTH * HEIGHT * PIXEL_SIZE * PIXEL_SIZE],
            window,
        }
    }

    pub fn refresh(&mut self, pixels: &[u8]) {
        let buffer_width = WIDTH * PIXEL_SIZE;
        for line in 0..HEIGHT {
            let buffer_line_start = line * PIXEL_SIZE * buffer_width;
            let buffer_line_range
                = buffer_line_start..(buffer_line_start + buffer_width);
            let buffer_line = &mut self.display_buffer[buffer_line_range
                                                       .clone()];
            for col in 0..WIDTH {
                let color = COLORS[pixels[line * WIDTH + col] as usize];
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
                                WIDTH * PIXEL_SIZE,
                                HEIGHT * PIXEL_SIZE)
            .unwrap();
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
