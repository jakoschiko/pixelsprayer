#[derive(Debug, Clone, Copy)]
pub enum Color {
    None,
    Grayscale(u8),
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
}

impl Color {
    pub fn normalize(self, disable_grayscale_support: bool) -> Self {
        match self {
            Color::Grayscale(c) if disable_grayscale_support => Color::Rgb(c, c, c),
            Color::Rgb(r, g, b) if r == g && r == b => Color::Grayscale(r),
            Color::Rgba(r, g, b, 255) if r == g && r == b => Color::Grayscale(r),
            Color::Rgba(r, g, b, 255) => Color::Rgb(r, g, b),
            Color::Rgba(_, _, _, 0) => Color::None,
            other => other,
        }
    }
}
