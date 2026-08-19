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

/// Main-axis placement is not possible as it needs the extent of every sibling
/// before the first one is placed and this is a single pass layout system.
/// Use a reverse flow ([`Flow::Left`], [`Flow::Up`]) to align against the far end instead.
///
/// A flow can only align itself if it states its own cross size up front for the same reason.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Fit {
    /// Fill the box exactly, ignoring the aspect ratio.
    #[default]
    Stretch,
    /// Scale down to fit inside the box, keeping the aspect ratio.
    Contain,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Padding {
    pub top: usize,
    pub bottom: usize,
    pub left: usize,
    pub right: usize,
}

impl Padding {
    pub const fn new() -> Self {
        Padding {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        }
    }

    pub const fn axis(self, flow: Flow) -> i32 {
        if flow.vertical() {
            (self.top + self.bottom) as i32
        } else {
            (self.left + self.right) as i32
        }
    }
}

pub const fn pad(p: usize) -> Padding {
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

/// Where this box lands in the parent's flow and how big it is.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub x: Option<Size>,
    pub y: Option<Size>,
    pub width: Option<Size>,
    pub height: Option<Size>,
    pub padding: Option<Padding>,
    /// Grows the painted and interactable box outwards without reserving any layout space.
    pub margin: Option<Padding>,
    /// Overrides the parent flow's gap for the space reserved after this box.
    pub gap: Option<Size>,
    /// Where this box sits across the parent's flow axis.
    /// `None` inherits the parent flow's align_children.
    pub align: Option<Align>,
    pub depth: Option<usize>,
    /// Resolve fill against parent's outer bounds, ignoring padding.
    pub bleed: bool,
    /// Disabled in scroll views.
    pub clip: bool,
    /// Items are culled if a fixed size is given in flows.
    /// Optionally they can be re-enabled if needed.
    pub skip_cull: bool,
    /// Rubber-band past the edges of a scroll view and bounce back.
    pub elastic: bool,
}

impl Layout {
    pub const fn new() -> Self {
        Layout {
            x: None,
            y: None,
            width: None,
            height: None,
            padding: None,
            margin: None,
            gap: None,
            align: None,
            depth: None,
            bleed: false,
            clip: false,
            skip_cull: false,
            elastic: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Paint {
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

    pub opacity: Option<u8>,
}

impl Paint {
    pub const fn new() -> Self {
        Paint {
            fg: None,
            bg: None,
            radius: None,
            border: None,
            border_thickness: None,
            border_side: None,
            is_selected: false,
            selected: None,
            selected_border: None,
            hover: None,
            hover_border: None,
            opacity: None,
        }
    }
}

/// A container which arranges children along a flow direction.
#[derive(Debug, Clone, Copy)]
pub struct FlowStyle {
    pub layout: Layout,
    pub paint: Paint,
    pub align_children: Align,
}

#[derive(Debug, Clone, Copy)]
pub struct RectStyle {
    pub layout: Layout,
    pub paint: Paint,
}

#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub layout: Layout,
    pub paint: Paint,
    pub font: Font,
    pub font_size: Option<usize>,
    pub line_height: Option<usize>,
    /// Where the glyph run sits inside this widget's own content box.
    pub content: Option<Alignment>,
}

#[derive(Debug, Clone, Copy)]
pub struct ImageStyle {
    pub layout: Layout,
    pub paint: Paint,
    pub fit: Fit,
    /// Where the scaled image sits inside this widget's own content box.
    pub content: Option<Alignment>,
}

impl FlowStyle {
    pub const fn new() -> Self {
        FlowStyle {
            layout: Layout::new(),
            paint: Paint::new(),
            align_children: Align::Start,
        }
    }
}

impl RectStyle {
    pub const fn new() -> Self {
        RectStyle {
            layout: Layout::new(),
            paint: Paint::new(),
        }
    }
}

impl TextStyle {
    pub const fn new() -> Self {
        TextStyle {
            layout: Layout::new(),
            paint: Paint::new(),
            font: Font {
                id: 0,
                weight: Weight::Regular,
                italic: false,
            },
            font_size: None,
            line_height: None,
            content: None,
        }
    }
}

impl ImageStyle {
    pub const fn new() -> Self {
        ImageStyle {
            layout: Layout::new(),
            paint: Paint::new(),
            fit: Fit::Stretch,
            content: None,
        }
    }
}

pub const fn flow() -> FlowStyle {
    FlowStyle::new()
}

pub const fn rect() -> RectStyle {
    RectStyle::new()
}

pub const fn circle() -> RectStyle {
    RectStyle::new()
}

pub const fn gradient() -> RectStyle {
    RectStyle::new()
}

pub const fn text() -> TextStyle {
    TextStyle::new()
}

pub const fn image() -> ImageStyle {
    ImageStyle::new()
}

impl FlowStyle {
    pub const fn align_children(mut self, align: Align) -> Self {
        self.align_children = align;
        self
    }

    pub const fn children_start(mut self) -> Self {
        self.align_children = Align::Start;
        self
    }

    pub const fn children_center(mut self) -> Self {
        self.align_children = Align::Center;
        self
    }

    pub const fn children_end(mut self) -> Self {
        self.align_children = Align::End;
        self
    }
}

impl TextStyle {
    pub const fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub const fn weight(mut self, weight: Weight) -> Self {
        self.font.weight = weight;
        self
    }

    pub const fn bold(mut self) -> Self {
        self.font.weight = Weight::Bold;
        self
    }

    pub const fn italic(mut self) -> Self {
        self.font.italic = true;
        self
    }

    pub const fn font_size(mut self, font_size: usize) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub const fn line_height(mut self, line_height: usize) -> Self {
        self.line_height = Some(line_height);
        self
    }

    pub const fn content(mut self, alignment: Alignment) -> Self {
        self.content = Some(alignment);
        self
    }

    pub const fn content_left(mut self) -> Self {
        self.content = Some(Alignment::Left);
        self
    }

    pub const fn content_center(mut self) -> Self {
        self.content = Some(Alignment::Center);
        self
    }

    pub const fn content_right(mut self) -> Self {
        self.content = Some(Alignment::Right);
        self
    }

    pub const fn content_top_left(mut self) -> Self {
        self.content = Some(Alignment::TopLeft);
        self
    }

    pub const fn content_top_center(mut self) -> Self {
        self.content = Some(Alignment::TopCenter);
        self
    }

    pub const fn content_top_right(mut self) -> Self {
        self.content = Some(Alignment::TopRight);
        self
    }

    pub const fn content_bottom_left(mut self) -> Self {
        self.content = Some(Alignment::BottomLeft);
        self
    }

    pub const fn content_bottom_center(mut self) -> Self {
        self.content = Some(Alignment::BottomCenter);
        self
    }

    pub const fn content_bottom_right(mut self) -> Self {
        self.content = Some(Alignment::BottomRight);
        self
    }
}

impl ImageStyle {
    pub const fn fit(mut self, fit: Fit) -> Self {
        self.fit = fit;
        self
    }

    pub const fn contain(mut self) -> Self {
        self.fit = Fit::Contain;
        self
    }

    pub const fn content(mut self, alignment: Alignment) -> Self {
        self.content = Some(alignment);
        self
    }

    pub const fn content_left(mut self) -> Self {
        self.content = Some(Alignment::Left);
        self
    }

    pub const fn content_center(mut self) -> Self {
        self.content = Some(Alignment::Center);
        self
    }

    pub const fn content_right(mut self) -> Self {
        self.content = Some(Alignment::Right);
        self
    }
}

pub const trait Boxed: Sized {
    fn layout_mut(&mut self) -> &mut Layout;

    #[inline]
    fn bounds(mut self, bounds: Rect) -> Self {
        let l = self.layout_mut();
        l.x = Some(Size::Pixel(bounds.x));
        l.y = Some(Size::Pixel(bounds.y));
        l.width = Some(Size::Pixel(bounds.width));
        l.height = Some(Size::Pixel(bounds.height));
        self
    }

    #[inline]
    fn x(mut self, x: impl [const] IntoSize) -> Self {
        self.layout_mut().x = x.into_size();
        self
    }

    #[inline]
    fn y(mut self, y: impl [const] IntoSize) -> Self {
        self.layout_mut().y = y.into_size();
        self
    }

    #[inline]
    fn w(mut self, w: impl [const] IntoSize) -> Self {
        self.layout_mut().width = w.into_size();
        self
    }

    #[inline]
    fn h(mut self, h: impl [const] IntoSize) -> Self {
        self.layout_mut().height = h.into_size();
        self
    }

    #[inline]
    fn wh(mut self, wh: impl [const] IntoSize) -> Self {
        let wh = wh.into_size();
        let l = self.layout_mut();
        l.width = wh;
        l.height = wh;
        self
    }

    #[inline]
    fn width(mut self, w: impl [const] IntoSize) -> Self {
        self.layout_mut().width = w.into_size();
        self
    }

    #[inline]
    fn height(mut self, h: impl [const] IntoSize) -> Self {
        self.layout_mut().height = h.into_size();
        self
    }

    #[inline]
    fn fill(mut self) -> Self {
        let l = self.layout_mut();
        l.width = Some(Size::Fill);
        l.height = Some(Size::Fill);
        self
    }

    #[inline]
    fn fillw(mut self) -> Self {
        self.layout_mut().width = Some(Size::Fill);
        self
    }

    #[inline]
    fn fillh(mut self) -> Self {
        self.layout_mut().height = Some(Size::Fill);
        self
    }

    #[inline]
    fn gap(mut self, gap: impl [const] IntoSize) -> Self {
        self.layout_mut().gap = gap.into_size();
        self
    }

    /// Where this box sits across the parent's flow axis.
    #[inline]
    fn align(mut self, align: Align) -> Self {
        self.layout_mut().align = Some(align);
        self
    }

    #[inline]
    fn align_start(mut self) -> Self {
        self.layout_mut().align = Some(Align::Start);
        self
    }

    #[inline]
    fn align_center(mut self) -> Self {
        self.layout_mut().align = Some(Align::Center);
        self
    }

    #[inline]
    fn align_end(mut self) -> Self {
        self.layout_mut().align = Some(Align::End);
        self
    }

    #[inline]
    fn bleed(mut self) -> Self {
        self.layout_mut().bleed = true;
        self
    }

    #[inline]
    fn clip(mut self, clip: bool) -> Self {
        self.layout_mut().clip = clip;
        self
    }

    #[inline]
    fn skip_cull(mut self) -> Self {
        self.layout_mut().skip_cull = true;
        self
    }

    #[inline]
    fn elastic(mut self, elastic: bool) -> Self {
        self.layout_mut().elastic = elastic;
        self
    }

    #[inline]
    fn depth(mut self, depth: usize) -> Self {
        self.layout_mut().depth = Some(depth);
        self
    }

    #[inline]
    fn padding(mut self, padding: Padding) -> Self {
        self.layout_mut().padding = Some(padding);
        self
    }

    #[inline]
    fn margin(mut self, margin: Padding) -> Self {
        self.layout_mut().margin = Some(margin);
        self
    }

    #[rustfmt::skip] #[inline]    fn pad(self, v: usize)   -> Self { self.pad_edges(v, true, true, true, true) }
    #[rustfmt::skip] #[inline]    fn padh(self, v: usize)  -> Self { self.pad_edges(v, false, false, true, true) }
    #[rustfmt::skip] #[inline]    fn padv(self, v: usize)  -> Self { self.pad_edges(v, true, true, false, false) }
    #[rustfmt::skip] #[inline]    fn padt(self, v: usize)  -> Self { self.pad_edges(v, true, false, false, false) }
    #[rustfmt::skip] #[inline]    fn padb(self, v: usize)  -> Self { self.pad_edges(v, false, true, false, false) }
    #[rustfmt::skip] #[inline]    fn padl(self, v: usize)  -> Self { self.pad_edges(v, false, false, true, false) }
    #[rustfmt::skip] #[inline]    fn padr(self, v: usize)  -> Self { self.pad_edges(v, false, false, false, true) }
    #[rustfmt::skip] #[inline]    fn padtl(self, v: usize) -> Self { self.pad_edges(v, true, false, true, false) }
    #[rustfmt::skip] #[inline]    fn padtr(self, v: usize) -> Self { self.pad_edges(v, true, false, false, true) }
    #[rustfmt::skip] #[inline]    fn padbl(self, v: usize) -> Self { self.pad_edges(v, false, true, true, false) }
    #[rustfmt::skip] #[inline]    fn padbr(self, v: usize) -> Self { self.pad_edges(v, false, true, false, true) }

    #[rustfmt::skip] #[inline]    fn padtb(self, v: usize) -> Self { self.pad_edges(v, true, true, false, false) }
    #[rustfmt::skip] #[inline]    fn padlr(self, v: usize) -> Self { self.pad_edges(v, false, false, true, true) }

    #[rustfmt::skip] #[inline]    fn mar(self, v: usize)   -> Self { self.mar_edges(v, true, true, true, true) }
    #[rustfmt::skip] #[inline]    fn marh(self, v: usize)  -> Self { self.mar_edges(v, false, false, true, true) }
    #[rustfmt::skip] #[inline]    fn marv(self, v: usize)  -> Self { self.mar_edges(v, true, true, false, false) }
    #[rustfmt::skip] #[inline]    fn mart(self, v: usize)  -> Self { self.mar_edges(v, true, false, false, false) }
    #[rustfmt::skip] #[inline]    fn marb(self, v: usize)  -> Self { self.mar_edges(v, false, true, false, false) }
    #[rustfmt::skip] #[inline]    fn marl(self, v: usize)  -> Self { self.mar_edges(v, false, false, true, false) }
    #[rustfmt::skip] #[inline]    fn marr(self, v: usize)  -> Self { self.mar_edges(v, false, false, false, true) }
    #[rustfmt::skip] #[inline]    fn martl(self, v: usize) -> Self { self.mar_edges(v, true, false, true, false) }
    #[rustfmt::skip] #[inline]    fn martr(self, v: usize) -> Self { self.mar_edges(v, true, false, false, true) }
    #[rustfmt::skip] #[inline]    fn marbl(self, v: usize) -> Self { self.mar_edges(v, false, true, true, false) }
    #[rustfmt::skip] #[inline]    fn marbr(self, v: usize) -> Self { self.mar_edges(v, false, true, false, true) }

    #[rustfmt::skip] #[inline]    fn martb(self, v: usize) -> Self { self.mar_edges(v, true, true, false, false) }
    #[rustfmt::skip] #[inline]    fn marlr(self, v: usize) -> Self { self.mar_edges(v, false, false, true, true) }

    #[inline]
    fn pad_edges(mut self, v: usize, top: bool, bottom: bool, left: bool, right: bool) -> Self {
        let layout = self.layout_mut();
        let mut p = match layout.padding {
            Some(p) => p,
            None => Padding::new(),
        };
        if top {
            p.top = v;
        }
        if bottom {
            p.bottom = v;
        }
        if left {
            p.left = v;
        }
        if right {
            p.right = v;
        }
        layout.padding = Some(p);
        self
    }

    #[inline]
    fn mar_edges(mut self, v: usize, top: bool, bottom: bool, left: bool, right: bool) -> Self {
        let layout = self.layout_mut();
        let mut p = match layout.margin {
            Some(p) => p,
            None => Padding::new(),
        };
        if top {
            p.top = v;
        }
        if bottom {
            p.bottom = v;
        }
        if left {
            p.left = v;
        }
        if right {
            p.right = v;
        }
        layout.margin = Some(p);
        self
    }
}

/// Fill, border and interaction colours, shared by every style.
pub const trait Painted: Sized {
    fn paint_mut(&mut self) -> &mut Paint;

    #[inline]
    fn bg(mut self, color: impl [const] IntoColor) -> Self {
        self.paint_mut().bg = color.into_color();
        self
    }

    #[inline]
    fn fg(mut self, color: impl [const] IntoColor) -> Self {
        self.paint_mut().fg = color.into_color();
        self
    }

    #[inline]
    fn radius(mut self, r: usize) -> Self {
        self.paint_mut().radius = Some(r);
        self
    }

    #[inline]
    fn border(mut self, color: impl [const] IntoColor) -> Self {
        self.paint_mut().border = color.into_color();
        self
    }

    #[inline]
    fn border_thickness(mut self, thickness: usize) -> Self {
        self.paint_mut().border_thickness = Some(thickness);
        self
    }

    #[inline]
    fn border_side(mut self, side: u8) -> Self {
        self.paint_mut().border_side = Some(side);
        self
    }

    #[inline]
    fn is_selected(mut self, is_selected: bool) -> Self {
        self.paint_mut().is_selected = is_selected;
        self
    }

    #[inline]
    fn selected(mut self, color: impl [const] IntoColor) -> Self {
        self.paint_mut().selected = color.into_color();
        self
    }

    #[inline]
    fn selected_border(mut self, color: impl [const] IntoColor) -> Self {
        self.paint_mut().selected_border = color.into_color();
        self
    }

    #[inline]
    fn hover(mut self, color: impl [const] IntoColor) -> Self {
        self.paint_mut().hover = color.into_color();
        self
    }

    #[inline]
    fn hover_border(mut self, color: impl [const] IntoColor) -> Self {
        self.paint_mut().hover_border = color.into_color();
        self
    }

    #[inline]
    fn opacity(mut self, opacity: u8) -> Self {
        self.paint_mut().opacity = Some(opacity);
        self
    }
}

impl const Boxed for FlowStyle {
    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl const Boxed for RectStyle {
    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl const Boxed for TextStyle {
    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl const Boxed for ImageStyle {
    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }
}

impl const Painted for FlowStyle {
    fn paint_mut(&mut self) -> &mut Paint {
        &mut self.paint
    }
}

impl const Painted for RectStyle {
    fn paint_mut(&mut self) -> &mut Paint {
        &mut self.paint
    }
}

impl const Painted for TextStyle {
    fn paint_mut(&mut self) -> &mut Paint {
        &mut self.paint
    }
}

impl const Painted for ImageStyle {
    fn paint_mut(&mut self) -> &mut Paint {
        &mut self.paint
    }
}

impl From<Rect> for FlowStyle {
    fn from(bounds: Rect) -> Self {
        flow().bounds(bounds)
    }
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

/// Lets colour setters take a colour or `None` to unset one.
/// Allows users to write `.bg(red())`, `.bg(hex("#101011"))` or `.bg(None)`.
pub const trait IntoColor {
    fn into_color(self) -> Option<u32>;
}

impl const IntoColor for u32 {
    fn into_color(self) -> Option<u32> {
        Some(self)
    }
}

impl const IntoColor for Option<u32> {
    fn into_color(self) -> Option<u32> {
        self
    }
}

/// Helper trait to simplify writing Size constraints.
/// Allows users to write None, 13, 0.2, -32 or Size::*.
pub const trait IntoSize {
    fn into_size(self) -> Option<Size>;
}

impl const IntoSize for Size {
    fn into_size(self) -> Option<Size> {
        Some(self)
    }
}

impl const IntoSize for f32 {
    fn into_size(self) -> Option<Size> {
        Some(Size::Percentage(self))
    }
}

//Yeah keep it for now, I'll think about it later...
impl const IntoSize for i32 {
    fn into_size(self) -> Option<Size> {
        if self < 0 {
            Some(Size::FillMinus(self))
        } else {
            Some(Size::Pixel(self))
        }
    }
}

impl const IntoSize for usize {
    fn into_size(self) -> Option<Size> {
        Some(Size::Pixel(self as i32))
    }
}

/// One styled run inside [`FrameContext::lines`].
pub fn line<'a>(content: impl Into<Cow<'a, str>>, style: TextStyle) -> Line<'a> {
    Line {
        content: content.into(),
        style,
    }
}

#[derive(Clone, Debug)]
pub struct Line<'a> {
    pub content: Cow<'a, str>,
    pub style: TextStyle,
}

impl<'a> From<&'a str> for Line<'a> {
    fn from(content: &'a str) -> Self {
        Line {
            content: Cow::Borrowed(content),
            style: TextStyle::new(),
        }
    }
}

impl From<String> for Line<'static> {
    fn from(content: String) -> Self {
        Line {
            content: Cow::Owned(content),
            style: TextStyle::new(),
        }
    }
}

impl<'a> From<Cow<'a, str>> for Line<'a> {
    fn from(content: Cow<'a, str>) -> Self {
        Line {
            content,
            style: TextStyle::new(),
        }
    }
}
