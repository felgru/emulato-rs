// SPDX-FileCopyrightText: 2021 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

use minifb::{Key, Window, WindowOptions};

use super::io::{IO, HEIGHT, WIDTH};

/// A 160x144 pixel display with 4 shades of gray
pub struct EmulatorWindow {
    display_buffer: Vec<u32>,
    window: Window,
}

const PIXEL_SIZE: usize = 4;

const COLORS: [u32; 4] = [0xFFFFFF, 0x808080, 0x404040, 0];

impl Default for EmulatorWindow {
    fn default() -> Self {
        let window = Window::new(
            "Game Boy emulator",
            WIDTH * PIXEL_SIZE,
            HEIGHT * PIXEL_SIZE,
            WindowOptions::default(),
        ).unwrap();
        Self{
            display_buffer: vec![0; WIDTH * HEIGHT * PIXEL_SIZE * PIXEL_SIZE],
            window,
        }
    }
}

impl IO for EmulatorWindow {
    fn refresh(&mut self, pixels: &[u8]) {
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

    fn is_esc_pressed(&self) -> bool {
        self.window.is_key_down(Key::Escape)
    }

    /// Get pressed JoyPad keys
    ///
    /// Arrows are mapped to arrow keys, B to Z, A to X, SELECT to Q and
    /// START to W.
    ///
    /// Return a bitmap with 1 bit per button, which is 1 if pressed
    /// and 0 if unpressed.
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
    fn get_key_presses(&self) -> u8 {
        let mut presses = 0x00;
        if self.window.is_key_down(Key::Right) {
            presses |= 0x01;
        }
        if self.window.is_key_down(Key::Left) {
            presses |= 0x02;
        }
        if self.window.is_key_down(Key::Up) {
            presses |= 0x04;
        }
        if self.window.is_key_down(Key::Down) {
            presses |= 0x08;
        }
        if self.window.is_key_down(Key::X) { // A
            presses |= 0x10;
        }
        if self.window.is_key_down(Key::Z) { // B
            presses |= 0x20;
        }
        if self.window.is_key_down(Key::Q) { // Select
            presses |= 0x40;
        }
        if self.window.is_key_down(Key::W) { // Start
            presses |= 0x80;
        }
        if presses != 0 {
            eprintln!("Keypresses: {:0>2X}", presses);
        }
        presses
    }
}
