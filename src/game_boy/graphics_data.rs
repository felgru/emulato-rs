/// A monochrome palette
///
/// Bit 7-6 - Color for index 3
/// Bit 5-4 - Color for index 2
/// Bit 3-2 - Color for index 1
/// Bit 1-0 - Color for index 0
#[derive(Copy, Clone)]
pub struct MonochromePalette {
    palette: u8,
}

impl From<u8> for MonochromePalette {
    fn from(palette: u8) -> Self {
        Self{palette}
    }
}

impl MonochromePalette {
    pub fn color(self, index: u8) -> u8 {
        (self.palette >> (2*index)) & 3
    }
}
