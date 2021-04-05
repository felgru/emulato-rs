use super::emulator_window::{EmulatorWindow, HEIGHT, WIDTH};

/// A 160x144 pixel display with 4 shades of gray
pub struct Display {
    pixels: Vec<u8>,
}

impl Display {
    pub fn new(refresh_rate: usize) -> Self {
        Self{
            pixels: vec![0; WIDTH * HEIGHT],
        }
    }

    pub fn write_pixel(&mut self, x: u8, y: u8, color: u8) {
        self.pixels[y as usize * HEIGHT + x as usize] = color;
    }

    pub fn refresh(&self, window: &mut EmulatorWindow) {
        window.refresh(&self.pixels);
    }
}
