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
pub enum AlignFlow {
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

impl Padding {
    pub fn axis(self, flow: Flow) -> i32 {
        if flow.vertical() {
            (self.top + self.bottom) as i32
        } else {
            (self.left + self.right) as i32
        }
    }
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    SemiLight,
    #[default]
    Regular,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Font {
    pub id: usize,
    pub weight: Weight,
    pub italic: bool,
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

    pub font: Font,
    pub font_size: Option<usize>,
    pub line_height: Option<usize>,

    pub x: Option<Size>,
    pub y: Option<Size>,
    pub width: Option<Size>,
    pub height: Option<Size>,
    pub padding: Option<Padding>,
    /// Grows the painted and interactable box outwards without reserving any layout space.
    pub margin: Option<Padding>,

    pub depth: Option<usize>,

    pub align_item: Option<Alignment>,
    pub align_flow: Option<AlignFlow>,
    pub gap: Option<Size>,
    pub clip: bool,
    pub skip_cull: bool,
    /// Rubber-band past the edges of a scroll view and bounce back.
    pub elastic: bool,
    /// Resolve fill against parent's outer bounds, ignoring padding.
    pub bleed: bool,

    pub opacity: Option<u8>,
}

impl Style {
    pub const fn new() -> Self {
        todo!();
    }
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

    pub fn line_height(mut self, line_height: usize) -> Self {
        self.line_height = Some(line_height);
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

    pub fn w(mut self, w: impl IntoSize) -> Self {
        self.width = w.into_size();
        self
    }

    pub fn h(mut self, h: impl IntoSize) -> Self {
        self.height = h.into_size();
        self
    }

    pub fn wh(mut self, wh: impl IntoSize) -> Self {
        let wh = wh.into_size();
        self.width = wh;
        self.height = wh;
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

    pub fn fill(mut self) -> Self {
        self.width = Some(Size::Fill);
        self.height = Some(Size::Fill);
        self
    }

    pub fn fillw(mut self) -> Self {
        self.width = Some(Size::Fill);
        self
    }

    pub fn fillh(mut self) -> Self {
        self.height = Some(Size::Fill);
        self
    }

    //TODO: Remove
    pub fn fill_width(mut self) -> Self {
        self.width = Some(Size::Fill);
        self
    }

    //TODO: Remove
    pub fn fill_height(mut self) -> Self {
        self.height = Some(Size::Fill);
        self
    }

    pub fn bleed(mut self) -> Self {
        self.bleed = true;
        self
    }

    pub fn align(mut self, alignment: Alignment) -> Self {
        self.align_item = Some(alignment);
        self
    }

    pub fn align_left(mut self) -> Self {
        self.align_item = Some(Alignment::Left);
        self
    }

    pub fn align_center(mut self) -> Self {
        self.align_item = Some(Alignment::Center);
        self
    }

    pub fn align_right(mut self) -> Self {
        self.align_item = Some(Alignment::Right);
        self
    }

    pub fn align_top_left(mut self) -> Self {
        self.align_item = Some(Alignment::TopLeft);
        self
    }

    pub fn align_top_center(mut self) -> Self {
        self.align_item = Some(Alignment::TopCenter);
        self
    }

    pub fn align_top_right(mut self) -> Self {
        self.align_item = Some(Alignment::TopRight);
        self
    }

    pub fn align_bottom_left(mut self) -> Self {
        self.align_item = Some(Alignment::BottomLeft);
        self
    }

    pub fn align_bottom_center(mut self) -> Self {
        self.align_item = Some(Alignment::BottomCenter);
        self
    }

    pub fn align_bottom_right(mut self) -> Self {
        self.align_item = Some(Alignment::BottomRight);
        self
    }

    /// Since the layout system is a single pass.
    /// AlignFlow::Center must have a fixed height.
    pub fn align_flow(mut self, align_flow: AlignFlow) -> Self {
        self.align_flow = Some(align_flow);
        self
    }

    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    pub fn skip_cull(mut self) -> Self {
        self.skip_cull = true;
        self
    }

    pub fn elastic(mut self, elastic: bool) -> Self {
        self.elastic = elastic;
        self
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn font(mut self, font: Font) -> Style {
        self.font = font;
        self
    }

    pub fn weight(mut self, weight: Weight) -> Style {
        self.font.weight = weight;
        self
    }

    pub fn bold(mut self) -> Style {
        self.font.weight = Weight::Bold;
        self
    }

    pub fn italic(mut self) -> Style {
        self.font.italic = true;
        self
    }

    pub fn opacity(mut self, opacity: u8) -> Self {
        self.opacity = Some(opacity);
        self
    }

}

impl Into<Style> for Rect {
    fn into(self) -> Style {
        style().bounds(self)
    }
}

macro_rules! impl_pad_swizzle {
    ($field:ident: $($name:ident => [$($edge:ident),+]);* $(;)?) => {
        impl Style {
            $(
                #[doc = concat!("Set ", stringify!($field), " for (", stringify!($($edge)+), ")")]
                pub fn $name(mut self, value: usize) -> Self {
                    let mut p = self.$field.unwrap_or_default();
                    $(
                        p.$edge = value;
                    )+
                    self.$field = Some(p);
                    self
                }
            )*
        }
    };
}

impl_pad_swizzle! {
    padding:
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

impl_pad_swizzle! {
    margin:
    mar   => [top, bottom, left, right];
    marh  => [left, right];
    marv  => [top, bottom];

    mart  => [top];
    marb  => [bottom];
    marl  => [left];
    marr  => [right];

    martl => [top, left];
    martr => [top, right];
    marbl => [bottom, left];
    marbr => [bottom, right];

    martb => [top, bottom];
    marlr => [left, right];
    marrl => [right, left];
}

pub fn bg(color: u32) -> Style {
    style().bg(color)
}

pub fn fg(color: u32) -> Style {
    style().fg(color)
}

pub fn font_size(font_size: usize) -> Style {
    style().font_size(font_size)
}

pub fn bounds(bounds: Rect) -> Style {
    style().bounds(bounds)
}

pub fn style() -> Style {
    Style::default()
}

pub fn s() -> Style {
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
    0xFF00_0000 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

pub const fn with_alpha(color: u32, a: u8) -> u32 {
    (color & 0x00FF_FFFF) | ((a as u32) << 24)
}

pub const fn alpha(color: u32) -> u8 {
    ((color >> 24) & 0xFF) as u8
}

pub const fn hex(color: &str) -> u32 {
    let bytes = color.as_bytes();
    let s = if !bytes.is_empty() && bytes[0] == b'#' {
        match color.split_at(1) {
            (_, rest) => rest,
        }
    } else {
        color
    };

    match s.len() {
        6 => match u32::from_str_radix(s, 16) {
            Ok(hex) => 0xFF00_0000 | hex,
            Err(_) => panic!("Invalid hex color."),
        },
        8 => match u32::from_str_radix(s, 16) {
            Ok(val) => {
                let rr = (val >> 24) & 0xFF;
                let gg = (val >> 16) & 0xFF;
                let bb = (val >> 8) & 0xFF;
                let aa = val & 0xFF;
                (aa << 24) | (rr << 16) | (gg << 8) | bb
            }
            Err(_) => panic!("Invalid hex color."),
        },
        _ => panic!("Hex color must be 6 or 8 characters."),
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
