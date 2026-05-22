//TODO: Fix this issue in mini
#![allow(unexpected_cfgs)]

pub mod style;
pub use style::*;

pub mod platform;
pub use platform::*;

use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Flow {
    Down,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub bounds: Rect,
    pub flow: Flow,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub max_child_width: usize,
    pub max_child_height: usize,
}

pub enum Command {
    Rect {
        rect: Rect,
        color: u32,
    },
    Triangle {
        a: (usize, usize),
        b: (usize, usize),
        c: (usize, usize),
        color: u32,
    },
    RectOutline {
        rect: Rect,
        color: u32,
    },
    Text {
        text: Cow<'static, str>,
        x: usize,
        y: usize,
        color: u32,
        size: usize,
    },
}

pub struct State {
    pub clicked: bool,
    pub hovered: bool,
    pub rect: Rect,
}

pub const FONT: &[u8] = include_bytes!("../fonts/Aptos.ttf");

pub fn create_ctx(title: &str, width: usize, height: usize) -> &'static mut Context {
    unsafe {
        #[cfg(target_os = "windows")]
        let window = create_window(title, 0, 0, width as i32, height as i32, WindowStyle::DEFAULT);

        #[cfg(target_os = "macos")]
        let window = Box::pin(Window::new(title, width, height));

        CTX.window = Some(window);
        CTX.font = Some(fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap());

        &mut *(&raw mut CTX)
    }
}

pub struct Context {
    pub commands: Vec<Command>,
    pub font: Option<fontdue::Font>,
    pub window: Option<std::pin::Pin<Box<Window>>>,
    pub layout_stack: Vec<Frame>,
    pub glyph_cache: Option<HashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    pub glyph_cache_subpixel: Option<HashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    pub default_font_size: usize,
}

impl Context {
    /// Walk the layout forward by an explicit size and return the screen-space bounding box.
    pub fn walk_layout(&mut self, width: usize, height: usize) -> Rect {
        let frame = self.layout_stack.last_mut().expect("No active layout frame");
        let rect = Rect::new(frame.cursor_x, frame.cursor_y, width, height);

        match frame.flow {
            Flow::Down => {
                frame.cursor_y += height;
                frame.max_child_width = frame.max_child_width.max(width);
                frame.max_child_height += height;
            }
            Flow::Right => {
                frame.cursor_x += width;
                frame.max_child_width += width;
                frame.max_child_height = frame.max_child_height.max(height);
            }
        }
        rect
    }

    pub fn clicked(&mut self, rect: Rect) -> bool {
        let window = self.window.as_mut().unwrap();
        window.left_mouse.clicked(rect)
    }

    pub fn hovered(&self, rect: Rect) -> bool {
        let window = self.window.as_ref().unwrap();
        window.mouse_position.intersects(rect)
    }

    //TODO: This should really be paint_rect or something.
    //Users should be able to use the layout system to render rectangles.
    pub fn rect(&mut self, rect: Rect, color: u32) {
        self.commands.push(Command::Rect { rect, color });
    }

    pub fn triangle(&mut self, a: (usize, usize), b: (usize, usize), c: (usize, usize), color: u32) {
        self.commands.push(Command::Triangle { a, b, c, color });
    }

    pub fn text_aligned(
        &mut self,
        dest: Rect,
        text: impl Into<Cow<'static, str>>,
        color: u32,
        font_size: usize,
        alignment: Alignment,
    ) {
        let text = text.into();
        let window = self.window.as_mut().unwrap();
        let cache_map = self.glyph_cache_subpixel.get_or_insert_with(HashMap::new);
        let font = self.font.as_ref().expect("Font asset missing from Context");
        let text_metrics = draw_text_subpixel(
            &text,
            font,
            0,
            0,
            font_size,
            1.0,
            window.width(),
            &mut [],
            color,
            true,
            cache_map,
        );

        let rect = align_rect(dest, text_metrics.width, text_metrics.height, alignment);
        self.commands.push(Command::Text {
            text,
            x: rect.x,
            y: rect.y,
            color,
            size: font_size,
        });
    }

    pub fn button<'a>(&mut self, text: impl Into<Cow<'static, str>>, style: Style) -> State {
        let text = text.into();
        let width_ctx = self.width();
        let cache_map = self.glyph_cache_subpixel.get_or_insert_with(HashMap::new);
        let font_size = style.font_size.unwrap_or(self.default_font_size);
        let text_metrics = draw_text_subpixel(
            &text,
            self.font.as_ref().unwrap(),
            0,
            0,
            font_size,
            1.0,
            width_ctx,
            &mut [],
            white(),
            true,
            cache_map,
        );

        let padding = style.padding.unwrap_or_default();
        let width = style.width.unwrap_or(text_metrics.width + padding.left + padding.right);
        let height = style
            .height
            .unwrap_or(text_metrics.height + padding.top + padding.bottom);

        let rect = self.walk_layout(width, height);
        let window = self.window.as_mut().unwrap();
        let hovered = window.mouse_position.intersects(rect);
        let clicked = window.left_mouse.clicked(rect);
        let bg = if hovered && style.hover.is_some() {
            style.hover
        } else {
            style.bg
        };

        if let Some(color) = bg {
            self.commands.push(Command::Rect { rect, color });
        }

        self.text_aligned(rect, text, style.fg.unwrap_or(white()), font_size, Alignment::Center);

        State { clicked, hovered, rect }
    }

    pub fn list_item(
        &mut self,
        text: impl Into<Cow<'static, str>>,
        selected: bool,
        width_override: usize,
        style: Style,
    ) -> State {
        let text = text.into();
        let font_size = style.font_size.unwrap_or(self.default_font_size);
        let padding = style.padding.unwrap_or_default();
        let allocated_h = font_size + padding.top + padding.bottom;
        let rect = self.walk_layout(width_override, allocated_h);
        let window = self.window.as_mut().unwrap();
        let hovered = window.mouse_position.intersects(rect);
        let clicked = window.left_mouse.clicked(rect);

        let bg = if selected {
            style.selected
        } else if hovered {
            style.hover
        } else {
            style.bg
        };

        if let Some(color) = bg {
            self.commands.push(Command::Rect { rect, color });
        }

        if selected {
            if let Some(color) = style.selected_border {
                self.commands.push(Command::RectOutline { rect, color });
            }
        }

        self.text_aligned(
            rect,
            text,
            style.fg.unwrap_or(white()),
            font_size,
            Alignment::Left { pad: padding.left },
        );

        State { clicked, hovered, rect }
    }

    pub fn spacer(&mut self, style: Style) {
        let frame = self.layout_stack.last_mut().expect("No active layout frame");

        let remaining_width = (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x);
        let remaining_height = (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y);

        let width = style.width.unwrap_or(remaining_width);
        let height = style.height.unwrap_or(remaining_height);
        let rect = self.walk_layout(width, height);

        if let Some(color) = style.bg {
            self.commands.push(Command::Rect { rect, color });
        }

        if let Some(color) = style.border {
            self.commands.push(Command::RectOutline { rect, color });
        }
    }

    pub fn fill(&mut self, color: u32) {
        let window = self.window.as_mut().unwrap();
        window.buffer.fill(color);
    }

    pub fn draw(&mut self) {
        let window = self.window.as_mut().unwrap();
        window.draw();
        window.vsync();
    }

    pub fn width(&mut self) -> usize {
        let window = self.window.as_ref().unwrap();
        window.width()
    }

    pub fn height(&mut self) -> usize {
        let window = self.window.as_ref().unwrap();
        window.height()
    }
}

pub static mut CTX: Context = Context {
    commands: Vec::new(),
    font: None,
    window: None,
    layout_stack: Vec::new(),
    glyph_cache: None,
    glyph_cache_subpixel: None,
    default_font_size: 32,
};

pub fn begin_ui(fill_color: u32) {
    let ctx = unsafe { &mut *(&raw mut CTX) };
    let bounds = Rect::new(0, 0, ctx.width(), ctx.height());
    ctx.fill(fill_color);
    ctx.layout_stack.clear();
    ctx.layout_stack.push(Frame {
        bounds,
        flow: Flow::Down,
        cursor_x: 0,
        cursor_y: 0,
        max_child_width: 0,
        max_child_height: 0,
    });
}

pub fn begin_layout(flow: Flow) {
    let ctx = unsafe { &mut *(&raw mut CTX) };
    let parent = ctx.layout_stack.last().expect("Layout stack empty");
    let new_frame = Frame {
        bounds: Rect::new(
            parent.cursor_x,
            parent.cursor_y,
            parent.bounds.width,
            parent.bounds.height,
        ),
        flow,
        cursor_x: parent.cursor_x,
        cursor_y: parent.cursor_y,
        max_child_width: 0,
        max_child_height: 0,
    };
    ctx.layout_stack.push(new_frame);
}

pub fn begin_layout_with_bounds(flow: Flow, explicit_bounds: Rect) {
    let ctx = unsafe { &mut *(&raw mut CTX) };
    let new_frame = Frame {
        bounds: explicit_bounds,
        flow,
        cursor_x: explicit_bounds.x,
        cursor_y: explicit_bounds.y,
        max_child_width: 0,
        max_child_height: 0,
    };
    ctx.layout_stack.push(new_frame);
}

pub fn begin_grid_cell(col: usize, row: usize, col_width: usize, row_height: usize, grid_bounds: Rect, flow: Flow) {
    let cell_x = grid_bounds.x + (col * col_width);
    let cell_y = grid_bounds.y + (row * row_height);

    let cell_w = col_width.min(grid_bounds.width.saturating_sub(col * col_width));
    let cell_h = row_height.min(grid_bounds.height.saturating_sub(row * row_height));

    begin_layout_with_bounds(flow, Rect::new(cell_x, cell_y, cell_w, cell_h));
}

pub fn end_layout() {
    let ctx = unsafe { &mut *(&raw mut CTX) };
    let finished = ctx.layout_stack.pop().expect("Layout underflow");

    if let Some(parent) = ctx.layout_stack.last_mut() {
        match parent.flow {
            Flow::Down => {
                parent.cursor_y += finished.max_child_height;
                parent.max_child_width = parent.max_child_width.max(finished.max_child_width);
                parent.max_child_height += finished.max_child_height;
            }
            Flow::Right => {
                parent.cursor_x += finished.max_child_width;
                parent.max_child_width += finished.max_child_width;
                parent.max_child_height = parent.max_child_height.max(finished.max_child_height);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left { pad: usize },
    Center,
    Right { pad: usize },
    TopLeft { padh: usize, padv: usize },
    TopCenter { pad: usize },
    TopRight { padh: usize, padv: usize },
    BottomLeft { padh: usize, padv: usize },
    BottomCenter { pad: usize },
    BottomRight { padh: usize, padv: usize },
}

pub fn align_rect(parent: Rect, child_w: usize, child_h: usize, alignment: Alignment) -> Rect {
    let mid_x = || parent.x + (parent.width / 2) - (child_w / 2);
    let mid_y = || parent.y + (parent.height / 2) - (child_h / 2);
    let right_edge = || parent.x + parent.width - child_w;
    let bottom_edge = || parent.y + parent.height - child_h;

    let (x, y) = match alignment {
        Alignment::Left { pad } => (parent.x + pad, mid_y()),
        Alignment::Center => (mid_x(), mid_y()),
        Alignment::Right { pad } => (right_edge().saturating_sub(pad), mid_y()),
        Alignment::TopLeft { padh, padv } => (parent.x + padh, parent.y + padv),
        Alignment::TopCenter { pad } => (mid_x(), parent.y + pad),
        Alignment::TopRight { padh, padv } => (right_edge().saturating_sub(padh), parent.y + padv),
        Alignment::BottomLeft { padh, padv } => (parent.x + padh, bottom_edge().saturating_sub(padv)),
        Alignment::BottomCenter { pad } => (mid_x(), bottom_edge().saturating_sub(pad)),
        Alignment::BottomRight { padh, padv } => {
            (right_edge().saturating_sub(padh), bottom_edge().saturating_sub(padv))
        }
    };

    Rect::new(x, y, child_w, child_h)
}

pub fn exit() -> bool {
    let ctx = unsafe { &mut *(&raw mut CTX) };
    let window = ctx.window.as_mut().unwrap();

    if let Some(event) = window.event() {
        return match event {
            Event::Quit | Event::Input(Key::Escape, _) => true,
            _ => false,
        };
    }
    false
}

pub fn draw_cmd() {
    let ctx = unsafe { &mut *(&raw mut CTX) };
    let ctx_width = ctx.width();
    let ctx_height = ctx.height();
    let window = ctx.window.as_mut().unwrap();

    for cmd in core::mem::take(&mut ctx.commands) {
        match cmd {
            Command::RectOutline { rect, color } => draw_rect_outline(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                ctx_width,
                &mut window.buffer,
                color,
            ),
            Command::Rect { rect, color } => draw_rect(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                ctx_width,
                ctx_height,
                &mut window.buffer,
                color,
            ),
            Command::Text {
                text,
                x,
                y,
                color,
                size,
            } => {
                let cache_map = ctx.glyph_cache_subpixel.get_or_insert_with(HashMap::new);
                draw_text_subpixel(
                    &text,
                    ctx.font.as_ref().unwrap(),
                    x,
                    y,
                    size,
                    window.display_scale(),
                    ctx_width,
                    &mut window.buffer,
                    color,
                    false,
                    cache_map,
                );
            }
            Command::Triangle {
                a: (ax, ay),
                b: (bx, by),
                c: (cx, cy),
                color,
            } => {
                //
                draw_triangle_sdf(&mut window.buffer, ctx_width, ctx_height, ax, ay, bx, by, cx, cy, color)
            }
        };
    }
}

pub fn draw_rect(
    x: usize,
    y: usize,
    mut width: usize,
    mut height: usize,
    window_width: usize,
    window_height: usize,
    buffer: &mut [u32],
    color: u32,
) {
    if x > window_width || y > window_height {
        return;
    }
    if x + width > window_width {
        width = window_width.saturating_sub(x);
    }
    if y + height > window_height {
        height = window_height.saturating_sub(y);
    }

    for i in y..y + height {
        let pos = x + window_width * i;
        if let Some(buffer) = buffer.get_mut(pos..pos + width) {
            buffer.fill(color);
        }
    }
}

pub fn draw_rect_outline(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    window_width: usize,
    buffer: &mut [u32],
    color: u32,
) {
    if height == 0 || width == 0 {
        return;
    }
    //Draw the first line
    let pos = x + window_width * y;
    if let Some(buffer) = buffer.get_mut(pos..=pos + width) {
        buffer.fill(color);
    }

    //Draw the middle pixels
    //Skip the first line.
    for i in (y + 1)..(y + height - 1) {
        let left = x + window_width * i;
        if let Some(buffer) = buffer.get_mut(left) {
            *buffer = color;
        }

        let right = x + width + window_width * i;
        if let Some(buffer) = buffer.get_mut(right) {
            *buffer = color;
        }
    }

    //Draw the last line
    if height > 1 {
        let pos = x + window_width * (y + height - 1);
        if let Some(buffer) = buffer.get_mut(pos..=pos + width) {
            buffer.fill(color);
        }
    }
}

pub fn draw_triangle_scanline(
    buffer: &mut [u32],
    window_width: usize,
    window_height: usize,
    mut x0: usize,
    mut y0: usize,
    mut x1: usize,
    mut y1: usize,
    mut x2: usize,
    mut y2: usize,
    color: u32,
) {
    mini::profile!();
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
        std::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y2 {
        std::mem::swap(&mut y0, &mut y2);
        std::mem::swap(&mut x0, &mut x2);
    }
    if y1 > y2 {
        std::mem::swap(&mut y1, &mut y2);
        std::mem::swap(&mut x1, &mut x2);
    }

    if y0 >= window_height || y2 == 0 {
        return;
    }

    let total_height = y2 - y0;
    if total_height == 0 {
        return;
    }

    for y in y0..=y2 {
        if y >= window_height {
            break;
        }

        let second_half = y > y1 || y1 == y0;
        let segment_height = if second_half { y2 - y1 } else { y1 - y0 };
        let alpha = (y - y0) as f32 / total_height as f32;
        let beta = if segment_height == 0 {
            1.0
        } else {
            (y - if second_half { y1 } else { y0 }) as f32 / segment_height as f32
        };

        let mut ax = x0 as f32 + (x2 as f32 - x0 as f32) * alpha;
        let mut bx = if second_half {
            x1 as f32 + (x2 as f32 - x1 as f32) * beta
        } else {
            x0 as f32 + (x1 as f32 - x0 as f32) * beta
        };

        if ax > bx {
            std::mem::swap(&mut ax, &mut bx);
        }

        let left = (ax as usize).min(window_width);
        let right = (bx as usize).min(window_width);

        if left < right && left < window_width {
            let row_start = y * window_width + left;
            let row_end = y * window_width + right;
            if let Some(slice) = buffer.get_mut(row_start..row_end) {
                slice.fill(color);
            }
        }
    }
}

pub fn draw_triangle_sdf(
    buffer: &mut [u32],
    window_width: usize,
    window_height: usize,
    mut x0: usize,
    mut y0: usize,
    mut x1: usize,
    mut y1: usize,
    mut x2: usize,
    mut y2: usize,
    color: u32,
) {
    mini::profile!();
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
        std::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y2 {
        std::mem::swap(&mut y0, &mut y2);
        std::mem::swap(&mut x0, &mut x2);
    }
    if y1 > y2 {
        std::mem::swap(&mut y1, &mut y2);
        std::mem::swap(&mut x1, &mut x2);
    }

    if y0 >= window_height || y2 == 0 || y0 == y2 {
        return;
    }

    let ax = x0 as f32;
    let ay = y0 as f32;
    let mut bx = x1 as f32;
    let mut by = y1 as f32;
    let mut cx = x2 as f32;
    let mut cy = y2 as f32;

    let area = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
    if area < 0.0 {
        std::mem::swap(&mut bx, &mut cx);
        std::mem::swap(&mut by, &mut cy);
    }

    let dx0 = bx - ax;
    let dy0 = by - ay;
    let dx1 = cx - bx;
    let dy1 = cy - by;
    let dx2 = ax - cx;
    let dy2 = ay - cy;

    let len0 = (dx0 * dx0 + dy0 * dy0).sqrt();
    let len1 = (dx1 * dx1 + dy1 * dy1).sqrt();
    let len2 = (dx2 * dx2 + dy2 * dy2).sqrt();

    if len0 == 0.0 || len1 == 0.0 || len2 == 0.0 {
        return;
    }

    let src_r = ((color >> 16 & 0xFF) as f32 / 255.0).powi(2);
    let src_g = ((color >> 8 & 0xFF) as f32 / 255.0).powi(2);
    let src_b = ((color & 0xFF) as f32 / 255.0).powi(2);

    let total_height = y2 - y0;
    let step_long = (x2 as f32 - x0 as f32) / total_height as f32;
    let mut x_long = x0 as f32;

    let height_top = y1 - y0;
    let step_short_top = if height_top > 0 {
        (x1 as f32 - x0 as f32) / height_top as f32
    } else {
        0.0
    };
    let mut x_short = x0 as f32;

    let pad_long = 1.5 + step_long.abs();

    for y in y0..=y2 {
        if y >= window_height {
            break;
        }
        let py = y as f32 + 0.5;

        if y == y1 && y1 < y2 {
            x_short = x1 as f32;
        }

        let step_short = if y < y1 {
            step_short_top
        } else {
            let height_bottom = y2 - y1;
            if height_bottom > 0 {
                (x2 as f32 - x1 as f32) / height_bottom as f32
            } else {
                0.0
            }
        };

        let pad_short = 1.5 + step_short.abs();

        let left_bound = (x_long - pad_long).min(x_short - pad_short);
        let right_bound = (x_long + pad_long).max(x_short + pad_short);

        let min_x = (left_bound.max(0.0) as usize).min(window_width);
        let max_x = (right_bound.max(0.0) as usize).min(window_width);

        let mut solid_start = window_width;
        let mut solid_end = 0;

        for x in min_x..max_x {
            let px = x as f32 + 0.5;

            let dist0 = (dx0 * (py - ay) - dy0 * (px - ax)) / len0;
            let dist1 = (dx1 * (py - by) - dy1 * (px - bx)) / len1;
            let dist2 = (dx2 * (py - cy) - dy2 * (px - cx)) / len2;

            let cov0 = (dist0 + 0.5).clamp(0.0, 1.0);
            let cov1 = (dist1 + 0.5).clamp(0.0, 1.0);
            let cov2 = (dist2 + 0.5).clamp(0.0, 1.0);

            let alpha = cov0 * cov1 * cov2;

            if alpha >= 0.999 {
                if solid_start == window_width {
                    solid_start = x;
                }
                solid_end = x + 1;
            } else if alpha > 0.0 {
                let idx = y * window_width + x;
                if let Some(bg) = buffer.get_mut(idx) {
                    let bg_r = (((*bg >> 16) & 0xFF) as f32 / 255.0).powi(2);
                    let bg_g = (((*bg >> 8) & 0xFF) as f32 / 255.0).powi(2);
                    let bg_b = ((*bg & 0xFF) as f32 / 255.0).powi(2);

                    let out_r = (src_r * alpha) + (bg_r * (1.0 - alpha));
                    let out_g = (src_g * alpha) + (bg_g * (1.0 - alpha));
                    let out_b = (src_b * alpha) + (bg_b * (1.0 - alpha));

                    *bg = ((out_r.sqrt() * 255.0) as u32) << 16
                        | ((out_g.sqrt() * 255.0) as u32) << 8
                        | ((out_b.sqrt() * 255.0) as u32);
                }
            }
        }

        if solid_start < solid_end {
            let start_idx = y * window_width + solid_start;
            let end_idx = y * window_width + solid_end;
            if let Some(slice) = buffer.get_mut(start_idx..end_idx) {
                slice.fill(color);
            }
        }

        x_long += step_long;
        x_short += step_short;
    }
}

pub const fn scale(value: usize, scale: f32) -> usize {
    (value as f32 * scale).round() as usize
}

pub const fn blend(color: u8, alpha: u8, bg_color: u8, bg_alpha: u8) -> u8 {
    ((color as f32 * alpha as f32 + bg_color as f32 * bg_alpha as f32) / 255.0).round() as u8
}

pub fn draw_text(
    text: &str,
    font: &fontdue::Font,
    x: usize,
    y: usize,
    font_size: usize,
    display_scale: f32,
    window_width: usize,
    buffer: &mut [u32],
    color: u32,
    skip_draw: bool,
    cache_map: &mut HashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>,
) -> Rect {
    if text.is_empty() || font_size == 0 {
        return Rect::new(0, 0, 0, 0);
    }

    let x = scale(x, display_scale);
    let y = scale(y, display_scale);
    let font_size = scale(font_size, display_scale);

    let mut area = Rect::new(x, y, 0, 0);
    let mut y_pos = area.y;

    let mut max_x = 0;
    let mut max_y = 0;

    let (r1, g1, b1) = split(color);

    'line: for line in text.lines() {
        let mut glyph_x = x as f32;

        for char in line.chars() {
            let (metrics, bitmap) = cache_map
                .entry((char, font_size))
                .or_insert_with(|| font.rasterize(char, font_size as f32));

            let glyph_y = y_pos as f32 - (metrics.height as f32 - metrics.advance_height) - metrics.ymin as f32;

            for y_px in 0..metrics.height {
                'x: for x_px in 0..metrics.width {
                    let screen_x_i32 = glyph_x.round() as i32 + metrics.xmin + x_px as i32;
                    if screen_x_i32 < 0 {
                        continue;
                    }

                    let screen_x = screen_x_i32 as usize;
                    if screen_x >= window_width {
                        continue;
                    }

                    let alpha = bitmap[x_px + y_px * metrics.width];
                    if alpha == 0 {
                        continue;
                    }

                    let offset = font_size as f32 + glyph_y + y_px as f32;

                    if offset < 0.0 {
                        continue;
                    }

                    if max_x < screen_x {
                        max_x = screen_x;
                    }

                    if max_y < offset as usize {
                        max_y = offset as usize;
                    }

                    if skip_draw {
                        continue;
                    }

                    let i = screen_x + window_width * offset as usize;

                    if i >= buffer.len() {
                        break 'x;
                    }

                    let (r2, g2, b2) = split(buffer[i]);

                    let r = blend(r1, alpha, r2, 255 - alpha);
                    let g = blend(g1, alpha, g2, 255 - alpha);
                    let b = blend(b1, alpha, b2, 255 - alpha);

                    if let Some(px) = buffer.get_mut(i) {
                        *px = rgb(r, g, b);
                    }
                }
            }

            glyph_x += metrics.advance_width;

            if glyph_x.round() as usize >= window_width {
                break 'line;
            }
        }

        y_pos += font_size;
    }

    area.height = max_y + 1 - area.y;
    area.width = max_x + 1 - area.x;
    area
}

pub fn draw_text_subpixel(
    text: &str,
    font: &fontdue::Font,
    x: usize,
    y: usize,
    font_size: usize,
    display_scale: f32,
    window_width: usize,
    buffer: &mut [u32],
    color: u32,
    skip_draw: bool,
    cache_map: &mut HashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>,
) -> Rect {
    pub fn apply_lcd_filter(bitmap: &[u8], width: usize, height: usize) -> Vec<u8> {
        let stride = width * 3;
        let mut output = vec![0u8; bitmap.len()];

        for row in 0..height {
            let offset = row * stride;
            for i in 0..stride {
                // We only filter horizontally across R, G, B values
                let idx = offset + i;

                // Boundary checks for left/right neighbors
                let left = if i == 0 { 0 } else { bitmap[idx - 1] as u16 };
                let center = bitmap[idx] as u16;
                let right = if i == stride - 1 { 0 } else { bitmap[idx + 1] as u16 };

                // [1, 2, 1] weighted average
                output[idx] = ((left + center * 2 + right) / 4) as u8;
            }
        }
        output
    }

    if text.is_empty() || font_size == 0 {
        return Rect::default();
    }

    let x_start = scale(x, display_scale);
    let y_start = scale(y, display_scale);
    let font_size = scale(font_size, display_scale);

    let mut area = Rect::new(x_start, y_start, 0, 0);
    let mut y_pos = area.y;

    let mut max_x = 0;
    let mut max_y = 0;

    let (r, g, b) = split(color);

    let txt_r_lin = (r as f32 / 255.0).powi(2);
    let txt_g_lin = (g as f32 / 255.0).powi(2);
    let txt_b_lin = (b as f32 / 255.0).powi(2);

    'line: for line in text.lines() {
        let mut glyph_x = x_start as f32;

        for char in line.chars() {
            let (metrics, bitmap) = cache_map.entry((char, font_size)).or_insert_with(|| {
                let (metrics, bitmap) = font.rasterize_subpixel(char, font_size as f32);
                let bitmap = apply_lcd_filter(&bitmap, metrics.width, metrics.height);
                (metrics, bitmap)
            });

            let glyph_y = y_pos as f32 - metrics.bounds.height - metrics.bounds.ymin;

            for y_px in 0..metrics.height {
                let offset = font_size as f32 + glyph_y + y_px as f32;

                if offset < 0.0 {
                    continue;
                }

                let screen_y = offset as usize;

                'x: for x_px in 0..metrics.width {
                    let screen_x_i32 = glyph_x.round() as i32 + metrics.xmin + x_px as i32;
                    if screen_x_i32 < 0 {
                        continue;
                    }

                    let screen_x = screen_x_i32 as usize;
                    if screen_x >= window_width {
                        continue;
                    }

                    let glyph_idx = (y_px * metrics.width + x_px) * 3;

                    let mask_r = bitmap[glyph_idx] as f32 / 255.0;
                    let mask_g = bitmap[glyph_idx + 1] as f32 / 255.0;
                    let mask_b = bitmap[glyph_idx + 2] as f32 / 255.0;

                    if mask_r == 0.0 && mask_g == 0.0 && mask_b == 0.0 {
                        continue;
                    }

                    if max_x < screen_x {
                        max_x = screen_x;
                    }

                    if max_y < screen_y {
                        max_y = screen_y;
                    }

                    if skip_draw {
                        continue;
                    }

                    let i = screen_x + window_width * screen_y;

                    if i >= buffer.len() {
                        break 'x;
                    }

                    if let Some(bg) = buffer.get_mut(i) {
                        let bg_r = (((*bg >> 16) & 0xFF) as f32 / 255.0).powi(2);
                        let bg_g = (((*bg >> 8) & 0xFF) as f32 / 255.0).powi(2);
                        let bg_b = ((*bg & 0xFF) as f32 / 255.0).powi(2);

                        let out_r_lin = (txt_r_lin * mask_r) + (bg_r * (1.0 - mask_r));
                        let out_g_lin = (txt_g_lin * mask_g) + (bg_g * (1.0 - mask_g));
                        let out_b_lin = (txt_b_lin * mask_b) + (bg_b * (1.0 - mask_b));

                        let r = (out_r_lin.sqrt() * 255.0) as u8;
                        let g = (out_g_lin.sqrt() * 255.0) as u8;
                        let b = (out_b_lin.sqrt() * 255.0) as u8;

                        *bg = rgb(r, g, b);
                    }
                }
            }

            glyph_x += metrics.advance_width;

            if glyph_x.round() as usize >= window_width {
                break 'line;
            }
        }

        y_pos += font_size;
    }

    area.height = max_y + 1 - area.y;
    area.width = max_x + 1 - area.x;

    area
}
