// SPDX-FileCopyrightText: 2021 Felix Gruber
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub trait IO {
    fn refresh(&mut self, pixels: &[u8]);

    fn is_esc_pressed(&self) -> bool;

    /// Get pressed JoyPad keys
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
    fn get_key_presses(&self) -> u8;
}
