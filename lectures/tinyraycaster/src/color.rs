/// RGBA color as 4-byte u32.
#[derive(Debug, Clone, Copy)]
pub struct Color(pub(crate) u32);

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: Option<u8>) -> Self {
        let a: u8 = a.unwrap_or(255);
        // rgba is stored in a u32
        Self(((a as u32) << 24) + ((b as u32) << 16) + ((g as u32) << 8) + (r as u32))
    }

    pub fn channels(&self) -> (u8, u8, u8, u8) {
        // Modify the r channel.
        // Extract the color at each channel by:
        // Right-shift bits by 1 byte chunks to get color channel as u8.
        (
            self.0 as u8,
            (self.0 >> 8) as u8,
            (self.0 >> 16) as u8,
            (self.0 >> 24) as u8,
        )
    }
}
