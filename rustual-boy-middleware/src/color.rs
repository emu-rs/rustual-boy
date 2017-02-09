//! Some color utilities that are useful for implementing the anaglyph modes

/// Represents a color
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl From<(f32, f32, f32)> for Color {
    /// Convert a tuple of RGB in the [0, 1] range to a color
    fn from((r, g, b): (f32, f32, f32)) -> Color {
        Color {r: r, g: g, b: b}
    }
}

impl From<u32> for Color {
    /// Convert a packed integer in to a color, where the compnents are RGB
    /// from most significant to least significant byte
    fn from(i: u32) -> Color {
        let r = (((i >> 16) & 0xFF) as u32) as f32;
        let g = (((i >> 8) & 0xFF) as u32) as f32;
        let b = ((i & 0xFF) as u32) as f32;

        Color {r: r, g: g, b: b}
    }
}

impl Into<u32> for Color {
    /// Convert a color from a float tuple in to RGB packed format
    fn into(self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;

        (r << 16) | (g << 8) | b
    }
}

impl From<(u8, u8, u8)> for Color {
    /// Convert a tuple of u8s to a color
    fn from((r, g, b): (u8, u8, u8)) -> Color {
        let r = (r as f32) / 255.0;
        let g = (g as f32) / 255.0;
        let b = (b as f32) / 255.0;

        Color {r: r, g: g, b: b}
    }
}

impl Color {
    /// Compute the sum of two colors
    pub fn add_color(&self, col: Color) -> Color {
        Color {
            r: self.r + col.r,
            g: self.g + col.g,
            b: self.b + col.b
        }
    }

    /// Scale a color by a uniform constant factor
    pub fn scale_color(&self, s: f32) -> Color {
        Color {
            r: s * self.r,
            g: s * self.g,
            b: s * self.b,
        }
    }
}
