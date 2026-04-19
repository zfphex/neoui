pub struct Style {
    pub bg: Option<u32>,
    pub fg: Option<u32>,
}

pub fn bg(color: u32) -> Style {
    Style {
        bg: Some(color),
        fg: None,
    }
}

pub fn fg(color: u32) -> Style {
    Style {
        bg: None,
        fg: Some(color),
    }
}

pub fn style() -> Style {
    Style { bg: None, fg: None }
}

pub const fn white() -> u32 {
    rgb(255, 255, 255)
}

pub const fn gray() -> u32 {
    rgb(128, 128, 128)
}

pub const fn black() -> u32 {
    rgb(0, 0, 0)
}

pub const fn red() -> u32 {
    rgb(255, 0, 0)
}
pub const fn green() -> u32 {
    rgb(0, 255, 0)
}
pub const fn blue() -> u32 {
    rgb(0, 0, 255)
}

pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

pub const fn split(color: u32) -> (u8, u8, u8) {
    (
        (color >> 16 & 0xFF) as u8,
        (color >> 8 & 0xFF) as u8,
        (color & 0xFF) as u8,
    )
}
