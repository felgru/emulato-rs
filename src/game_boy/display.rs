// SPDX-FileCopyrightText: 2021 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

use super::io::{IO, HEIGHT, WIDTH};

/// A 160x144 pixel display with 4 shades of gray
pub struct Display {
    pixels: Vec<u8>,
}

impl Display {
    pub fn new() -> Self {
        Self{
            pixels: vec![0; WIDTH * HEIGHT],
        }
    }

    pub fn line_buffer(&mut self, y: u8) -> &mut [u8] {
        &mut self.pixels[y as usize * WIDTH..((y + 1) as usize * WIDTH)]
    }

    pub fn refresh<Window: IO>(&self, window: &mut Window) {
        window.refresh(&self.pixels);
    }
}
