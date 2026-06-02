#[derive(Debug, Clone, Copy)]
pub enum Size {
    Pixel(usize),
    ///0.0 to 1.0 of parent's total space
    Percentage(f32),
    Remaining,
    RemainingMinus(i32),
}

impl Default for Size {
    fn default() -> Self {
        Size::Pixel(0)
    }
}

impl Into<Size> for usize {
    fn into(self) -> Size {
        Size::Pixel(self)
    }
}

impl Into<Size> for f32 {
    fn into(self) -> Size {
        Size::Percentage(self)
    }
}

//Yeah keep it for now, I'll think about it later...
impl Into<Size> for i32 {
    fn into(self) -> Size {
        if self < 0 {
            Size::RemainingMinus(self)
        } else {
            Size::Pixel(self as usize)
        }
    }
}

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
    pub border: Option<u32>,
    pub padding: Option<Padding>,
    pub font_size: Option<usize>,
    pub selected: Option<u32>,
    pub selected_border: Option<u32>,
    pub hover: Option<u32>,
    pub hover_border: Option<u32>,
    pub radius: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub outline_thickness: Option<usize>,
    pub depth: Option<usize>,
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

    pub fn border(mut self, color: u32) -> Self {
        self.border = Some(color);
        self
    }

    pub fn radius(mut self, r: usize) -> Self {
        self.radius = Some(r);
        self
    }

    pub fn font_size(mut self, font_size: usize) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn selected(mut self, color: u32) -> Self {
        self.selected = Some(color);
        self
    }

    pub fn selected_border(mut self, color: u32) -> Self {
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

    pub fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }

    pub fn height(mut self, h: usize) -> Self {
        self.height = Some(h);
        self
    }

    pub fn outline(mut self, outline_thickness: usize) -> Self {
        self.outline_thickness = Some(outline_thickness);
        self
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }
}

macro_rules! impl_pad_swizzle {
    ($($name:ident => [$($edge:ident),+]);* $(;)?) => {
        impl Style {
            $(
                #[doc = concat!("Set padding for: (", stringify!($($edge)+), ")")]
                pub fn $name(mut self, value: usize) -> Self {
                    let mut p = self.padding.unwrap_or_default();
                    $(
                        p.$edge = value;
                    )+
                    self.padding = Some(p);
                    self
                }
            )*
        }
    };
}

impl_pad_swizzle! {
    pad   => [top, bottom, left, right];
    padh  => [left, right];
    padv  => [top, bottom];

    padt  => [top];
    padb  => [bottom];
    padl  => [left];
    padr  => [right];

    padtl => [top, left];
    padtr => [top, right];
    padbl => [bottom, left];
    padbr => [bottom, right];

    padtb => [top, bottom];
    padlr => [left, right];
    padrl => [right, left];
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

pub fn hex(color: &str) -> u32 {
    u32::from_str_radix(color.split_at(1).1, 16).expect("Invalid hex color")
}

pub const fn split(color: u32) -> (u8, u8, u8) {
    (
        (color >> 16 & 0xFF) as u8,
        (color >> 8 & 0xFF) as u8,
        (color & 0xFF) as u8,
    )
}
