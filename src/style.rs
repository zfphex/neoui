#[derive(Debug, Clone, Copy, Default)]
pub struct Padding {
    pub top: usize,
    pub bottom: usize,
    pub left: usize,
    pub right: usize,
}

pub fn pad(p: usize) -> Padding {
    Padding {
        top: p,
        bottom: p,
        left: p,
        right: p,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub bg: Option<u32>,
    pub fg: Option<u32>,
    pub padding: Option<Padding>,
    pub font_size: Option<usize>,
    pub selected: Option<u32>,
    pub selected_border: Option<u32>,
    pub hover: Option<u32>,
    pub hover_border: Option<u32>,
}

impl Style {
    pub fn bg(mut self, color: u32) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn fg(mut self, color: u32) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn pad(mut self, p: usize) -> Self {
        self.padding = Some(pad(p));
        self
    }

    pub fn padt(mut self, v: usize) -> Self {
        let mut p = self.padding.unwrap_or_default();
        p.top = v;
        self.padding = Some(p);
        self
    }

    pub fn padb(mut self, v: usize) -> Self {
        let mut p = self.padding.unwrap_or_default();
        p.bottom = v;
        self.padding = Some(p);
        self
    }

    pub fn padl(mut self, v: usize) -> Self {
        let mut p = self.padding.unwrap_or_default();
        p.left = v;
        self.padding = Some(p);
        self
    }

    pub fn padr(mut self, v: usize) -> Self {
        let mut p = self.padding.unwrap_or_default();
        p.right = v;
        self.padding = Some(p);
        self
    }

    pub fn font_size(mut self, font_size: usize) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn selection(mut self, color: u32) -> Self {
        self.selected = Some(color);
        self
    }

    pub fn selection_border(mut self, color: u32) -> Self {
        self.selected_border = Some(color);
        self
    }

    pub fn hover(mut self, color: u32) -> Self {
        self.hover = Some(color);
        self
    }

    pub fn hover_border(mut self, color: u32) -> Self {
        self.hover_border = Some(color);
        self
    }
}

pub fn font_size(font_size: usize) -> Style {
    Style {
        font_size: Some(font_size),
        ..Default::default()
    }
}

pub fn bg(color: u32) -> Style {
    Style {
        bg: Some(color),
        ..Default::default()
    }
}

pub fn fg(color: u32) -> Style {
    Style {
        fg: Some(color),
        ..Default::default()
    }
}

pub fn style() -> Style {
    Style::default()
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
