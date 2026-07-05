pub mod style;
pub use style::*;

pub mod shapes;
pub use shapes::*;

pub use mini::*;
pub use miniwin::*;

use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

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
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub max_child_width: usize,
    pub max_child_height: usize,
    pub scroll_y: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Ease {
    Linear,
    OutCubic,
    InOutSine,
    OutBack,
}

pub fn apply_ease(t: f32, ease: Ease) -> f32 {
    match ease {
        Ease::Linear => t,
        Ease::OutCubic => 1.0 - (1.0 - t).powi(3),
        Ease::InOutSine => -(std::f32::consts::PI * t).cos() / 2.0 + 0.5,
        Ease::OutBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
    }
}

pub struct AnimationStateF32 {
    pub current: f32,
    pub target: f32,
    pub initial: f32,
    pub elapsed: f32,
}

#[derive(Debug)]
pub enum Command<'a> {
    Rect {
        x: i32,
        y: i32,
        width: usize,
        height: usize,
        clip: Rect,
        color: u32,
        radius: usize,
    },
    RectOutline {
        x: i32,
        y: i32,
        width: usize,
        height: usize,
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
        x: i32,
        y: i32,
        color: u32,
        size: usize,
    },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Layout {
    pub size: Rect,
    pub paint_x: i32,
    pub paint_y: i32,
}

#[derive(Debug, Clone)]
pub struct State {
    pub clicked: bool,
    pub hovered: bool,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct ScrollState {
    pub max_scroll: usize,
    pub content_height: usize,
    pub scrolled: bool,
    /// 1 up, -1 down.
    pub direction: i32,
}

pub const FONT: &[u8] = include_bytes!("../fonts/Aptos.ttf");

pub fn ui<'a>(title: &str, width: usize, height: usize) -> Context<'a> {
    let window = miniwin::create_window(title, None, width as i32, height as i32, false, WindowStyle::Standard);

    Context {
        window,
        state: UiState {
            commands: [const { Vec::new() }; 16],
            fonts: vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()],
            layout_stack: Vec::new(),
            font_bitmaps: FxHashMap::default(),
            font_metrics: FxHashMap::default(),
            default_font_size: 32,
            clear_color: black(),
            scroll_y: 0,
            left_mouse_start: None,
            left_mouse_release: None,
            dt: 0.0,
            anim_counter: 0,
            anim_state_f32: FxHashMap::default(),
            anim_state_color: FxHashMap::default(),
            last_frame_time: std::time::Instant::now(),
            animating: false,
        },
    }
}

pub struct Context<'a> {
    pub window: std::pin::Pin<Box<Window>>,
    state: UiState<'a>,
}

pub struct UiState<'a> {
    pub commands: [Vec<Command<'a>>; 16],
    pub fonts: Vec<fontdue::Font>,
    pub font_bitmaps: FxHashMap<usize, FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    pub font_metrics: FxHashMap<usize, FxHashMap<(char, usize), fontdue::Metrics>>,
    pub default_font_size: usize,
    pub clear_color: u32,
    pub scroll_y: i32,

    //Animation
    pub dt: f32,
    pub anim_counter: usize,
    pub anim_state_f32: FxHashMap<usize, AnimationStateF32>,
    pub anim_state_color: FxHashMap<usize, (f32, f32, f32)>,
    pub last_frame_time: std::time::Instant,
    pub animating: bool,

    left_mouse_start: Option<Rect>,
    left_mouse_release: Option<Rect>,
    layout_stack: Vec<Frame>,
}

pub struct FrameContext<'frame, 'a> {
    pub window: &'frame mut Window,
    state: &'frame mut UiState<'a>,
}

impl<'a> Deref for Context<'a> {
    type Target = UiState<'a>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'a> DerefMut for Context<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<'frame, 'a> Deref for FrameContext<'frame, 'a> {
    type Target = UiState<'a>;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<'frame, 'a> DerefMut for FrameContext<'frame, 'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl<'a> Context<'a> {
    pub fn frame<F>(&mut self, mut ui: F)
    where
        F: for<'frame> FnMut(&mut FrameContext<'frame, 'a>),
    {
        let state = &mut self.state;

        self.window.draw(|window| {
            let mut frame = FrameContext { window, state };
            let now = std::time::Instant::now();
            frame.dt = (now - frame.last_frame_time).as_secs_f32();
            frame.last_frame_time = now;
            frame.anim_counter = 0;
            frame.animating = false;
            frame.scroll_y = frame.window.scroll_delta().1.round() as i32;

            if frame.window.mouse_pressed(Mouse::Left) {
                frame.left_mouse_start = Some(frame.mouse_position());
                frame.left_mouse_release = None;
            }

            if frame.window.mouse_released(Mouse::Left) {
                frame.left_mouse_release = Some(frame.mouse_position());
            }

            let (width, height) = frame.window.content_size();
            let bounds = Rect::new(0, 0, width, height);

            let clear_color = frame.clear_color;
            frame.window.framebuffer().fill(clear_color);
            frame.layout_stack.clear();
            frame.layout_stack.push(Frame {
                bounds,
                clip: bounds,
                flow: Flow::Down,
                ..Default::default()
            });

            ui(&mut frame);

            frame.draw_frame();
            frame.window.present();

            if frame.window.mouse_released(Mouse::Left) {
                frame.left_mouse_start = None;
                frame.left_mouse_release = None;
            }
        });

        if self.state.animating {
            self.window.wait_for_vsync();
        } else {
            self.window.wait_for_event();
            self.state.last_frame_time = std::time::Instant::now();
        }
    }
}

impl<'a> UiState<'a> {
    /// Add a font then return a font ID to use.
    pub fn add_font(&mut self, font: fontdue::Font) -> usize {
        let id = self.fonts.len();
        self.fonts.push(font);
        id
    }
}

impl<'frame, 'a> FrameContext<'frame, 'a> {
    /// Walk the layout forward by an explicit size and return the screen-space bounding box.
    pub fn walk_layout(&mut self, width: usize, height: usize, gap: usize) -> Layout {
        let frame = self.layout_stack.last_mut().expect("No active layout frame");
        let rect = Rect::new(frame.cursor_x, frame.cursor_y, width, height);

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
                paint_x: rect.x as i32,
                paint_y: rect.y as i32,
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
        let x = rect.x as i32;
        let y = rect.y as i32 - frame.scroll_y as i32;
        let clip_top = frame.clip.y as i32;
        let clip_bottom = (frame.clip.y + frame.clip.height) as i32;

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

        if y + height as i32 <= clip_top {
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

        let visible_y = y.max(clip_top).max(0) as usize;
        let visible_bottom = (y + height as i32).min(clip_bottom).max(0) as usize;

        Layout {
            size: Rect::new(rect.x, visible_y, width, visible_bottom.saturating_sub(visible_y)),
            paint_x: x,
            paint_y: y,
        }
    }

    /// Add a font then return a font ID to use.
    pub fn add_font(&mut self, font: fontdue::Font) -> usize {
        let id = self.fonts.len();
        self.fonts.push(font);
        id
    }

    pub fn resolve_rect(&self, rect: Rect, flow: Flow, size: Size) -> usize {
        match size {
            Size::Pixel(px) => px,
            Size::Percentage(pct) => {
                let total = match flow {
                    Flow::Down => rect.height,
                    Flow::Right => rect.width,
                };
                (total as f32 * pct) as usize
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
                remaining.saturating_sub(sub.abs() as usize)
            }
        }
    }

    pub fn split_rect_h(&self, rect: Rect, size: impl IntoSize) -> (Rect, Rect) {
        let left_width = self.resolve_rect(rect, Flow::Right, size.into_size().unwrap_or_default());
        let total_w = (rect.x + rect.width).saturating_sub(rect.x);
        let total_h = (rect.y + rect.height).saturating_sub(rect.y);
        let left_w = left_width.min(total_w);
        let right_w = total_w.saturating_sub(left_w);
        let left_rect = Rect::new(rect.x, rect.y, left_w, total_h);
        let right_rect = Rect::new(rect.x + left_w, rect.y, right_w, total_h);
        (left_rect, right_rect)
    }

    pub fn split_rect_v(&self, rect: Rect, size: impl IntoSize) -> (Rect, Rect) {
        let top_height = self.resolve_rect(rect, Flow::Down, size.into_size().unwrap_or_default());
        let total_w = (rect.x + rect.width).saturating_sub(rect.x);
        let total_h = (rect.y + rect.height).saturating_sub(rect.y);
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

        let total_w = (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x);
        let total_h = (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y);

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

        let total_w = (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x);
        let total_h = (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y);

        let top_h = top_height.min(total_h);
        let bottom_h = total_h.saturating_sub(top_h);

        let top_rect = Rect::new(frame.cursor_x, frame.cursor_y, total_w, top_h);
        let bottom_rect = Rect::new(frame.cursor_x, frame.cursor_y + top_h, total_w, bottom_h);

        (top_rect, bottom_rect)
    }

    pub fn resolve_size(&self, size: Size, flow: Flow) -> usize {
        let frame = self.layout_stack.last().expect("No active frame");
        match size {
            Size::Pixel(px) => px,
            Size::Percentage(pct) => {
                let total = match flow {
                    Flow::Down => frame.bounds.height,
                    Flow::Right => frame.bounds.width,
                };
                (total as f32 * pct) as usize
            }
            Size::Fill => match flow {
                Flow::Down => (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y),
                Flow::Right => (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x),
            },
            Size::FillMinus(sub) => {
                let remaining = match flow {
                    Flow::Down => (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y),
                    Flow::Right => (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x),
                };
                remaining.saturating_sub(sub.abs() as usize)
            }
        }
    }

    pub fn clicked(&self, rect: Rect) -> bool {
        let frame = self.layout_stack.last().expect("No active frame");
        self.window.mouse_clicked(Mouse::Left, rect) && self.mouse_position().intersects(frame.bounds)
    }

    pub fn pressed(&self, rect: Rect) -> bool {
        let frame = self.layout_stack.last().expect("No active frame");
        self.window.mouse_down(Mouse::Left)
            && self.mouse_position().intersects(rect)
            && self.mouse_position().intersects(frame.bounds)
    }

    pub fn hovered(&self, rect: Rect) -> bool {
        let frame = self.layout_stack.last().expect("No active frame");
        self.mouse_position().intersects(rect) && self.mouse_position().intersects(frame.bounds)
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
        let Some(initial) = self.left_mouse_start else {
            return false;
        };

        let Some(release) = self.left_mouse_release else {
            return false;
        };

        self.window.mouse_released(Mouse::Left) && !initial.intersects(rect) && !release.intersects(rect)
    }

    pub fn mouse_position(&self) -> Rect {
        let Some((x, y)) = self.window.mouse_pos() else {
            return Rect::default();
        };

        Rect::new(x.max(0.0) as usize, y.max(0.0) as usize, 1, 1)
    }

    pub fn paint_rect(&mut self, rect: Rect, style: Style) {
        let clip = self.layout_stack.last().expect("No active frame").clip;
        let depth = style.depth.unwrap_or(0);

        if let Some(color) = style.bg {
            self.commands[depth].push(Command::Rect {
                x: rect.x as i32,
                y: rect.y as i32,
                width: rect.width,
                height: rect.height,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
            });
        }

        if let Some(color) = style.border {
            self.commands[depth].push(Command::RectOutline {
                x: rect.x as i32,
                y: rect.y as i32,
                width: rect.width,
                height: rect.height,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
                border_thickness: style.border_thickness.unwrap_or(1),
                border_sides: style.border_side.unwrap_or(border::ALL),
            });
        }
    }

    pub fn paint_triangle(&mut self, a: (usize, usize), b: (usize, usize), c: (usize, usize), style: Style) {
        let clip = self.layout_stack.last().expect("No active frame").clip;
        let depth = style.depth.unwrap_or(0);
        if let Some(color) = style.bg {
            self.commands[depth].push(Command::Triangle {
                a: (a.0 as i32, a.1 as i32),
                b: (b.0 as i32, b.1 as i32),
                c: (c.0 as i32, c.1 as i32),
                clip,
                color,
            });
        }
    }

    pub fn measure_text(&mut self, text: &str, font_id: usize, font_size: usize) -> Rect {
        let state = &mut *self.state;
        let font = &state.fonts[font_id];
        let metrics = state.font_metrics.entry(font_id).or_default();
        measure_text(text, font, font_size, 1.0, metrics)
    }

    pub fn paint_text(
        &mut self,
        text: impl Into<Cow<'a, str>>,
        paint_x: i32,
        paint_y: i32,
        width: usize,
        height: usize,
        color: u32,
        font_id: usize,
        font_size: usize,
        alignment: Alignment,
        padding: Padding,
        depth: usize,
    ) {
        let text = text.into();
        let text_metrics = self.measure_text(&text, font_id, font_size);

        let Some((x, y)) = align_rect(
            paint_x,
            paint_y,
            width,
            height,
            text_metrics.width,
            text_metrics.height,
            alignment,
            padding,
        ) else {
            return;
        };

        let clip = self.layout_stack.last().expect("No active frame").clip;
        self.commands[depth].push(Command::Text {
            text,
            clip,
            x,
            y,
            color,
            font_id,
            size: font_size,
        });
    }

    pub fn gap(&mut self, gap: impl IntoSize) {
        if let Some(gap) = gap.into_size() {
            let frame = self.layout_stack.last().expect("No active frame");
            let gap = self.resolve_size(gap, frame.flow);
            self.walk_layout(0, 0, gap);
        }
    }

    pub fn rect(&mut self, style: Style) -> State {
        self.item("", false, style)
    }

    pub fn text(&mut self, text: impl Into<Cow<'a, str>>, style: Style) -> State {
        self.item(text, false, style)
    }

    pub fn item(&mut self, text: impl Into<Cow<'a, str>>, selected: bool, style: Style) -> State {
        let text = text.into();
        let font_size = style.font_size.unwrap_or(self.default_font_size);
        let text_metrics = if text.is_empty() {
            Rect::default()
        } else {
            self.measure_text(&text, style.font, font_size)
        };

        let padding = style.padding.unwrap_or_default();
        let width = style
            .width
            .map(|w| self.resolve_size(w, Flow::Right))
            .unwrap_or(text_metrics.width + padding.left + padding.right);
        let height = style
            .height
            .map(|h| self.resolve_size(h, Flow::Down))
            .unwrap_or(text_metrics.height + padding.top + padding.bottom);

        let flow = self.layout_stack.last().expect("No active frame").flow;
        let gap = style.gap.map(|gap| self.resolve_size(gap, flow)).unwrap_or_default();
        let layout = self.walk_layout(width, height, gap);
        let rect = layout.size;

        if rect.width == 0 || rect.height == 0 {
            return State {
                clicked: false,
                hovered: false,
                rect,
            };
        }

        let clicked = self.clicked(rect);
        let hovered = self.hovered(rect);
        let depth = style.depth.unwrap_or(0);
        let clip = self.layout_stack.last().expect("No active frame").clip;

        let bg = if selected && style.selected.is_some() {
            style.selected
        } else if hovered && style.hover.is_some() {
            style.hover
        } else {
            style.bg
        };

        if let Some(color) = bg {
            self.commands[depth].push(Command::Rect {
                x: layout.paint_x,
                y: layout.paint_y,
                width,
                height,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
            });
        }

        let border = if selected && style.selected_border.is_some() {
            style.selected_border
        } else if style.border.is_some() {
            style.border
        } else {
            None
        };

        // TODO: Borders render inside of the bounding box
        // for text which means they can overlap...
        if let Some(border) = border {
            self.commands[depth].push(Command::RectOutline {
                x: layout.paint_x,
                y: layout.paint_y,
                width,
                height,
                clip,
                color: border,
                radius: style.radius.unwrap_or(0),
                border_thickness: style.border_thickness.unwrap_or(1),
                border_sides: style.border_side.unwrap_or(border::ALL),
            });
        }

        if !text.is_empty() {
            self.paint_text(
                text,
                layout.paint_x,
                layout.paint_y,
                width,
                height,
                style.fg.unwrap_or(white()),
                style.font,
                font_size,
                style.alignment.unwrap_or(Alignment::Center),
                style.padding.unwrap_or_default(),
                depth,
            );
        }

        State { clicked, hovered, rect }
    }

    pub fn flow<R>(
        &mut self,
        style: impl Into<Style>,
        flow: Flow,
        advance: bool,
        scroll_y: usize,
        ui: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let style = style.into();
        let mut bounds = self.current_frame_bounds();

        if let Some(width) = style.width {
            bounds.width = self.resolve_size(width, flow);
        }

        if let Some(height) = style.height {
            bounds.height = self.resolve_size(height, flow);
        }

        if let Some(x) = style.x {
            bounds.x = self.resolve_size(x, flow)
        }

        if let Some(y) = style.y {
            bounds.y = self.resolve_size(y, flow)
        }

        let clip = self.layout_stack.last().expect("No active frame").clip;
        let depth = style.depth.unwrap_or(0);
        let padding = style.padding.unwrap_or_default();

        bounds.x += padding.left;
        bounds.width = bounds.width.saturating_sub(padding.left + padding.right);
        bounds.y += padding.top;
        bounds.height = bounds.height.saturating_sub(padding.top + padding.bottom);

        // Draw the background first.
        if let Some(color) = style.bg {
            self.commands[depth].push(Command::Rect {
                x: bounds.x as i32,
                y: bounds.y as i32,
                width: bounds.width,
                height: bounds.height,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
            });
        }

        let new_frame = Frame {
            bounds,
            clip: clip.intersection(bounds),
            flow,
            cursor_x: bounds.x,
            cursor_y: bounds.y,
            scroll_y,
            ..Default::default()
        };

        self.layout_stack.push(new_frame);
        let result = ui(self);

        // Draw the border over the content, idk.
        if let Some(color) = style.border {
            self.commands[depth].push(Command::RectOutline {
                x: bounds.x as i32,
                y: bounds.y as i32,
                width: bounds.width,
                height: bounds.height,
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

    /// Layout widgets inside the container normally but don't add the area to the layout stack after.
    pub fn flow_skip<R>(&mut self, style: impl Into<Style>, flow: Flow, ui: impl FnOnce(&mut Self) -> R) -> R {
        let r = self.flow(style, flow, false, 0, ui);
        self.layout_stack.pop().expect("Layout underflow");
        r
    }

    pub fn scroll_view<R>(
        &mut self,
        style: impl Into<Style>,
        scroll_y: &mut usize,
        ui: impl FnOnce(&mut Self) -> R,
    ) -> ScrollState {
        let flow = Flow::Down;
        let style = style.into();
        let _ = self.flow(style, flow, false, *scroll_y, ui);

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

        let max_scroll = content_height.saturating_sub(bounds.height);
        let mut state = ScrollState {
            max_scroll,
            content_height,
            scrolled: false,
            direction: self.scroll_y,
        };

        if self.scroll_y != 0 && self.mouse_position().intersects(bounds) {
            const WHEEL_STEP: usize = 30;

            if self.scroll_y > 0 {
                *scroll_y = (*scroll_y).saturating_sub(self.scroll_y.unsigned_abs() as usize * WHEEL_STEP);
            } else {
                *scroll_y = (*scroll_y).saturating_add(self.scroll_y.unsigned_abs() as usize * WHEEL_STEP);
            }
            state.scrolled = true;
        }

        *scroll_y = (*scroll_y).clamp(0, max_scroll);

        state
    }

    #[deprecated]
    pub fn scroll<R>(&mut self, bounds: Option<Rect>, scroll_y: usize, ui: impl FnOnce(&mut Self) -> R) -> usize {
        let bounds = if let Some(bounds) = bounds {
            bounds
        } else {
            self.current_frame_bounds()
        };

        let parent_clip = self.layout_stack.last().map(|p| p.clip).unwrap_or(bounds);
        let new_frame = Frame {
            bounds,
            clip: parent_clip.intersection(bounds),
            flow: Flow::Down,
            cursor_x: bounds.x,
            cursor_y: bounds.y,
            scroll_y,
            ..Default::default()
        };
        self.layout_stack.push(new_frame);

        let _ = ui(self); //Probably fine.

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

        frame.max_child_height
    }

    pub fn current_frame(&self) -> &Frame {
        self.layout_stack.last().as_ref().unwrap()
    }

    pub fn current_frame_bounds(&self) -> Rect {
        let parent = self.layout_stack.last().expect("Layout stack empty");
        Rect::new(
            parent.cursor_x,
            parent.cursor_y,
            (parent.bounds.x + parent.bounds.width).saturating_sub(parent.cursor_x),
            (parent.bounds.y + parent.bounds.height).saturating_sub(parent.cursor_y),
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
            cursor_x: bounds.x,
            cursor_y: bounds.y,
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

    /// End layout without walking the max_child_width/height
    pub fn end_layout_absolute(&mut self) {
        self.layout_stack.pop().expect("Layout underflow");
    }

    pub fn begin_grid_cell(
        &mut self,
        col: usize,
        row: usize,
        col_width: usize,
        row_height: usize,
        grid_bounds: Rect,
        flow: Flow,
    ) {
        let cell_x = grid_bounds.x + (col * col_width);
        let cell_y = grid_bounds.y + (row * row_height);

        let cell_w = col_width.min(grid_bounds.width.saturating_sub(col * col_width));
        let cell_h = row_height.min(grid_bounds.height.saturating_sub(row * row_height));

        self.begin_layout(flow, Some(Rect::new(cell_x, cell_y, cell_w, cell_h)));
    }

    fn draw_frame(&mut self) {
        profile!();
        let window = &mut *self.window;
        let state = &mut *self.state;
        let (self_width, self_height) = window.content_size();
        let display_scale = window.scale_factor() as f32;
        let framebuffer_width = scale(self_width, display_scale);
        let framebuffer_height = scale(self_height, display_scale);
        let buffer = window.framebuffer();

        for layer in &mut state.commands {
            for cmd in layer.drain(..) {
                match cmd {
                    Command::Rect {
                        x,
                        y,
                        width,
                        height,
                        clip,
                        color,
                        radius,
                    } => draw_rounded_rect(
                        buffer,
                        scale_f32(x as f32, display_scale),
                        scale_f32(y as f32, display_scale),
                        scale(width, display_scale),
                        scale(height, display_scale),
                        framebuffer_width,
                        framebuffer_height,
                        scale(radius, display_scale),
                        color,
                        clip.scale(display_scale),
                    ),
                    Command::RectOutline {
                        x,
                        y,
                        width,
                        height,
                        clip,
                        color,
                        radius: _,
                        border_thickness: _,
                        border_sides,
                    } => draw_rect_outline(
                        buffer,
                        scale_f32(x as f32, display_scale),
                        scale_f32(y as f32, display_scale),
                        scale(width, display_scale),
                        scale(height, display_scale),
                        framebuffer_width,
                        color,
                        clip.scale(display_scale),
                        border_sides,
                    ),
                    Command::Text {
                        text,
                        clip,
                        x,
                        y,
                        color,
                        size,
                        font_id,
                    } => {
                        let bitmap = state.font_bitmaps.entry(font_id).or_default();
                        draw_text(
                            &text,
                            &state.fonts[font_id],
                            x,
                            y,
                            size,
                            display_scale,
                            framebuffer_width,
                            buffer,
                            color,
                            bitmap,
                            clip.scale(display_scale),
                        );
                    }
                    // Command::Icon {
                    //     icon,
                    //     font,
                    //     clip,
                    //     x,
                    //     y,
                    //     color,
                    //     size,
                    // } => {
                    //     //The library should probably handle font loading
                    //     //and each font should have a unique ID that would be
                    //     //used here instead of a pointer.
                    //     let font_key = font as *const fontdue::Font as usize;
                    //     let cache = self.icon_glyph_cache.entry(font_key).or_default();
                    //     let mut icon_buffer = [0; 4];
                    //     let icon = icon.encode_utf8(&mut icon_buffer);
                    //     draw_text(
                    //         icon,
                    //         font,
                    //         x,
                    //         y,
                    //         size,
                    //         self.window.display_scale(),
                    //         self_width,
                    //         &mut self.window.buffer,
                    //         color,
                    //         cache,
                    //         clip,
                    //     );
                    // }
                    Command::Triangle {
                        a: (ax, ay),
                        b: (bx, by),
                        c: (cx, cy),
                        clip,
                        color,
                    } => draw_triangle_sdf(
                        buffer,
                        framebuffer_width,
                        framebuffer_height,
                        scale_f32(ax as f32, display_scale),
                        scale_f32(ay as f32, display_scale),
                        scale_f32(bx as f32, display_scale),
                        scale_f32(by as f32, display_scale),
                        scale_f32(cx as f32, display_scale),
                        scale_f32(cy as f32, display_scale),
                        color,
                        clip.scale(display_scale),
                    ),
                };
            }
        }
    }

    pub fn animate_f32(&mut self, target: f32, duration: f32, ease: Ease) -> f32 {
        let id = self.anim_counter;
        self.anim_counter += 1;
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

            let eased_t = apply_ease(t, ease);

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
        let id = self.anim_counter;
        self.anim_counter += 1;
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
