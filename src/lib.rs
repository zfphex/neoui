#![feature(portable_simd)]
pub mod style;
pub use style::*;

pub mod shapes;
pub use shapes::*;

pub mod render_cache;
pub use render_cache::*;

pub mod image;
pub use image::*;

pub mod pool;

pub use mini::*;
pub use minwin::*;

use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

pub use fontdue;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Flow {
    #[default]
    Down,
    Right,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Frame {
    pub bounds: Rect,
    pub clip: Rect,
    pub flow: Flow,
    pub cross_align: CrossAlign,
    pub depth: usize,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub max_child_width: i32,
    pub max_child_height: i32,
    pub scroll_y: usize,
    scope: usize,
    anim_slot: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Ease {
    Linear,
    OutCubic,
    InOutSine,
    OutBack,
}

pub struct AnimationStateF32 {
    pub current: f32,
    pub target: f32,
    pub initial: f32,
    pub elapsed: f32,
}

#[derive(Debug, std::hash::Hash)]
pub enum Command<'a> {
    Rect {
        bounds: Rect,
        clip: Rect,
        color: u32,
        radius: usize,
    },
    RectStroke {
        bounds: Rect,
        clip: Rect,
        color: u32,
        radius: usize,
        border_thickness: usize,
        border_sides: u8,
    },
    Triangle {
        a: (i32, i32),
        b: (i32, i32),
        c: (i32, i32),
        clip: Rect,
        color: u32,
    },
    Text {
        text: Cow<'a, str>,
        font_id: usize,
        clip: Rect,
        bounds: Rect,
        color: u32,
        size: usize,
    },
    Image {
        image: &'a Image,
        bounds: Rect,
        clip: Rect,
        fit: ImageFit,
        opacity: u8,
        radius: usize,
    },
}

impl<'a> Command<'a> {
    pub fn clip(&self) -> Rect {
        match self {
            Command::Rect { clip, .. }
            | Command::RectStroke { clip, .. }
            | Command::Triangle { clip, .. }
            | Command::Text { clip, .. }
            | Command::Image { clip, .. } => *clip,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Layout {
    pub size: Rect,
    pub paint_x: i32,
    pub paint_y: i32,
}

#[derive(Debug, Clone)]
pub struct State {
    pub pressed: bool,
    pub released: bool,
    pub clicked: bool,
    pub double_clicked: bool,
    pub hovered: bool,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct ScrollState {
    pub max_scroll: i32,
    pub content_height: i32,
    pub scrolled: bool,
    /// 1 up, -1 down.
    pub direction: i32,
}

pub const FONT: &[u8] = include_bytes!("../fonts/Aptos.ttf");

pub fn ui(title: &str, width: usize, height: usize) -> Context {
    let window = create_window(title, None, width as i32, height as i32, false, WindowStyle::Standard);

    Context {
        window,
        state: UiState {
            fonts: vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()],
            layout_stack: Vec::new(),
            font_bitmaps: FxHashMap::default(),
            font_metrics: FxHashMap::default(),
            image_cache: FxHashMap::default(),
            default_font_size: 32,
            clear_color: black(),
            scroll_y: 0,
            left_mouse_start: None,
            left_mouse_release: None,
            dt: 0.0,
            next_scope: 0,
            id_stack: Vec::new(),
            anim_state_f32: FxHashMap::default(),
            anim_state_color: FxHashMap::default(),
            last_frame_time: std::time::Instant::now(),
            animating: false,
            hovered_depth: None,
            render_cache: RenderCache::default(),
        },
    }
}

pub struct Context {
    pub window: std::pin::Pin<Box<Window>>,
    state: UiState,
}

impl Deref for Context {
    type Target = UiState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

pub struct UiState {
    pub fonts: Vec<fontdue::Font>,
    pub font_bitmaps: FxHashMap<usize, FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    pub font_metrics: FxHashMap<usize, FxHashMap<(char, usize), fontdue::Metrics>>,
    pub image_cache: FxHashMap<ImageKey, ImageEntry>,
    pub default_font_size: usize,
    pub clear_color: u32,
    pub scroll_y: i32,

    pub dt: f32,
    pub next_scope: usize,
    pub id_stack: Vec<(usize, usize)>,
    pub anim_state_f32: FxHashMap<(usize, usize), AnimationStateF32>,
    pub anim_state_color: FxHashMap<(usize, usize), (f32, f32, f32)>,
    pub last_frame_time: std::time::Instant,
    pub animating: bool,

    pub left_mouse_start: Option<Rect>,
    pub left_mouse_release: Option<Rect>,
    pub hovered_depth: Option<usize>,
    pub layout_stack: Vec<Frame>,
    pub render_cache: RenderCache,
}

impl UiState {
    /// Add a font then return a font ID to use.
    pub fn add_font(&mut self, font: fontdue::Font) -> usize {
        let id = self.fonts.len();
        self.fonts.push(font);
        id
    }
}

pub struct FrameContext<'frame, 'text> {
    pub window: &'frame mut Window,
    pub commands: [Vec<Command<'text>>; 16],
    state: &'frame mut UiState,
}

impl Deref for FrameContext<'_, '_> {
    type Target = UiState;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl DerefMut for FrameContext<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl Context {
    pub fn frame<'text, F>(&mut self, mut ui: F)
    where
        F: for<'frame> FnMut(&mut FrameContext<'frame, 'text>),
    {
        let state = &mut self.state;

        self.window.draw(|window| {
            let mut frame = FrameContext {
                window,
                state,
                commands: [const { Vec::new() }; 16],
            };
            let now = std::time::Instant::now();
            frame.dt = (now - frame.last_frame_time).as_secs_f32();
            frame.last_frame_time = now;
            frame.next_scope = 0;
            frame.id_stack.clear();
            frame.animating = false;
            frame.scroll_y = frame.window.scroll_delta().1.round() as i32;
            frame.hovered_depth = None;

            if frame.window.mouse_pressed(Mouse::Left) {
                frame.left_mouse_start = Some(frame.mouse_position());
                frame.left_mouse_release = None;
            }

            if frame.window.mouse_released(Mouse::Left) {
                frame.left_mouse_release = Some(frame.mouse_position());
            }

            let (width, height) = frame.window.content_size();
            let bounds = Rect::new(0, 0, width as i32, height as i32);

            frame.layout_stack.clear();
            let scope = frame.next_scope;
            frame.next_scope += 1;
            frame.layout_stack.push(Frame {
                bounds,
                clip: bounds,
                flow: Flow::Down,
                scope,
                ..Default::default()
            });

            ui(&mut frame);

            frame.draw_frame();

            if frame.window.mouse_released(Mouse::Left) {
                frame.left_mouse_start = None;
                frame.left_mouse_release = None;
            }
        });

        if !self.window.open() {
            return;
        }

        self.window.wait_for_vsync();

        if !self.state.animating {
            self.state.last_frame_time = std::time::Instant::now();
        }
    }
}

impl<'frame, 'text> FrameContext<'frame, 'text> {
    /// Force the current frame to rebuild the complete framebuffer.
    pub fn invalidate_render_cache(&mut self) {
        self.state.render_cache.invalidate();
    }

    /// Walk the layout forward by an explicit size and return the screen-space bounding box.
    pub fn walk_layout(&mut self, width: i32, height: i32, gap: i32) -> Layout {
        let frame = self.layout_stack.last_mut().expect("No active layout frame");
        let (x, y) = match frame.flow {
            Flow::Down => {
                let x = match frame.cross_align {
                    CrossAlign::Start => frame.cursor_x,
                    CrossAlign::Center => frame.bounds.x + (frame.bounds.width.saturating_sub(width)) / 2,
                    CrossAlign::End => frame.bounds.right().saturating_sub(width),
                };
                (x, frame.cursor_y)
            }
            Flow::Right => {
                let y = match frame.cross_align {
                    CrossAlign::Start => frame.cursor_y,
                    CrossAlign::Center => frame.bounds.y + (frame.bounds.height.saturating_sub(height)) / 2,
                    CrossAlign::End => frame.bounds.bottom().saturating_sub(height),
                };
                (frame.cursor_x, y)
            }
        };
        let rect = Rect::new(x, y, width, height);

        match frame.flow {
            Flow::Down => {
                frame.cursor_y += height + gap;
                frame.max_child_width = frame.max_child_width.max(width);
                frame.max_child_height += height + gap;
            }
            Flow::Right => {
                frame.cursor_x += width + gap;
                frame.max_child_width += width + gap;
                frame.max_child_height = frame.max_child_height.max(height);
            }
        }

        if frame.scroll_y == 0 {
            return Layout {
                size: rect,
                paint_x: rect.x,
                paint_y: rect.y,
            };
        }

        // Calculate the scroll position.

        //  0 --------+
        //            |
        //            | <-- `frame.scroll_y`
        //            |
        //  0 ========|========= <--  Screen top
        //            |        |
        //            |        | <-- `y` = rect.y - frame.scroll_y
        //            |        |     (Distance from screen top)
        //  prev_y -> +------+ |
        //            | ITEM | |
        //            +------+ |
        //                     |
        //            VIEWPORT |
        //  ====================
        let x = rect.x;
        let y = rect.y - frame.scroll_y as i32;
        let clip_top = frame.clip.y;
        let clip_bottom = frame.clip.bottom();

        // Check if the current item underflows the viewport.
        //
        //    +---------+
        //    | ITEM    |
        //    +---------+     <-- y + height
        //
        // ================== <-- frame.bounds.y
        // |                |
        // |    VIEWPORT    |
        // |                |
        // ==================

        if y + height <= clip_top {
            return Layout {
                paint_x: x,
                paint_y: y,
                ..Default::default()
            };
        }

        // Check if the current item overflows the viewport.
        //
        // ==================
        // |                |
        // |    VIEWPORT    |
        // |                |
        // ================== <-- frame.bounds.y + frame.bounds.height
        //
        //    +---------+     <-- y
        //    | ITEM    |
        //    +---------+

        if y >= clip_bottom {
            return Layout {
                paint_x: x,
                paint_y: y,
                ..Default::default()
            };
        }

        let visible_y = y.max(clip_top).max(0);
        let visible_bottom = (y + height).min(clip_bottom).max(0);

        Layout {
            size: Rect::new(rect.x, visible_y, width, visible_bottom.saturating_sub(visible_y)),
            paint_x: x,
            paint_y: y,
        }
    }

    pub fn resolve_rect(&self, rect: Rect, flow: Flow, size: Size) -> i32 {
        match size {
            Size::Pixel(px) => px,
            Size::Percentage(pct) => {
                let total = match flow {
                    Flow::Down => rect.height,
                    Flow::Right => rect.width,
                };
                (total as f32 * pct) as i32
            }
            Size::Fill => match flow {
                Flow::Down => rect.height,
                Flow::Right => rect.width,
            },
            Size::FillMinus(sub) => {
                let remaining = match flow {
                    Flow::Down => rect.height,
                    Flow::Right => rect.width,
                };
                remaining.saturating_sub(sub.abs())
            }
        }
    }

    pub fn split_rect_h(&self, rect: Rect, size: impl IntoSize) -> (Rect, Rect) {
        let left_width = self.resolve_rect(rect, Flow::Right, size.into_size().unwrap_or_default());
        let total_w = rect.width;
        let total_h = rect.height;
        let left_w = left_width.min(total_w);
        let right_w = total_w.saturating_sub(left_w);
        let left_rect = Rect::new(rect.x, rect.y, left_w, total_h);
        let right_rect = Rect::new(rect.x + left_w, rect.y, right_w, total_h);
        (left_rect, right_rect)
    }

    pub fn split_rect_v(&self, rect: Rect, size: impl IntoSize) -> (Rect, Rect) {
        let top_height = self.resolve_rect(rect, Flow::Down, size.into_size().unwrap_or_default());
        let total_w = rect.width;
        let total_h = rect.height;
        let top_h = top_height.min(total_h);
        let bottom_h = total_h.saturating_sub(top_h);
        let top_rect = Rect::new(rect.x, rect.y, total_w, top_h);
        let bottom_rect = Rect::new(rect.x, rect.y + top_h, total_w, bottom_h);
        (top_rect, bottom_rect)
    }

    /// Splits the current frame's remaining space horizontally.
    pub fn split_h(&self, left_width: impl IntoSize) -> (Rect, Rect) {
        let left_width = self.resolve_size(left_width.into_size().unwrap_or_default(), Flow::Right);
        let frame = self.layout_stack.last().expect("No active frame");

        let total_w = frame.bounds.right().saturating_sub(frame.cursor_x);
        let total_h = frame.bounds.bottom().saturating_sub(frame.cursor_y);

        let left_w = left_width.min(total_w);
        let right_w = total_w.saturating_sub(left_w);

        let left_rect = Rect::new(frame.cursor_x, frame.cursor_y, left_w, total_h);
        let right_rect = Rect::new(frame.cursor_x + left_w, frame.cursor_y, right_w, total_h);

        (left_rect, right_rect)
    }

    /// Splits the current frame's remaining space vertically.
    pub fn split_v(&self, top_height: impl IntoSize) -> (Rect, Rect) {
        let top_height = self.resolve_size(top_height.into_size().unwrap_or_default(), Flow::Down);
        let frame = self.layout_stack.last().expect("No active frame");

        let total_w = frame.bounds.right().saturating_sub(frame.cursor_x);
        let total_h = frame.bounds.bottom().saturating_sub(frame.cursor_y);

        let top_h = top_height.min(total_h);
        let bottom_h = total_h.saturating_sub(top_h);

        let top_rect = Rect::new(frame.cursor_x, frame.cursor_y, total_w, top_h);
        let bottom_rect = Rect::new(frame.cursor_x, frame.cursor_y + top_h, total_w, bottom_h);

        (top_rect, bottom_rect)
    }

    pub fn resolve_size(&self, size: Size, flow: Flow) -> i32 {
        let frame = self.layout_stack.last().expect("No active frame");
        match size {
            Size::Pixel(px) => px,
            Size::Percentage(pct) => {
                let total = match flow {
                    Flow::Down => frame.bounds.height,
                    Flow::Right => frame.bounds.width,
                };
                (total as f32 * pct) as i32
            }
            Size::Fill => match flow {
                Flow::Down => frame.bounds.bottom().saturating_sub(frame.cursor_y),
                Flow::Right => frame.bounds.right().saturating_sub(frame.cursor_x),
            },
            Size::FillMinus(sub) => {
                let remaining = match flow {
                    Flow::Down => frame.bounds.bottom().saturating_sub(frame.cursor_y),
                    Flow::Right => frame.bounds.right().saturating_sub(frame.cursor_x),
                };
                remaining.saturating_sub(sub.abs())
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn clicked(&self, rect: Rect) -> bool {
        self.window.mouse_clicked(Mouse::Left, rect)
    }

    //Trackpads are really awful if you use standard click behaviour.
    //The user could plug in a mouse so this is not great.
    //TODO: Programatically change based on user input device.
    #[cfg(target_os = "macos")]
    pub fn clicked(&self, rect: Rect) -> bool {
        self.pressed(rect)
    }

    pub fn double_clicked(&self, rect: Rect) -> bool {
        self.window.mouse_double_clicked(Mouse::Left, rect)
    }

    pub fn pressed(&self, rect: Rect) -> bool {
        self.window.mouse_pressed(Mouse::Left) && self.mouse_position().intersects(rect)
    }

    pub fn released(&self, rect: Rect) -> bool {
        self.window.mouse_released(Mouse::Left) && self.mouse_position().intersects(rect)
    }

    pub fn hovered(&self, rect: Rect) -> bool {
        self.mouse_position().intersects(rect)
    }

    pub fn hovered_depth(&mut self, rect: Rect, depth: usize) -> bool {
        if !self.hovered(rect) {
            return false;
        }

        if self.hovered_depth.is_some_and(|hovered_depth| depth < hovered_depth) {
            return false;
        }

        self.hovered_depth = Some(depth);
        true
    }

    pub fn dragged(&self, rect: Rect) -> bool {
        let Some(initial) = self.left_mouse_start else {
            return false;
        };

        self.window.mouse_down(Mouse::Left) && initial.intersects(rect)
    }

    /// Return what percentage of the rectangle has been dragged on the x-axis.
    pub fn drag_percentage_x(&self, rect: Rect) -> Option<f32> {
        if !self.dragged(rect) {
            return None;
        }

        if rect.width == 0 {
            return Some(0.0);
        }

        let x = self.mouse_position().x.saturating_sub(rect.x);
        Some((x as f32 / rect.width as f32).clamp(0.0, 1.0))
    }

    /// Return what percentage of the rectangle has been dragged on the y-axis.
    pub fn drag_percentage_y(&self, rect: Rect) -> Option<f32> {
        if !self.dragged(rect) {
            return None;
        }

        if rect.height == 0 {
            return Some(0.0);
        }

        let y = self.mouse_position().y.saturating_sub(rect.y);
        Some((y as f32 / rect.height as f32).clamp(0.0, 1.0))
    }

    /// Check if a rectangle is clicked off of
    pub fn lost_focus(&self, rect: Rect) -> bool {
        let (Some(initial), Some(release)) = (self.left_mouse_start, self.left_mouse_release) else {
            return false;
        };

        !initial.intersects(rect) && !release.intersects(rect)
    }

    pub fn mouse_position(&self) -> Rect {
        let Some((x, y)) = self.window.mouse_pos() else {
            return Rect::default();
        };

        Rect::new(x.max(0.0) as i32, y.max(0.0) as i32, 1, 1)
    }

    pub fn paint_rect(&mut self, rect: Rect, style: Style) {
        let frame = self.current_frame();
        let clip = frame.clip;
        let depth = style.depth.unwrap_or(frame.depth);

        if let Some(color) = style.bg {
            self.commands[depth].push(Command::Rect {
                bounds: rect,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
            });
        }

        if let Some(color) = style.border {
            self.commands[depth].push(Command::RectStroke {
                bounds: rect,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
                border_thickness: style.border_thickness.unwrap_or(1),
                border_sides: style.border_side.unwrap_or(border::ALL),
            });
        }
    }

    pub fn paint_triangle(&mut self, a: (i32, i32), b: (i32, i32), c: (i32, i32), style: Style) {
        let frame = self.current_frame();
        let clip = frame.clip;
        let depth = style.depth.unwrap_or(frame.depth);
        if let Some(color) = style.bg {
            self.commands[depth].push(Command::Triangle { a, b, c, clip, color });
        }
    }

    #[cfg(feature = "image")]
    pub fn paint_image(&mut self, bounds: Rect, image: &'text Image, style: Style) {
        if bounds.is_empty() {
            return;
        }
        let frame = self.current_frame();
        let depth = style.depth.unwrap_or(frame.depth);
        let clip = frame.clip;
        let opacity = style.opacity.unwrap_or(255);
        let fit = style.fit.unwrap_or_default();
        let radius = style.radius.unwrap_or(0);
        self.commands[depth].push(Command::Image {
            image,
            bounds,
            clip,
            fit,
            opacity,
            radius,
        });
    }

    pub fn paint_text_measured(
        &mut self,
        text: impl Into<Cow<'text, str>>,
        metrics: Rect,
        rect: Rect,
        color: u32,
        font_id: usize,
        font_size: usize,
        alignment: Alignment,
        padding: Padding,
        depth: usize,
    ) {
        let text = text.into();
        if text.is_empty() {
            return;
        }

        let Some((x, y)) = align_rect(rect, metrics.width, metrics.height, alignment, padding) else {
            return;
        };

        // Clip glyphs
        let content = Rect::new(
            rect.x + padding.left as i32,
            rect.y + padding.top as i32,
            (rect.width - padding.left as i32 - padding.right as i32).max(0),
            (rect.height - padding.top as i32 - padding.bottom as i32).max(0),
        );
        let frame_clip = self.layout_stack.last().expect("No active frame").clip;
        let clip = frame_clip.intersection(content);
        if clip.is_empty() {
            return;
        }

        self.commands[depth].push(Command::Text {
            text,
            clip,
            bounds: Rect::new(x, y, metrics.width, metrics.height),
            color,
            font_id,
            size: font_size,
        });
    }

    pub fn paint_text(
        &mut self,
        text: impl Into<Cow<'text, str>>,
        rect: Rect,
        color: u32,
        font_id: usize,
        font_size: usize,
        alignment: Alignment,
        padding: Padding,
        depth: usize,
    ) {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        let metrics = self.measure_text(&text, font_id, font_size);
        self.paint_text_measured(
            text, metrics, rect, color, font_id, font_size, alignment, padding, depth,
        );
    }

    pub fn measure_text(&mut self, text: &str, font_id: usize, font_size: usize) -> Rect {
        let state = &mut *self.state;
        let font = &state.fonts[font_id];
        let metrics = state.font_metrics.entry(font_id).or_default();
        measure_text(text, font, font_size, metrics)
    }

    pub fn gap(&mut self, gap: impl IntoSize) {
        if let Some(gap) = gap.into_size() {
            let frame = self.layout_stack.last().expect("No active frame");
            let gap = self.resolve_size(gap, frame.flow);
            self.walk_layout(0, 0, gap);
        }
    }

    pub fn rect(&mut self, style: Style) -> State {
        self.item("", style)
    }

    pub fn text(&mut self, text: impl Into<Cow<'text, str>>, style: Style) -> State {
        self.item(text, style)
    }

    pub fn line(&mut self, parts: impl IntoIterator<Item = impl Into<Line<'text>>>, style: Style) -> State {
        let parts: Vec<Line<'text>> = parts.into_iter().map(Into::into).collect();
        let default_size = self.default_font_size;
        let selected = style.is_selected;

        let mut content_w = 0i32;
        let mut content_h = 0i32;
        let mut run_metrics: Vec<(Rect, Padding)> = Vec::with_capacity(parts.len());
        for part in &parts {
            let font_size = part.style.font_size.unwrap_or(default_size);
            let metrics = if part.content.is_empty() {
                Rect::default()
            } else {
                self.measure_text(&part.content, part.style.font, font_size)
            };
            let run_pad = part.style.padding.unwrap_or_default();
            let w = metrics.width + run_pad.left as i32 + run_pad.right as i32;
            let h = metrics.height + run_pad.top as i32 + run_pad.bottom as i32;
            content_w += w;
            content_h = content_h.max(h);
            run_metrics.push((metrics, run_pad));
        }

        let padding = style.padding.unwrap_or_default();
        let width = style
            .width
            .map(|w| self.resolve_size(w, Flow::Right))
            .unwrap_or(content_w + padding.left as i32 + padding.right as i32);
        let height = style
            .height
            .map(|h| self.resolve_size(h, Flow::Down))
            .unwrap_or(content_h + padding.top as i32 + padding.bottom as i32);

        let (paint_x, paint_y, rect) = self.resolve_item_layout(width, height, &style);

        if rect.is_empty() {
            return State {
                clicked: false,
                double_clicked: false,
                hovered: false,
                pressed: false,
                released: false,
                rect,
            };
        }

        let frame = self.current_frame();
        let depth = style.depth.unwrap_or(frame.depth);
        let clip = frame.clip;
        let hovered = self.hovered_depth(rect, depth);
        let clicked = hovered && self.clicked(rect);
        let double_clicked = hovered && self.double_clicked(rect);
        let pressed = hovered && self.pressed(rect);
        let released = hovered && self.released(rect);
        let paint_bounds = Rect::new(paint_x, paint_y, width, height);

        let bg = if selected && style.selected.is_some() {
            style.selected
        } else if hovered && style.hover.is_some() {
            style.hover
        } else {
            style.bg
        };

        if let Some(color) = bg {
            self.commands[depth].push(Command::Rect {
                bounds: paint_bounds,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
            });
        }

        let border = if selected && style.selected_border.is_some() {
            style.selected_border
        } else {
            style.border
        };

        if let Some(border) = border {
            self.commands[depth].push(Command::RectStroke {
                bounds: paint_bounds,
                clip,
                color: border,
                radius: style.radius.unwrap_or(0),
                border_thickness: style.border_thickness.unwrap_or(1),
                border_sides: style.border_side.unwrap_or(border::ALL),
            });
        }

        let inner_x = paint_x + padding.left as i32;
        let inner_y = paint_y + padding.top as i32;
        let inner_w = (width - padding.left as i32 - padding.right as i32).max(0);
        let inner_h = (height - padding.top as i32 - padding.bottom as i32).max(0);

        let group_x = match style.alignment.unwrap_or(Alignment::Left) {
            Alignment::Left | Alignment::TopLeft | Alignment::BottomLeft => inner_x,
            Alignment::Center | Alignment::TopCenter | Alignment::BottomCenter => {
                inner_x + (inner_w.saturating_sub(content_w)) / 2
            }
            Alignment::Right | Alignment::TopRight | Alignment::BottomRight => {
                inner_x + inner_w.saturating_sub(content_w)
            }
        };

        let mut cursor_x = group_x;
        for (part, (metrics, run_pad)) in parts.into_iter().zip(run_metrics) {
            if part.content.is_empty() {
                cursor_x += run_pad.left as i32 + run_pad.right as i32;
                continue;
            }
            let font_size = part.style.font_size.unwrap_or(default_size);
            let run_w = metrics.width + run_pad.left as i32 + run_pad.right as i32;

            self.paint_text_measured(
                part.content,
                metrics,
                Rect::new(cursor_x, inner_y, run_w, inner_h),
                part.style.fg.unwrap_or(style.fg.unwrap_or(white())),
                part.style.font,
                font_size,
                Alignment::Left,
                run_pad,
                part.style.depth.unwrap_or(depth),
            );

            cursor_x += run_w;
        }

        State {
            clicked,
            double_clicked,
            hovered,
            rect,
            pressed,
            released,
        }
    }

    pub fn item(&mut self, text: impl Into<Cow<'text, str>>, style: Style) -> State {
        let text = text.into();
        let selected = style.is_selected;
        let font_size = style.font_size.unwrap_or(self.default_font_size);
        let metrics = if text.is_empty() {
            Rect::default()
        } else {
            self.measure_text(&text, style.font, font_size)
        };

        let padding = style.padding.unwrap_or_default();
        let width = style
            .width
            .map(|w| self.resolve_size(w, Flow::Right))
            .unwrap_or(metrics.width + padding.left as i32 + padding.right as i32);
        let height = style
            .height
            .map(|h| self.resolve_size(h, Flow::Down))
            .unwrap_or(metrics.height + padding.top as i32 + padding.bottom as i32);

        let (paint_x, paint_y, rect) = self.resolve_item_layout(width, height, &style);

        if rect.is_empty() {
            return State {
                clicked: false,
                double_clicked: false,
                hovered: false,
                pressed: false,
                released: false,
                rect,
            };
        }

        let frame = self.current_frame();
        let depth = style.depth.unwrap_or(frame.depth);
        let clip = frame.clip;
        let hovered = self.hovered_depth(rect, depth);
        // Input follows the same depth ordering as hover.
        let clicked = hovered && self.clicked(rect);
        let double_clicked = hovered && self.double_clicked(rect);
        let pressed = hovered && self.pressed(rect);
        let released = hovered && self.released(rect);
        let paint_bounds = Rect::new(paint_x, paint_y, width, height);

        let bg = if selected && style.selected.is_some() {
            style.selected
        } else if hovered && style.hover.is_some() {
            style.hover
        } else {
            style.bg
        };

        if let Some(color) = bg {
            self.commands[depth].push(Command::Rect {
                bounds: paint_bounds,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
            });
        }

        let border = if selected && style.selected_border.is_some() {
            style.selected_border
        } else {
            style.border
        };

        // TODO: Borders render inside of the bounding box
        // for text which means they can overlap...
        if let Some(border) = border {
            self.commands[depth].push(Command::RectStroke {
                bounds: paint_bounds,
                clip,
                color: border,
                radius: style.radius.unwrap_or(0),
                border_thickness: style.border_thickness.unwrap_or(1),
                border_sides: style.border_side.unwrap_or(border::ALL),
            });
        }

        if !text.is_empty() {
            self.paint_text_measured(
                text,
                metrics,
                Rect::new(paint_x, paint_y, width, height),
                style.fg.unwrap_or(white()),
                style.font,
                font_size,
                style.alignment.unwrap_or(Alignment::Center),
                style.padding.unwrap_or_default(),
                depth,
            );
        }

        State {
            clicked,
            double_clicked,
            hovered,
            rect,
            pressed,
            released,
        }
    }

    pub fn flow<R>(
        &mut self,
        style: impl Into<Style>,
        flow: Flow,
        advance: bool,
        scroll_y: usize,
        ui: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let parent_scroll = self.layout_stack.last().map(|f| f.scroll_y).unwrap_or(0);

        let style = style.into();
        let mut bounds = self.current_frame_bounds();

        if let Some(width) = style.width {
            bounds.width = self.resolve_size(width, Flow::Right);
        }

        if let Some(height) = style.height {
            bounds.height = self.resolve_size(height, Flow::Down);
        }

        if let Some(x) = style.x {
            bounds.x = self.resolve_size(x, Flow::Right);
        }

        if let Some(y) = style.y {
            bounds.y = self.resolve_size(y, Flow::Down);
        } else if parent_scroll != 0 {
            bounds.y -= parent_scroll as i32;
        }

        let frame = self.current_frame();
        let depth = style.depth.unwrap_or(frame.depth);
        let clip = frame.clip;
        let padding = style.padding.unwrap_or_default();
        let cross_align = style.cross_align.unwrap_or_default();

        bounds.x += padding.left as i32;
        bounds.width = bounds.width.saturating_sub((padding.left + padding.right) as i32);
        bounds.y += padding.top as i32;
        bounds.height = bounds.height.saturating_sub((padding.top + padding.bottom) as i32);

        // Draw the background first.
        if let Some(color) = style.bg {
            self.commands[depth].push(Command::Rect {
                bounds,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
            });
        }

        let new_frame = Frame {
            bounds,
            clip: clip.intersection(bounds),
            flow,
            cross_align,
            depth,
            cursor_x: bounds.x,
            cursor_y: bounds.y,
            // Nested flows are already placed in screen space; do not re-apply parent scroll
            // on their children. Only this frame's own scroll_y (e.g. scroll_view) applies.
            scroll_y,
            scope: {
                let s = self.next_scope;
                self.next_scope += 1;
                s
            },
            ..Default::default()
        };

        self.layout_stack.push(new_frame);
        let result = ui(self);

        // Draw the border over the content, idk.
        if let Some(color) = style.border {
            self.commands[depth].push(Command::RectStroke {
                bounds,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
                border_thickness: style.border_thickness.unwrap_or(1),
                border_sides: style.border_side.unwrap_or(border::ALL),
            });
        }

        if advance {
            self.end_layout();
        }

        result
    }

    pub fn flow_down<R>(&mut self, style: impl Into<Style>, ui: impl FnOnce(&mut Self) -> R) -> R {
        self.flow(style, Flow::Down, true, 0, ui)
    }

    pub fn flow_right<R>(&mut self, style: impl Into<Style>, ui: impl FnOnce(&mut Self) -> R) -> R {
        self.flow(style, Flow::Right, true, 0, ui)
    }

    pub fn place_down<R>(&mut self, style: impl Into<Style>, ui: impl FnOnce(&mut Self) -> R) -> R {
        let r = self.flow(style, Flow::Down, false, 0, ui);
        self.layout_stack.pop().expect("Layout underflow");
        r
    }

    pub fn place_right<R>(&mut self, style: impl Into<Style>, ui: impl FnOnce(&mut Self) -> R) -> R {
        let r = self.flow(style, Flow::Right, false, 0, ui);
        self.layout_stack.pop().expect("Layout underflow");
        r
    }

    pub fn scroll<R>(
        &mut self,
        style: impl Into<Style>,
        scroll_y: &mut usize,
        ui: impl FnOnce(&mut Self) -> R,
    ) -> ScrollState {
        self.flow_scroll(style, scroll_y, ui, false)
    }

    pub fn flow_scroll<R>(
        &mut self,
        style: impl Into<Style>,
        scroll_y: &mut usize,
        ui: impl FnOnce(&mut Self) -> R,
        advance: bool,
    ) -> ScrollState {
        let flow = Flow::Down;
        let style = style.into();
        let _ = self.flow(style, flow, advance, *scroll_y, ui);

        let frame = self.layout_stack.pop().expect("Layout underflow");
        if let Some(parent) = self.layout_stack.last_mut() {
            match parent.flow {
                Flow::Down => {
                    parent.cursor_y += frame.bounds.height;
                    parent.max_child_width = parent.max_child_width.max(frame.bounds.width);
                    parent.max_child_height += frame.bounds.height;
                }
                Flow::Right => {
                    parent.cursor_x += frame.bounds.width;
                    parent.max_child_width += frame.bounds.width;
                    parent.max_child_height = parent.max_child_height.max(frame.bounds.height);
                }
            }
        }

        let bounds = frame.bounds;
        let content_height = frame.max_child_height;

        let max_scroll = content_height.saturating_sub(bounds.height).max(0);
        let mut state = ScrollState {
            max_scroll,
            content_height,
            scrolled: false,
            direction: self.scroll_y,
        };

        if self.scroll_y != 0 && self.mouse_position().intersects(bounds) {
            #[cfg(target_os = "windows")]
            const WHEEL_STEP: usize = 50;

            #[cfg(target_os = "macos")]
            const WHEEL_STEP: usize = 1;

            if self.scroll_y > 0 {
                *scroll_y = (*scroll_y).saturating_sub(self.scroll_y.unsigned_abs() as usize * WHEEL_STEP);
            } else {
                *scroll_y = (*scroll_y).saturating_add(self.scroll_y.unsigned_abs() as usize * WHEEL_STEP);
            }
            state.scrolled = true;
        }

        *scroll_y = (*scroll_y).clamp(0, max_scroll as usize);

        state
    }

    pub fn current_frame(&self) -> &Frame {
        self.layout_stack.last().as_ref().unwrap()
    }

    fn resolve_item_layout(&mut self, width: i32, height: i32, style: &Style) -> (i32, i32, Rect) {
        let flow = self.layout_stack.last().expect("No active frame").flow;
        let gap = style.gap.map(|gap| self.resolve_size(gap, flow)).unwrap_or_default();
        let layout = self.walk_layout(width, height, gap);
        let paint_x = style.x.map_or(layout.paint_x, |x| self.resolve_size(x, Flow::Right));
        let paint_y = style.y.map_or(layout.paint_y, |y| self.resolve_size(y, Flow::Down));
        let rect = if style.x.is_some() || style.y.is_some() {
            Rect::new(paint_x, paint_y, width, height)
        } else {
            layout.size
        };
        (paint_x, paint_y, rect)
    }

    pub fn current_frame_bounds(&self) -> Rect {
        let parent = self.layout_stack.last().expect("Layout stack empty");
        Rect::new(
            parent.cursor_x,
            parent.cursor_y,
            parent.bounds.right().saturating_sub(parent.cursor_x),
            parent.bounds.bottom().saturating_sub(parent.cursor_y),
        )
    }

    pub fn begin_layout(&mut self, flow: Flow, bounds: Option<Rect>) {
        let bounds = if let Some(bounds) = bounds {
            bounds
        } else {
            self.current_frame_bounds()
        };

        let parent = self.layout_stack.last().expect("Layout stack empty");
        let new_frame = Frame {
            bounds,
            clip: parent.clip.intersection(bounds),
            flow,
            depth: parent.depth,
            cursor_x: bounds.x,
            cursor_y: bounds.y,
            scope: {
                let s = self.next_scope;
                self.next_scope += 1;
                s
            },
            ..Default::default()
        };

        self.layout_stack.push(new_frame);
    }

    pub fn end_layout(&mut self) {
        let finished = self.layout_stack.pop().expect("Layout underflow");
        if let Some(parent) = self.layout_stack.last_mut() {
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

    fn draw_frame(&mut self) {
        let window = &mut *self.window;
        let state = &mut *self.state;
        let display_scale = window.scale_factor() as f32;
        let (framebuffer_width, framebuffer_height) = window.framebuffer_size();

        let dirty = state.render_cache.update(
            &self.commands,
            display_scale,
            framebuffer_width,
            framebuffer_height,
            state.clear_color,
        );

        if dirty {
            raster_damage(
                &self.commands,
                &state.render_cache,
                window.framebuffer(),
                framebuffer_width,
                framebuffer_height,
                display_scale,
                state.clear_color,
                &state.fonts,
                &mut state.font_bitmaps,
                &mut state.image_cache,
            );
            window.present();
        }

        state.render_cache.finish();
    }

    pub fn with_id<R>(&mut self, id: impl Hash, ui: impl FnOnce(&mut Self) -> R) -> R {
        let parent = self
            .id_stack
            .last()
            .map(|(s, _)| *s)
            .unwrap_or(self.layout_stack.last().unwrap().scope);
        let mut h = rustc_hash::FxHasher::default();
        parent.hash(&mut h);
        id.hash(&mut h);
        self.id_stack.push((h.finish() as usize, 0));
        let r = ui(self);
        self.id_stack.pop();
        r
    }

    fn anim_id(&mut self) -> (usize, usize) {
        if let Some((s, n)) = self.id_stack.last_mut() {
            let id = (*s, *n);
            *n += 1;
            id
        } else {
            let f = self.layout_stack.last_mut().unwrap();
            let id = (f.scope, f.anim_slot);
            f.anim_slot += 1;
            id
        }
    }

    pub fn animate_f32(&mut self, target: f32, duration: f32, ease: Ease) -> f32 {
        let id = self.anim_id();
        let dt = self.dt;
        let state = self.anim_state_f32.entry(id).or_insert(AnimationStateF32 {
            current: target,
            target,
            initial: target,
            elapsed: duration,
        });

        if state.target != target {
            state.initial = state.current;
            state.target = target;
            state.elapsed = 0.0;
        }

        let still_animating;
        if state.elapsed < duration {
            state.elapsed += dt;

            let mut t = if duration > 0.0 { state.elapsed / duration } else { 1.0 };
            if t > 1.0 {
                t = 1.0;
            }

            let eased_t = match ease {
                Ease::Linear => t,
                Ease::OutCubic => 1.0 - (1.0 - t).powi(3),
                Ease::InOutSine => -(std::f32::consts::PI * t).cos() / 2.0 + 0.5,
                Ease::OutBack => {
                    let c1 = 1.70158;
                    let c3 = c1 + 1.0;
                    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
                }
            };

            state.current = state.initial + (state.target - state.initial) * eased_t;
            still_animating = true;
        } else {
            state.current = state.target;
            still_animating = false;
        }

        let current = state.current;
        if still_animating {
            self.animating = true;
        }
        current
    }

    pub fn animate_color(&mut self, target: u32, speed: f32) -> u32 {
        let id = self.anim_id();
        let dt = self.dt;
        let (tr, tg, tb) = split_f32(target);
        let (r, g, b) = self.anim_state_color.entry(id).or_insert((tr, tg, tb));
        let blend = 1.0 - (-speed * dt).exp();

        *r += (tr - *r) * blend;
        *g += (tg - *g) * blend;
        *b += (tb - *b) * blend;

        let result = rgb(*r as u8, *g as u8, *b as u8);
        if (*r - tr).abs() > 0.5 || (*g - tg).abs() > 0.5 || (*b - tb).abs() > 0.5 {
            self.animating = true;
        }
        result
    }
}
