use crate::*;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Pixel(i32),
    /// 0.0 to 1.0 of parent's total space.
    Percentage(f32),
    /// Fill remaining space in container.
    Fill,
    /// Fill remaing space minus some value.
    FillMinus(i32),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CrossAlign {
    #[default]
    Start,
    Center,
    End,
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

pub use border::*;

#[rustfmt::skip] 
pub mod border {
    pub const NONE:   u8 = 0     ;
    pub const TOP:    u8 = 1 << 0;
    pub const BOTTOM: u8 = 1 << 1;
    pub const LEFT:   u8 = 1 << 2;
    pub const RIGHT:  u8 = 1 << 3;
    pub const ALL:    u8 = 0b1111;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub fg: Option<u32>,
    pub bg: Option<u32>,
    pub radius: Option<usize>,

    pub border: Option<u32>,
    pub border_thickness: Option<usize>,
    pub border_side: Option<u8>,

    pub is_selected: bool,
    pub selected: Option<u32>,
    pub selected_border: Option<u32>,

    pub hover: Option<u32>,
    pub hover_border: Option<u32>,

    pub font: usize,
    pub font_size: Option<usize>,

    pub x: Option<Size>,
    pub y: Option<Size>,
    pub width: Option<Size>,
    pub height: Option<Size>,
    pub padding: Option<Padding>,

    pub depth: Option<usize>,

    pub alignment: Option<Alignment>,
    pub cross_align: Option<CrossAlign>,
    pub gap: Option<Size>,

    pub opacity: Option<u8>,

    #[cfg(feature = "image")]
    pub fit: Option<ImageFit>,
}

impl Style {
    pub fn gap(mut self, gap: impl IntoSize) -> Self {
        self.gap = gap.into_size();
        self
    }

    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.x = Some(Size::Pixel(bounds.x));
        self.y = Some(Size::Pixel(bounds.y));
        self.width = Some(Size::Pixel(bounds.width));
        self.height = Some(Size::Pixel(bounds.height));
        self
    }

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

    pub fn border_thickness(mut self, border_thickness: usize) -> Self {
        self.border_thickness = Some(border_thickness);
        self
    }

    pub fn border_side(mut self, border_side: u8) -> Self {
        self.border_side = Some(border_side);
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

    pub fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
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

    pub fn width(mut self, w: impl IntoSize) -> Self {
        self.width = w.into_size();
        self
    }

    pub fn height(mut self, h: impl IntoSize) -> Self {
        self.height = h.into_size();
        self
    }

    pub fn y(mut self, y: impl IntoSize) -> Self {
        self.y = y.into_size();
        self
    }

    pub fn x(mut self, x: impl IntoSize) -> Self {
        self.x = x.into_size();
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.width = Some(Size::Fill);
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.height = Some(Size::Fill);
        self
    }

    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn align_left(mut self) -> Self {
        self.alignment = Some(Alignment::Left);
        self
    }

    pub fn align_center(mut self) -> Self {
        self.alignment = Some(Alignment::Center);
        self
    }

    pub fn align_right(mut self) -> Self {
        self.alignment = Some(Alignment::Right);
        self
    }

    pub fn align_top_left(mut self) -> Self {
        self.alignment = Some(Alignment::TopLeft);
        self
    }

    pub fn align_top_center(mut self) -> Self {
        self.alignment = Some(Alignment::TopCenter);
        self
    }

    pub fn align_top_right(mut self) -> Self {
        self.alignment = Some(Alignment::TopRight);
        self
    }

    pub fn align_bottom_left(mut self) -> Self {
        self.alignment = Some(Alignment::BottomLeft);
        self
    }

    pub fn align_bottom_center(mut self) -> Self {
        self.alignment = Some(Alignment::BottomCenter);
        self
    }

    pub fn align_bottom_right(mut self) -> Self {
        self.alignment = Some(Alignment::BottomRight);
        self
    }

    pub fn cross_align(mut self, cross_align: CrossAlign) -> Self {
        self.cross_align = Some(cross_align);
        self
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Add fonts to the library to get a font ID.
    /// Fonts are all in a vector, so it's just the index...
    /// ```ignore
    /// let font_id = ui.add_font(new_font);
    /// ui.text(":)", style().font(font_id));
    /// ```
    pub fn font(mut self, font_id: usize) -> Style {
        self.font = font_id;
        self
    }

    #[cfg(feature = "image")]
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = Some(fit);
        self
    }

    pub fn opacity(mut self, opacity: u8) -> Self {
        self.opacity = Some(opacity);
        self
    }

    pub fn fill(mut self) -> Self {
        self.width = Some(Size::Fill);
        self.height = Some(Size::Fill);
        self
    }
}

impl Into<Style> for Rect {
    fn into(self) -> Style {
        style().bounds(self)
    }
}

macro_rules! impl_pad_swizzle {
    ($($name:ident => [$($edge:ident),+]);* $(;)?) => {
        impl Style {
            $(
                #[doc = concat!("Set padding for (", stringify!($($edge)+), ")")]
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

pub fn bg(color: u32) -> Style {
    style().bg(color)
}

pub fn fg(color: u32) -> Style {
    style().fg(color)
}

pub fn bounds(bounds: Rect) -> Style {
    style().bounds(bounds)
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

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

pub const fn with_alpha(color: u32, a: u8) -> u32 {
    (color & 0x00FF_FFFF) | ((a as u32) << 24)
}

pub const fn alpha(color: u32) -> u8 {
    match (color >> 24) & 0xFF {
        0 => 255,
        a => a as u8,
    }
}

pub const fn hex(color: &str) -> u32 {
    match u32::from_str_radix(color.split_at(1).1, 16) {
        Ok(hex) => hex,
        Err(_) => panic!("Invalid hex color."),
    }
}

pub const fn split(color: u32) -> (u8, u8, u8, u8) {
    (
        (color >> 16 & 0xFF) as u8,
        (color >> 8 & 0xFF) as u8,
        (color & 0xFF) as u8,
        alpha(color),
    )
}

pub const fn split_f32(color: u32) -> (f32, f32, f32) {
    (
        (color >> 16 & 0xFF) as f32,
        (color >> 8 & 0xFF) as f32,
        (color & 0xFF) as f32,
    )
}

impl Default for Size {
    fn default() -> Self {
        Size::Pixel(0)
    }
}

/// Helper trait to simplify writing Size constraints.
/// Allows users to write None, 13, 0.2, -32 or Size::*.
pub trait IntoSize {
    fn into_size(self) -> Option<Size>;
}

impl IntoSize for Size {
    fn into_size(self) -> Option<Size> {
        Some(self)
    }
}

impl IntoSize for f32 {
    fn into_size(self) -> Option<Size> {
        Some(Size::Percentage(self))
    }
}

//Yeah keep it for now, I'll think about it later...
impl IntoSize for i32 {
    fn into_size(self) -> Option<Size> {
        if self < 0 {
            Some(Size::FillMinus(self))
        } else {
            Some(Size::Pixel(self))
        }
    }
}

impl IntoSize for usize {
    fn into_size(self) -> Option<Size> {
        Some(Size::Pixel(self as i32))
    }
}

pub fn text<'a>(content: impl Into<Cow<'a, str>>, style: Style) -> Line<'a> {
    Line {
        content: content.into(),
        style,
    }
}

#[derive(Clone, Debug)]
pub struct Line<'a> {
    pub content: Cow<'a, str>,
    pub style: Style,
}

impl<'a> From<&'a str> for Line<'a> {
    fn from(content: &'a str) -> Self {
        Line {
            content: Cow::Borrowed(content),
            style: Style::default(),
        }
    }
}

impl From<String> for Line<'static> {
    fn from(content: String) -> Self {
        Line {
            content: Cow::Owned(content),
            style: Style::default(),
        }
    }
}

impl<'a> From<Cow<'a, str>> for Line<'a> {
    fn from(content: Cow<'a, str>) -> Self {
        Line {
            content,
            style: Style::default(),
        }
    }
}
