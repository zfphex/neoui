#![feature(portable_simd)]
#![feature(const_trait_impl)]
pub mod style;
pub use style::*;

pub mod shapes;
pub use shapes::*;

pub mod render_cache;
pub use render_cache::*;

pub mod image;
pub use image::*;

pub mod scroll;
pub use scroll::*;

pub mod accessability;
pub use accessability::*;

pub use mini::*;
pub use minwin::*;

use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::cell::{Cell, UnsafeCell};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

pub use fontdue;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Flow {
    #[default]
    Down,
    Right,
    Up,
    Left,
}

impl Flow {
    pub const fn vertical(self) -> bool {
        matches!(self, Flow::Down | Flow::Up)
    }

    pub const fn reverse(self) -> bool {
        matches!(self, Flow::Up | Flow::Left)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Frame {
    pub inner_bounds: Rect,
    /// bounds without padding
    pub outer_bounds: Rect,
    pub bleed: bool,
    pub clip: Rect,
    pub flow: Flow,
    pub align_children: Align,
    pub depth: usize,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub max_child_width: i32,
    pub max_child_height: i32,
    pub scroll_y: i32,
    pub padding: Padding,
    pub gap: i32,
    pub outer_width: Option<i32>,
    pub outer_height: Option<i32>,
    pub scope: usize,
    pub child_index: usize,
    pub anim_slot: usize,
}

impl Frame {
    pub fn fitted_size(&self) -> (i32, i32) {
        let (mut w, mut h) = (self.max_child_width, self.max_child_height);
        if self.flow.vertical() {
            h = h.saturating_sub(self.gap);
        } else {
            w = w.saturating_sub(self.gap);
        }
        let (w, h) = (
            self.outer_width
                .unwrap_or(w + (self.padding.left + self.padding.right) as i32),
            self.outer_height
                .unwrap_or(h + (self.padding.top + self.padding.bottom) as i32),
        );
        if self.bleed {
            (w.min(self.outer_bounds.width), h.min(self.outer_bounds.height))
        } else {
            (w, h)
        }
    }
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

pub const MAX_GRADIENT_STOPS: usize = 5;

pub const SCOPE_SEED: u64 = 0x9e3779b97f4a7c15;

static DEBUG_DAMAGE_SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(SCOPE_SEED);

#[derive(Debug, Clone, Copy)]
pub struct Gradient {
    pub stops: [(f32, u32); MAX_GRADIENT_STOPS],
    pub count: u8,
    pub angle: f32,
}

impl Hash for Gradient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (position, color) in &self.stops[..self.count as usize] {
            position.to_bits().hash(state);
            color.hash(state);
        }
        self.angle.to_bits().hash(state);
    }
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
    Circle {
        bounds: Rect,
        clip: Rect,
        color: u32,
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
        line_height: Option<usize>,
        alignment: Alignment,
        breaks: (u32, u32),
    },
    Image {
        image: Image<'a>,
        bounds: Rect,
        clip: Rect,
        opacity: u8,
        radius: usize,
    },
    Gradient {
        bounds: Rect,
        clip: Rect,
        radius: usize,
        gradient: Gradient,
    },
}

impl<'a> Command<'a> {
    pub fn clip(&self) -> Rect {
        match self {
            Command::Rect { clip, .. }
            | Command::RectStroke { clip, .. }
            | Command::Circle { clip, .. }
            | Command::Triangle { clip, .. }
            | Command::Text { clip, .. }
            | Command::Image { clip, .. }
            | Command::Gradient { clip, .. } => *clip,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextMetrics {
    pub width: i32,
    pub height: i32,
    pub start_linebreak: u32,
    pub end_linebreak: u32,
}

/// The space a widget reserved, in screen coordinates.
#[derive(Debug, Clone, Copy, Default)]
pub struct Walk {
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
    pub bounds: Rect,
    pub focused: bool,
    pub activated: bool,
}

impl State {
    pub fn new(bounds: Rect) -> Self {
        State {
            pressed: false,
            released: false,
            clicked: false,
            double_clicked: false,
            hovered: false,
            bounds,
            focused: false,
            activated: false,
        }
    }
}

fn align_cross(start: i32, extent: i32, size: i32, align: Align) -> i32 {
    match align {
        Align::Start => start,
        Align::Center => start + extent.saturating_sub(size) / 2,
        Align::End => start + extent.saturating_sub(size),
    }
}

fn resolve_bg(style: &Paint, hovered: bool) -> Option<u32> {
    if style.is_selected && style.selected.is_some() {
        style.selected
    } else if hovered && style.hover.is_some() {
        style.hover
    } else {
        style.bg
    }
}

fn resolve_border(style: &Paint, hovered: bool) -> Option<u32> {
    if style.is_selected && style.selected_border.is_some() {
        style.selected_border
    } else if hovered && style.hover_border.is_some() {
        style.hover_border
    } else {
        style.border
    }
}

pub const DEFAULT_FONT: &[u8] = include_bytes!("../fonts/Aptos.ttf");

pub fn family(id: usize) -> [[Option<usize>; 9]; 2] {
    let mut faces = [[None; 9]; 2];
    faces[false as usize][Weight::Regular as usize] = Some(id);
    faces
}

pub fn ui(title: &str, width: usize, height: usize) -> Context {
    let window = create_window(title, None, width as i32, height as i32, false, WindowStyle::Standard);
    Context::new(window)
}

pub fn ui_hidden(width: usize, height: usize) -> Context {
    let mut window = create_window(
        "hidden",
        None,
        width as i32,
        height as i32,
        false,
        WindowStyle::Borderless,
    );

    window.hide();
    window.set_size(width as i32, height as i32);

    let mut context = Context::new(window);
    context.vsync = false;
    context
}

pub struct Context {
    pub window: std::pin::Pin<Box<Window>>,
    pub state: UiState,
}

impl Context {
    pub fn new(window: std::pin::Pin<Box<Window>>) -> Self {
        Context {
            window,
            state: UiState {
                fonts: vec![fontdue::Font::from_bytes(DEFAULT_FONT, fontdue::FontSettings::default()).unwrap()],
                fallbacks: Vec::new(),
                families: vec![family(0)],
                layout_stack: Vec::new(),
                font_bitmaps: FxHashMap::default(),
                image_columns: Vec::new(),
                debug_damage: false,
                debug_damage_fade: 1.5,
                debug_damage_cache: Vec::new(),
                font_metrics: FxHashMap::default(),
                text_measure_cache: FxHashMap::default(),
                line_breaks: Vec::new(),
                default_font_size: 32,
                clear_color: black(),
                scroll_y: 0.0,
                scroll_events: Vec::new(),
                left_mouse_start: None,
                left_mouse_release: None,
                dt: 0.0,
                id_stack: Vec::new(),
                anim_state_f32: FxHashMap::default(),
                anim_state_color: FxHashMap::default(),
                last_frame_time: std::time::Instant::now(),
                animating: false,
                hovered_depth: None,
                render_cache: RenderCache::default(),
                commands: [const { Vec::new() }; 16],
                vsync: true,
                string_pool: UnsafeCell::new(Vec::with_capacity(128)),
                string_index: Cell::new(0),
                accessability: true,
                accessability_state: AccessabilityState::new(),
            },
        }
    }
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
    /// Font IDs searched in order when the requested font has no glyph for a character.
    pub fallbacks: Vec<usize>,
    /// Maps a (family, italic, weight) request onto the font ID holding that face.
    pub families: Vec<[[Option<usize>; 9]; 2]>,
    pub font_bitmaps: FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    /// Scratch buffer of source columns reused by every image blit.
    pub image_columns: Vec<u32>,
    pub debug_damage: bool,
    pub debug_damage_fade: f32,
    pub debug_damage_cache: Vec<(Rect, u32, std::time::Instant)>,
    pub font_metrics: FxHashMap<(usize, char, usize), fontdue::Metrics>,
    /// (text hash, font ID, font size, line height, wrap width).
    /// The two indices bound this run's slice of line_breaks.
    pub text_measure_cache: FxHashMap<(u64, usize, usize, Option<usize>, i32), TextMetrics>,
    /// Byte offset pairs, marking where each line starts and ends.
    pub line_breaks: Vec<u32>,
    pub default_font_size: usize,
    pub vsync: bool,
    pub clear_color: u32,
    pub scroll_y: f32,
    pub scroll_events: Vec<ScrollEvent>,

    pub dt: f32,
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
    pub commands: [Vec<Command<'static>>; 16],
    pub string_pool: UnsafeCell<Vec<Box<String>>>,
    pub string_index: Cell<usize>,
    pub accessability: bool,
    pub accessability_state: AccessabilityState,
}

impl UiState {
    pub fn add_font(&mut self, font: fontdue::Font) -> Font {
        let id = self.fonts.len();
        self.fonts.push(font);
        self.families.push(family(id));
        Font {
            id,
            weight: Weight::Regular,
            italic: false,
        }
    }

    pub fn add_face(&mut self, family: usize, weight: Weight, italic: bool, font: fontdue::Font) {
        let f = self.add_font(font);
        self.families[family][italic as usize][weight as usize] = Some(f.id);
    }

    pub fn face(&self, font: Font) -> usize {
        self.families[font.id][font.italic as usize][font.weight as usize].unwrap_or_else(|| {
            panic!(
                "font {} has no {:?}{} face",
                font.id,
                font.weight,
                if font.italic { " italic" } else { "" }
            )
        })
    }

    pub fn add_font_fallback(&mut self, font: fontdue::Font) {
        let font = self.add_font(font);
        self.fallbacks.push(font.id);
    }
}

pub struct FrameContext<'frame, 'a> {
    pub window: &'frame mut Window,
    pub commands: [Vec<Command<'a>>; 16],
    pub state: &'frame mut UiState,
}

impl<'frame, 'a> FrameContext<'frame, 'a> {
    pub fn fmt(&self, format_args: std::fmt::Arguments<'_>) -> &'a str {
        use std::fmt::Write;
        let index = self.state.string_index.get();
        self.state.string_index.set(index + 1);
        let buffer = unsafe {
            let pool = &mut *self.state.string_pool.get();
            if index >= pool.len() {
                pool.push(Box::new(String::with_capacity(64)));
            }
            &mut *(&raw mut *pool[index])
        };
        buffer.clear();
        let _ = buffer.write_fmt(format_args);
        unsafe { std::mem::transmute::<&str, &'a str>(buffer.as_str()) }
    }
}

pub struct GradientStops<'ui, 'frame, 'text> {
    pub slot: Option<(usize, usize)>,
    ui: &'ui mut FrameContext<'frame, 'text>,
    state: State,
}

impl GradientStops<'_, '_, '_> {
    pub fn stop(self, position: f32, color: u32) -> Self {
        if let Some((depth, index)) = self.slot
            && let Command::Gradient { gradient, .. } = &mut self.ui.commands[depth][index]
        {
            assert!(
                (gradient.count as usize) < MAX_GRADIENT_STOPS,
                "a gradient holds at most {MAX_GRADIENT_STOPS} stops"
            );
            gradient.stops[gradient.count as usize] = (position, color);
            gradient.count += 1;
        }
        self
    }
}

impl Deref for GradientStops<'_, '_, '_> {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl Deref for FrameContext<'_, '_> {
    type Target = UiState;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl DerefMut for FrameContext<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Context {
    pub fn frame<'a, F>(&'a mut self, mut ui: F)
    where
        F: for<'frame> FnMut(&mut FrameContext<'frame, 'a>),
    {
        let state = &mut self.state;

        self.window.draw(|window| {
            state.string_index.set(0);
            //Safety: Requires that commands are cleared each frame and adhear to the 'a lifetime.
            let commands: [Vec<Command<'a>>; 16] = unsafe { std::mem::transmute(std::mem::take(&mut state.commands)) };
            let mut frame = FrameContext {
                window,
                state,
                commands,
            };
            let now = std::time::Instant::now();
            frame.dt = (now - frame.last_frame_time).as_secs_f32();
            frame.last_frame_time = now;
            frame.id_stack.clear();
            frame.animating = false;
            frame.scroll_y = frame.window.scroll_delta().1 as f32;
            frame.state.scroll_events.clear();
            frame
                .state
                .scroll_events
                .extend_from_slice(frame.window.scroll_events());
            frame.hovered_depth = None;

            if frame.window.mouse_pressed(Mouse::Left) {
                frame.left_mouse_start = Some(frame.mouse_position());
                frame.left_mouse_release = None;
            }

            if frame.window.mouse_released(Mouse::Left) {
                frame.left_mouse_release = Some(frame.mouse_position());
            }

            if frame.state.accessability {
                frame
                    .state
                    .accessability_state
                    .begin_frame(Some(frame.window), frame.hovered_depth);
            }

            let (width, height) = frame.window.size();
            let bounds = Rect::new(0, 0, width as i32, height as i32);

            frame.layout_stack.clear();
            frame.layout_stack.push(Frame {
                inner_bounds: bounds,
                outer_bounds: bounds,
                clip: bounds,
                flow: Flow::Down,
                ..Default::default()
            });

            ui(&mut frame);

            if frame.state.accessability {
                frame.state.accessability_state.end_frame(frame.hovered_depth);
            }

            frame.draw_frame();

            if frame.window.mouse_released(Mouse::Left) {
                frame.left_mouse_start = None;
                frame.left_mouse_release = None;
            }

            for layer in &mut frame.commands {
                layer.clear();
            }

            frame.state.commands = unsafe { std::mem::transmute(frame.commands) };
        });

        if !self.window.open() {
            return;
        }

        if self.vsync {
            self.window.wait_for_vsync();
        }

        if !self.state.animating {
            self.state.last_frame_time = std::time::Instant::now();
        }
    }

    ///Render a single frame to a png.
    pub fn frame_hidden<F>(&mut self, path: &str, ui: F) -> std::io::Result<()>
    where
        F: for<'frame, 'text> FnMut(&mut FrameContext<'frame, 'text>),
    {
        self.frame(ui);
        let (width, height) = self.window.scaled_size();
        let buffer = self.window.framebuffer();
        let bgra = unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, buffer.len() * 4) };
        write_png(path, width, height, bgra, width * 4)
    }
}

impl<'frame, 'a> FrameContext<'frame, 'a> {
    /// Force the current frame to rebuild the complete framebuffer.
    pub fn invalidate_render_cache(&mut self) {
        self.state.render_cache.invalidate();
    }

    pub fn next_scope(&mut self) -> usize {
        let Some(parent) = self.layout_stack.last_mut() else {
            return 0;
        };
        let index = parent.child_index;
        parent.child_index += 1;
        let mut hasher = rustc_hash::FxHasher::default();
        SCOPE_SEED.hash(&mut hasher);
        parent.scope.hash(&mut hasher);
        index.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Walk the layout forward by an explicit size and return the screen-space bounding box.
    /// `align` overrides the parent's `align_children` for this child only.
    pub fn walk_layout(&mut self, width: i32, height: i32, gap: i32, align: Option<Align>) -> Walk {
        let frame = self.layout_stack.last_mut().expect("No active layout frame");
        let align = align.unwrap_or(frame.align_children);
        let (x, y) = if frame.flow.vertical() {
            (
                align_cross(frame.inner_bounds.x, frame.inner_bounds.width, width, align),
                if frame.flow.reverse() { frame.cursor_y - height } else { frame.cursor_y },
            )
        } else {
            (
                if frame.flow.reverse() { frame.cursor_x - width } else { frame.cursor_x },
                align_cross(frame.inner_bounds.y, frame.inner_bounds.height, height, align),
            )
        };
        let rect = Rect::new(x, y, width, height);

        match frame.flow {
            Flow::Down | Flow::Up => {
                let step = height + gap;
                frame.cursor_y += if frame.flow.reverse() { -step } else { step };
                frame.max_child_width = frame.max_child_width.max(width);
                frame.max_child_height += step;
            }
            Flow::Right | Flow::Left => {
                let step = width + gap;
                frame.cursor_x += if frame.flow.reverse() { -step } else { step };
                frame.max_child_width += step;
                frame.max_child_height = frame.max_child_height.max(height);
            }
        }

        if frame.scroll_y == 0 {
            return Walk {
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
        let paint_x = rect.x;
        let paint_y = rect.y - frame.scroll_y;
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

        if paint_y + height <= clip_top {
            return Walk {
                paint_x,
                paint_y,
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

        if paint_y >= clip_bottom {
            return Walk {
                paint_x,
                paint_y,
                ..Default::default()
            };
        }

        let visible_y = paint_y.max(clip_top).max(0);
        let visible_bottom = (paint_y + height).min(clip_bottom).max(0);

        Walk {
            size: Rect::new(rect.x, visible_y, width, visible_bottom.saturating_sub(visible_y)),
            paint_x,
            paint_y,
        }
    }

    pub fn resolve_rect(&self, rect: Rect, flow: Flow, size: Size) -> i32 {
        let total = if flow.vertical() { rect.height } else { rect.width };
        match size {
            Size::Pixel(px) => px,
            Size::Percentage(pct) => (total as f32 * pct) as i32,
            Size::Fill => total,
            Size::FillMinus(sub) => total.saturating_sub(sub.abs()).max(0),
        }
    }

    pub fn split_rect_h(&self, rect: Rect, size: impl IntoSize) -> (Rect, Rect) {
        let left_width = self.resolve_rect(rect, Flow::Right, size.into_size().unwrap_or_default());
        let total_w = rect.width.max(0);
        let total_h = rect.height.max(0);
        let left_w = left_width.clamp(0, total_w);
        let right_w = total_w.saturating_sub(left_w);
        let left_rect = Rect::new(rect.x, rect.y, left_w, total_h);
        let right_rect = Rect::new(rect.x + left_w, rect.y, right_w, total_h);
        (left_rect, right_rect)
    }

    pub fn split_rect_v(&self, rect: Rect, size: impl IntoSize) -> (Rect, Rect) {
        let top_height = self.resolve_rect(rect, Flow::Down, size.into_size().unwrap_or_default());
        let total_w = rect.width.max(0);
        let total_h = rect.height.max(0);
        let top_h = top_height.clamp(0, total_h);
        let bottom_h = total_h.saturating_sub(top_h);
        let top_rect = Rect::new(rect.x, rect.y, total_w, top_h);
        let bottom_rect = Rect::new(rect.x, rect.y + top_h, total_w, bottom_h);
        (top_rect, bottom_rect)
    }

    pub fn split_hs<const N: usize>(&self, rect: Rect, weights: [f32; N]) -> [Rect; N] {
        let total: f32 = weights.iter().sum();
        let mut cols = [Rect::default(); N];
        let mut acc = 0.0;
        let mut x = rect.x;
        for (col, weight) in cols.iter_mut().zip(weights) {
            acc += weight;
            let edge = rect.x + (rect.width as f32 * acc / total).round() as i32;
            *col = Rect::new(x, rect.y, (edge - x).max(0), rect.height);
            x = edge;
        }
        cols
    }

    pub fn split_vs<const N: usize>(&self, rect: Rect, weights: [f32; N]) -> [Rect; N] {
        let total: f32 = weights.iter().sum();
        let mut rows = [Rect::default(); N];
        let mut acc = 0.0;
        let mut y = rect.y;
        for (row, weight) in rows.iter_mut().zip(weights) {
            acc += weight;
            let edge = rect.y + (rect.height as f32 * acc / total).round() as i32;
            *row = Rect::new(rect.x, y, rect.width, (edge - y).max(0));
            y = edge;
        }
        rows
    }

    /// Splits the current frame's remaining space horizontally.
    pub fn split_h(&self, left_width: impl IntoSize) -> (Rect, Rect) {
        let left_width = self.resolve_size(left_width.into_size().unwrap_or_default(), Flow::Right);
        let remaining = self.current_frame_bounds();

        let left_w = left_width.min(remaining.width);
        let right_w = remaining.width.saturating_sub(left_w);

        let left_rect = Rect::new(remaining.x, remaining.y, left_w, remaining.height);
        let right_rect = Rect::new(remaining.x + left_w, remaining.y, right_w, remaining.height);

        (left_rect, right_rect)
    }

    /// Splits the current frame's remaining space vertically.
    pub fn split_v(&self, top_height: impl IntoSize) -> (Rect, Rect) {
        let top_height = self.resolve_size(top_height.into_size().unwrap_or_default(), Flow::Down);
        let remaining = self.current_frame_bounds();

        let top_h = top_height.min(remaining.height);
        let bottom_h = remaining.height.saturating_sub(top_h);

        let top_rect = Rect::new(remaining.x, remaining.y, remaining.width, top_h);
        let bottom_rect = Rect::new(remaining.x, remaining.y + top_h, remaining.width, bottom_h);

        (top_rect, bottom_rect)
    }

    pub fn resolve_size_in(&self, bounds: Rect, size: Size, flow: Flow, start_pos: i32) -> i32 {
        let (total, start, end) = if flow.vertical() {
            (bounds.height, bounds.y, bounds.bottom())
        } else {
            (bounds.width, bounds.x, bounds.right())
        };
        let remaining = if flow.reverse() { start_pos - start } else { end - start_pos }.max(0);
        match size {
            Size::Pixel(px) => px,
            Size::Percentage(pct) => (total as f32 * pct) as i32,
            Size::Fill => remaining,
            Size::FillMinus(sub) => (remaining - sub.abs()).max(0),
        }
    }

    pub fn resolve_size_relative(&self, size: Size, flow: Flow, start_pos: i32) -> i32 {
        let frame = self.layout_stack.last().expect("No active frame");
        self.resolve_size_in(frame.inner_bounds, size, flow, start_pos)
    }

    pub fn resolve_size(&self, size: Size, flow: Flow) -> i32 {
        self.resolve_style_size(size, flow, &Layout::new())
    }

    pub fn resolve_style_size(&self, size: Size, flow: Flow, style: &Layout) -> i32 {
        let frame = self.layout_stack.last().expect("No active frame");
        let bounds = if style.bleed { frame.outer_bounds } else { frame.inner_bounds };
        let start_pos = if flow.vertical() { frame.cursor_y } else { frame.cursor_x };
        self.resolve_size_in(bounds, size, flow, start_pos)
    }

    #[inline]
    pub fn hit(&self, rect: Rect) -> Rect {
        match self.layout_stack.last() {
            Some(frame) => rect.intersection(frame.clip),
            None => rect,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn clicked(&self, rect: Rect) -> bool {
        self.window.mouse_clicked(Mouse::Left, self.hit(rect))
    }

    //Trackpads are really awful if you use standard click behaviour.
    //The user could plug in a mouse so this is not great.
    //TODO: Programatically change based on user input device.
    #[cfg(target_os = "macos")]
    pub fn clicked(&self, rect: Rect) -> bool {
        self.pressed(rect)
    }

    pub fn double_clicked(&self, rect: Rect) -> bool {
        self.window.mouse_double_clicked(Mouse::Left, self.hit(rect))
    }

    pub fn pressed(&self, rect: Rect) -> bool {
        self.window.mouse_pressed(Mouse::Left) && self.mouse_position().intersects(self.hit(rect))
    }

    pub fn released(&self, rect: Rect) -> bool {
        self.window.mouse_released(Mouse::Left) && self.mouse_position().intersects(self.hit(rect))
    }

    pub fn hovered(&self, rect: Rect) -> bool {
        self.mouse_position().intersects(self.hit(rect))
    }

    pub fn interact(&mut self, rect: Rect, depth: usize) -> State {
        let hovered = self.hovered_depth(rect, depth);
        let clicked = hovered && self.clicked(rect);

        let (focused, key_activated) = if self.state.accessability {
            let focused = self.state.accessability_state.is_focused(rect, Role::FOCUSABLE);
            let key_activated = focused && (self.window.pressed(Key::Enter) || self.window.pressed(Key::Space));
            (focused, key_activated)
        } else {
            (false, false)
        };

        State {
            clicked,
            double_clicked: hovered && self.double_clicked(rect),
            pressed: hovered && self.pressed(rect),
            released: hovered && self.released(rect),
            hovered,
            bounds: rect,
            focused,
            activated: clicked || key_activated,
        }
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

        self.window.mouse_down(Mouse::Left) && initial.intersects(self.hit(rect))
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

        Rect::new(x as i32, y as i32, 1, 1)
    }

    pub fn paint_rect(&mut self, rect: Rect, style: RectStyle) {
        let frame = self.current_frame();
        let clip = frame.clip;
        let depth = style.layout.depth.unwrap_or(frame.depth);

        if let Some(color) = style.paint.bg {
            self.commands[depth].push(Command::Rect {
                bounds: rect,
                clip,
                color,
                radius: style.paint.radius.unwrap_or(0),
            });
        }

        if let Some(color) = style.paint.border {
            self.commands[depth].push(Command::RectStroke {
                bounds: rect,
                clip,
                color,
                radius: style.paint.radius.unwrap_or(0),
                border_thickness: style.paint.border_thickness.unwrap_or(1),
                border_sides: style.paint.border_side.unwrap_or(border::ALL),
            });
        }
    }

    pub fn paint_triangle(&mut self, a: (i32, i32), b: (i32, i32), c: (i32, i32), style: RectStyle) {
        let frame = self.current_frame();
        let clip = frame.clip;
        let depth = style.layout.depth.unwrap_or(frame.depth);
        if let Some(color) = style.paint.bg {
            self.commands[depth].push(Command::Triangle { a, b, c, clip, color });
        }
    }

    pub fn paint_circle(&mut self, bounds: Rect, style: RectStyle) {
        let frame = self.current_frame();
        let clip = frame.clip;
        let depth = style.layout.depth.unwrap_or(frame.depth);
        if let Some(color) = style.paint.bg.or(style.paint.fg) {
            self.commands[depth].push(Command::Circle { bounds, clip, color });
        }
    }

    pub fn paint_text_measured(
        &mut self,
        text: impl Into<Cow<'a, str>>,
        metrics: TextMetrics,
        rect: Rect,
        color: u32,
        font: Font,
        font_size: usize,
        line_height: Option<usize>,
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
            font_id: self.face(font),
            size: font_size,
            line_height,
            alignment,
            breaks: (metrics.start_linebreak, metrics.end_linebreak),
        });
    }

    pub fn paint_text(
        &mut self,
        text: impl Into<Cow<'a, str>>,
        rect: Rect,
        color: u32,
        font: Font,
        font_size: usize,
        line_height: Option<usize>,
        alignment: Alignment,
        padding: Padding,
        depth: usize,
    ) {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        let metrics = self.measure_text(&text, font, font_size, line_height, i32::MAX);
        self.paint_text_measured(
            text,
            metrics,
            rect,
            color,
            font,
            font_size,
            line_height,
            alignment,
            padding,
            depth,
        );
    }

    pub fn measure_text(
        &mut self,
        text: &str,
        font: Font,
        font_size: usize,
        line_height: Option<usize>,
        max_width: i32,
    ) -> TextMetrics {
        if text.is_empty() || font_size == 0 {
            return TextMetrics::default();
        }
        let font_id = self.face(font);
        let state = &mut *self.state;

        let mut hasher = rustc_hash::FxHasher::default();
        text.hash(&mut hasher);
        let text_hash = hasher.finish();
        let key = (text_hash, font_id, font_size, line_height, max_width);

        if let Some(metrics) = state.text_measure_cache.get(&key) {
            return *metrics;
        }

        let start_linebreak = state.line_breaks.len() as u32;
        let (width, height) = measure_text(
            text,
            &state.fonts,
            font_id,
            &state.fallbacks,
            font_size,
            line_height,
            max_width,
            &mut state.font_metrics,
            &mut state.line_breaks,
        );
        let end_linebreak = state.line_breaks.len() as u32;
        let metrics = TextMetrics {
            width,
            height,
            start_linebreak,
            end_linebreak,
        };
        state.text_measure_cache.insert(key, metrics);
        metrics
    }

    pub fn gap(&mut self, gap: impl IntoSize) {
        if let Some(gap) = gap.into_size() {
            let frame = self.layout_stack.last().expect("No active frame");
            let gap = self.resolve_size(gap, frame.flow);
            self.walk_layout(0, 0, gap, None);
        }
    }

    #[inline]
    pub fn rect(&mut self, style: RectStyle) -> State {
        let role = style.role.unwrap_or_else(|| {
            if style.paint.hover.is_some()
                || style.paint.hover_border.is_some()
                || style.paint.is_selected
                || style.paint.selected.is_some()
                || style.paint.selected_border.is_some()
            {
                Role::BUTTON
            } else {
                Role::NONE
            }
        });
        let hint = style.hint.unwrap_or("");
        self.widget_with_role(0, 0, &style.layout, &style.paint, role, "", hint, |_, _, _, _| {})
    }

    pub fn circle(&mut self, mut style: RectStyle) -> State {
        if style.layout.width.is_none() && style.layout.height.is_none() {
            if let Some(r) = style.paint.radius {
                let diameter = (r * 2) as i32;
                style.layout.width = Some(Size::Pixel(diameter));
                style.layout.height = Some(Size::Pixel(diameter));
            }
        }
        let paint = style.paint;
        // The circle command paints the fill, so the box must not paint it as a rect.
        let mut box_paint = paint;
        box_paint.bg = None;
        self.widget(0, 0, &style.layout, &box_paint, |ui, content, state, depth| {
            let color = if paint.is_selected && paint.selected.is_some() {
                paint.selected
            } else if state.hovered && paint.hover.is_some() {
                paint.hover
            } else {
                paint.bg.or(paint.fg)
            };
            if let Some(color) = color {
                let clip = ui.current_frame().clip;
                ui.commands[depth].push(Command::Circle {
                    bounds: content,
                    clip,
                    color,
                });
            }
        })
    }

    #[inline]
    pub fn emit_semantic(&mut self, bounds: Rect, role: Role, text: &str, hint: &str, state: StateFlags) -> usize {
        if !self.state.accessability {
            return 0;
        }
        let text_start = self.state.accessability_state.text_arena.len() as u32;
        self.state.accessability_state.text_arena.push_str(text);
        let text_end = self.state.accessability_state.text_arena.len() as u32;

        let hint_start = self.state.accessability_state.text_arena.len() as u32;
        if !hint.is_empty() {
            self.state.accessability_state.text_arena.push_str(hint);
        }
        let hint_end = self.state.accessability_state.text_arena.len() as u32;

        let sig = hash32(text);
        let depth = self.current_frame().depth;

        let index = self.state.accessability_state.current_nodes.len();
        let mut node = SemanticNode::new(
            bounds,
            text_start..text_end,
            role,
            state,
            depth,
            sig,
        );
        node.hint_range = (hint_start, hint_end);
        self.state.accessability_state.current_nodes.push(node);
        index
    }

    #[inline]
    pub fn is_focused(&self, bounds: Rect, role: Role) -> bool {
        if !self.state.accessability {
            return false;
        }
        self.state.accessability_state.is_focused(bounds, role)
    }

    #[inline]
    pub fn focus_cursor(&self) -> Option<SpatialCursor> {
        if !self.state.accessability {
            return None;
        }
        self.state.accessability_state.cursor
    }

    #[inline]
    pub fn set_focus_cursor(&mut self, cursor: Option<SpatialCursor>) {
        if self.state.accessability {
            self.state.accessability_state.cursor = cursor;
        }
    }

    #[inline]
    pub fn widget_with_role(
        &mut self,
        content_width: i32,
        content_height: i32,
        layout: &Layout,
        style: &Paint,
        role: Role,
        text: &str,
        hint: &str,
        paint: impl FnOnce(&mut Self, Rect, &State, usize),
    ) -> State {
        let padding = layout.padding.unwrap_or_default();
        let width = layout
            .width
            .map(|w| self.resolve_style_size(w, Flow::Right, layout))
            .unwrap_or(content_width + padding.left as i32 + padding.right as i32);
        let height = layout
            .height
            .map(|h| self.resolve_style_size(h, Flow::Down, layout))
            .unwrap_or(content_height + padding.top as i32 + padding.bottom as i32);

        let (paint_x, paint_y, rect, paint_bounds) = self.resolve_item_layout(width, height, layout);

        if rect.is_empty() {
            return State::new(rect);
        }

        let frame = self.current_frame();
        let depth = layout.depth.unwrap_or(frame.depth);
        let clip = frame.clip;
        let mut state = self.interact(rect, depth);

        if self.state.accessability && !role.is_empty() {
            if role.is_focusable() && state.clicked {
                let centroid = (
                    rect.x as f32 + rect.width as f32 * 0.5,
                    rect.y as f32 + rect.height as f32 * 0.5,
                );
                self.state.accessability_state.cursor = Some(SpatialCursor::new(
                    centroid,
                    role,
                    hash32(text),
                    self.state.accessability_state.current_nodes.len(),
                    depth,
                ));
                state.focused = true;
            }

            let state_flags = if state.focused { StateFlags::FOCUSED } else { StateFlags::NONE }
                | if state.hovered { StateFlags::HOVERED } else { StateFlags::NONE }
                | if style.is_selected { StateFlags::SELECTED } else { StateFlags::NONE };
            self.emit_semantic(rect, role, text, hint, state_flags);
        }

        if let Some(color) = resolve_bg(style, state.hovered) {
            self.commands[depth].push(Command::Rect {
                bounds: paint_bounds,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
            });
        }

        let content = Rect::new(
            paint_x + padding.left as i32,
            paint_y + padding.top as i32,
            (width - (padding.left + padding.right) as i32).max(0),
            (height - (padding.top + padding.bottom) as i32).max(0),
        );

        paint(self, content, &state, depth);

        // TODO: Borders render inside of the bounding box
        // for text which means they can overlap...
        if let Some(color) = resolve_border(style, state.hovered) {
            self.commands[depth].push(Command::RectStroke {
                bounds: paint_bounds,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
                border_thickness: style.border_thickness.unwrap_or(1),
                border_sides: style.border_side.unwrap_or(border::ALL),
            });
        }

        state
    }

    ///paint: (FrameContext, Rect, State, Depth)
    #[inline]
    pub fn widget(
        &mut self,
        content_width: i32,
        content_height: i32,
        layout: &Layout,
        style: &Paint,
        paint: impl FnOnce(&mut Self, Rect, &State, usize),
    ) -> State {
        let role = if style.hover.is_some()
            || style.hover_border.is_some()
            || style.is_selected
            || style.selected.is_some()
            || style.selected_border.is_some()
        {
            Role::BUTTON
        } else {
            Role::NONE
        };
        self.widget_with_role(content_width, content_height, layout, style, role, "", "", paint)
    }

    pub fn image(&mut self, image: Image<'a>, style: ImageStyle) -> State {
        let hint = style.hint.unwrap_or("");
        self.widget_with_role(
            image.width as i32,
            image.height as i32,
            &style.layout,
            &style.paint,
            Role::IMAGE,
            "",
            hint,
            |ui, content, _, depth| {
                let bounds = match style.fit {
                    Fit::Stretch => content,
                    Fit::Contain => {
                        // Scale down to fit, keeping the aspect ratio, then place the
                        // result inside the content box.
                        let (iw, ih) = (image.width as i32, image.height as i32);
                        let (w, h) = if iw * content.height <= ih * content.width {
                            ((iw * content.height) / ih.max(1), content.height)
                        } else {
                            (content.width, (ih * content.width) / iw.max(1))
                        };
                        let alignment = style.content.unwrap_or(Alignment::Center);
                        match align_rect(content, w, h, alignment, Padding::new()) {
                            Some((x, y)) => Rect::new(x, y, w, h),
                            None => return,
                        }
                    }
                };
                let clip = ui.current_frame().clip;
                ui.commands[depth].push(Command::Image {
                    image,
                    bounds,
                    clip,
                    opacity: style.paint.opacity.unwrap_or(255),
                    radius: style.paint.radius.unwrap_or(0),
                });
            },
        )
    }

    pub fn paint_image(&mut self, bounds: Rect, image: Image<'a>, style: ImageStyle) {
        if bounds.is_empty() {
            return;
        }
        let frame = self.current_frame();
        let depth = style.layout.depth.unwrap_or(frame.depth);
        let clip = frame.clip;
        self.commands[depth].push(Command::Image {
            image,
            bounds,
            clip,
            opacity: style.paint.opacity.unwrap_or(255),
            radius: style.paint.radius.unwrap_or(0),
        });
    }

    pub fn gradient(&mut self, style: RectStyle, angle: f32) -> GradientStops<'_, 'frame, 'a> {
        let mut slot = None;
        let state = self.widget(0, 0, &style.layout, &style.paint, |ui, content, _, depth| {
            let clip = ui.current_frame().clip;
            ui.commands[depth].push(Command::Gradient {
                bounds: content,
                clip,
                radius: style.paint.radius.unwrap_or(0),
                gradient: Gradient {
                    stops: [(0.0, 0); MAX_GRADIENT_STOPS],
                    count: 0,
                    angle,
                },
            });
            slot = Some((depth, ui.commands[depth].len() - 1));
        });

        GradientStops { ui: self, slot, state }
    }

    pub fn lines(&mut self, parts: impl IntoIterator<Item = impl Into<Line<'a>>>, style: TextStyle) -> State {
        let parts: Vec<Line<'a>> = parts.into_iter().map(Into::into).collect();
        let default_size = self.default_font_size;

        let mut content_w = 0i32;
        let mut baseline = 0i32;
        let mut run_metrics: Vec<(TextMetrics, Padding, i32)> = Vec::with_capacity(parts.len());
        for part in &parts {
            let font_size = part.style.font_size.unwrap_or(default_size);
            let metrics = if part.content.is_empty() {
                TextMetrics::default()
            } else {
                self.measure_text(
                    &part.content,
                    part.style.font,
                    font_size,
                    part.style.line_height,
                    i32::MAX,
                )
            };
            let run_pad = part.style.layout.padding.unwrap_or_default();
            let face = self.face(part.style.font);
            let line_metrics = self.state.fonts[face]
                .horizontal_line_metrics(font_size as f32)
                .unwrap();
            let line_step = part.style.line_height.map_or(line_metrics.new_line_size, |h| h as f32);
            let above = (line_metrics.ascent + (line_step - line_metrics.ascent + line_metrics.descent) / 2.0).round()
                as i32
                + run_pad.top as i32;
            content_w += metrics.width + run_pad.left as i32 + run_pad.right as i32;
            baseline = baseline.max(above);
            run_metrics.push((metrics, run_pad, above));
        }
        let content_h = run_metrics
            .iter()
            .map(|(metrics, run_pad, above)| {
                baseline - above + run_pad.top as i32 + metrics.height + run_pad.bottom as i32
            })
            .max()
            .unwrap_or(0);

        self.widget(
            content_w,
            content_h,
            &style.layout,
            &style.paint,
            |ui, inner, _, depth| {
                // The whole run is placed as one group, on both axes.
                let alignment = style.content.unwrap_or(Alignment::Center);
                let Some((group_x, group_y)) = align_rect(inner, content_w, content_h, alignment, Padding::new())
                else {
                    return;
                };

                let mut cursor_x = group_x;
                for (part, (metrics, run_pad, above)) in parts.into_iter().zip(run_metrics) {
                    if part.content.is_empty() {
                        cursor_x += run_pad.left as i32 + run_pad.right as i32;
                        continue;
                    }
                    let font_size = part.style.font_size.unwrap_or(default_size);
                    let run_w = metrics.width + run_pad.left as i32 + run_pad.right as i32;
                    let run_y = group_y + baseline - above;

                    ui.paint_text_measured(
                        part.content,
                        metrics,
                        Rect::new(cursor_x, run_y, run_w, (inner.bottom() - run_y).max(0)),
                        part.style.paint.fg.unwrap_or(style.paint.fg.unwrap_or(white())),
                        part.style.font,
                        font_size,
                        part.style.line_height,
                        Alignment::TopLeft,
                        run_pad,
                        part.style.layout.depth.unwrap_or(depth),
                    );

                    cursor_x += run_w;
                }
            },
        )
    }

    #[inline]
    pub fn text(&mut self, text: impl Into<Cow<'a, str>>, style: TextStyle) -> State {
        let text: Cow<'a, str> = text.into();
        let font_size = style.font_size.unwrap_or(self.default_font_size);
        let padding = style.layout.padding.unwrap_or_default();
        let max_width = match (style.wrap, style.layout.width) {
            (true, Some(width)) => (self.resolve_style_size(width, Flow::Right, &style.layout)
                - (padding.left + padding.right) as i32)
                .max(1),
            _ => i32::MAX,
        };
        let metrics = self.measure_text(&text, style.font, font_size, style.line_height, max_width);
        let text_str = text.clone();
        let role = style.role.unwrap_or_else(|| {
            if style.paint.hover.is_some()
                || style.paint.hover_border.is_some()
                || style.paint.is_selected
                || style.paint.selected.is_some()
                || style.paint.selected_border.is_some()
            {
                Role::BUTTON
            } else {
                Role::LABEL
            }
        });
        let hint = style.hint.unwrap_or("");
        self.widget_with_role(
            metrics.width,
            metrics.height,
            &style.layout,
            &style.paint,
            role,
            &text_str,
            hint,
            move |ui, content, _, depth| {
                ui.paint_text_measured(
                    text,
                    metrics,
                    content,
                    style.paint.fg.unwrap_or(white()),
                    style.font,
                    font_size,
                    style.line_height,
                    style.content.unwrap_or(Alignment::Center),
                    Padding::default(),
                    depth,
                );
            },
        )
    }

    pub fn flow<R>(
        &mut self,
        style: impl Into<FlowStyle>,
        flow: Flow,
        advance: bool,
        scroll_y: i32,
        ui: impl FnOnce(&mut Self) -> R,
    ) -> State {
        let parent_frame = self.current_frame();
        let parent_scroll = parent_frame.scroll_y;
        let parent_default_align = parent_frame.align_children;
        let parent_vertical = parent_frame.flow.vertical();

        let style = style.into();
        let layout = style.layout;

        let pb = if layout.bleed { parent_frame.outer_bounds } else { parent_frame.inner_bounds };
        let explicit_x = layout.x.map(|x| self.resolve_size(x, Flow::Right));
        let explicit_y = layout.y.map(|y| self.resolve_size(y, Flow::Down));

        let reverse_x = parent_frame.flow == Flow::Left && explicit_x.is_none();
        let reverse_y = parent_frame.flow == Flow::Up && explicit_y.is_none();

        let margin = layout.margin.unwrap_or_default();

        let anchor_x = explicit_x.unwrap_or(parent_frame.cursor_x);
        let anchor_y = explicit_y.unwrap_or(if parent_scroll != 0 {
            parent_frame.cursor_y - parent_scroll
        } else {
            parent_frame.cursor_y
        });

        let width = layout
            .width
            .map(|w| self.resolve_size_in(pb, w, if reverse_x { Flow::Left } else { Flow::Right }, anchor_x))
            .unwrap_or(if reverse_x { anchor_x - pb.x } else { pb.right() - anchor_x })
            .max(0);

        let height = layout
            .height
            .map(|h| self.resolve_size_in(pb, h, if reverse_y { Flow::Up } else { Flow::Down }, anchor_y))
            .unwrap_or(if reverse_y { anchor_y - pb.y } else { pb.bottom() - anchor_y })
            .max(0);

        let x = if reverse_x { anchor_x - width } else { anchor_x };
        let y = if reverse_y { anchor_y - height } else { anchor_y };

        // Cross-axis alignment, using the same rule as every other widget. A flow can
        // only align itself if it stated its cross size: without one it is sized to fill,
        // and its children are placed before its fitted size is known.
        let align = layout.align.unwrap_or(parent_default_align);
        let x = if parent_vertical && layout.width.is_some() && explicit_x.is_none() {
            align_cross(pb.x, pb.width, width, align)
        } else {
            x
        };
        let y = if !parent_vertical && layout.height.is_some() && explicit_y.is_none() {
            align_cross(pb.y, pb.height, height, align)
        } else {
            y
        };

        let outer_bounds = Rect::new(x, y, width, height);

        if layout.bleed {
            self.layout_stack.last_mut().unwrap().bleed = true;
        }

        let scope = self.next_scope();
        let frame = self.current_frame();
        let depth = layout.depth.unwrap_or(frame.depth);
        let clip = frame.clip;
        let padding = layout.padding.unwrap_or_default();
        let align_children = style.align_children;

        let culled = !layout.skip_cull
            && layout.width.is_some()
            && layout.height.is_some()
            && clip.intersection(outer_bounds).is_empty();

        let paints_bg =
            !culled && (style.paint.bg.is_some() || style.paint.hover.is_some() || style.paint.selected.is_some());
        let bg_index = paints_bg.then(|| {
            self.commands[depth].push(Command::Rect {
                bounds: outer_bounds,
                clip,
                color: style.paint.bg.unwrap_or_default(),
                radius: style.paint.radius.unwrap_or(0),
            });
            self.commands[depth].len() - 1
        });

        let mut inner_bounds = outer_bounds;
        inner_bounds.x += padding.left as i32;
        inner_bounds.width = inner_bounds.width.saturating_sub((padding.left + padding.right) as i32);
        inner_bounds.y += padding.top as i32;
        inner_bounds.height = inner_bounds
            .height
            .saturating_sub((padding.top + padding.bottom) as i32);

        let gap = layout.gap.map(|gap| self.resolve_size(gap, flow)).unwrap_or_default();

        let new_frame = Frame {
            inner_bounds,
            outer_bounds,
            clip: if layout.clip { clip.intersection(outer_bounds) } else { clip },
            flow,
            align_children,
            depth,
            cursor_x: if flow == Flow::Left { inner_bounds.right() } else { inner_bounds.x },
            cursor_y: if flow == Flow::Up { inner_bounds.bottom() } else { inner_bounds.y },
            // Nested flows are already placed in screen space; do not re-apply parent scroll
            // on their children. Only this frame's own scroll_y (e.g. scroll_view) applies.
            scroll_y,
            padding,
            gap,
            outer_width: layout.width.map(|_| width),
            outer_height: layout.height.map(|_| height),
            scope,
            ..Default::default()
        };

        self.layout_stack.push(new_frame);
        if !culled {
            ui(self);
        }

        let (fitted_w, fitted_h) = self.current_frame().fitted_size();
        let fitted_bounds = Rect::new(
            if flow == Flow::Left { outer_bounds.right() - fitted_w } else { x },
            if flow == Flow::Up { outer_bounds.bottom() - fitted_h } else { y },
            fitted_w,
            fitted_h,
        );
        let fitted_bounds = Rect::new(
            fitted_bounds.x - margin.left as i32,
            fitted_bounds.y - margin.top as i32,
            fitted_w + margin.axis(Flow::Right),
            fitted_h + margin.axis(Flow::Down),
        );

        let state = if culled {
            State::new(fitted_bounds)
        } else {
            self.interact(fitted_bounds, depth)
        };

        if let Some(index) = bg_index {
            let bg = resolve_bg(&style.paint, state.hovered);
            if let Command::Rect { bounds, color, .. } = &mut self.commands[depth][index] {
                match bg {
                    Some(bg) => {
                        *bounds = fitted_bounds;
                        *color = bg;
                    }
                    None => *bounds = Rect::default(),
                }
            }
        }

        if !culled && let Some(color) = resolve_border(&style.paint, state.hovered) {
            self.commands[depth].push(Command::RectStroke {
                bounds: fitted_bounds,
                clip,
                color,
                radius: style.paint.radius.unwrap_or(0),
                border_thickness: style.paint.border_thickness.unwrap_or(1),
                border_sides: style.paint.border_side.unwrap_or(border::ALL),
            });
        }

        if advance {
            self.end_layout();
        }

        state
    }

    pub fn flow_down<R>(&mut self, style: impl Into<FlowStyle>, ui: impl FnOnce(&mut Self) -> R) -> State {
        self.flow(style, Flow::Down, true, 0, ui)
    }

    pub fn flow_right<R>(&mut self, style: impl Into<FlowStyle>, ui: impl FnOnce(&mut Self) -> R) -> State {
        self.flow(style, Flow::Right, true, 0, ui)
    }

    pub fn flow_up<R>(&mut self, style: impl Into<FlowStyle>, ui: impl FnOnce(&mut Self) -> R) -> State {
        self.flow(style, Flow::Up, true, 0, ui)
    }

    pub fn flow_left<R>(&mut self, style: impl Into<FlowStyle>, ui: impl FnOnce(&mut Self) -> R) -> State {
        self.flow(style, Flow::Left, true, 0, ui)
    }

    pub fn place_up<R>(&mut self, style: impl Into<FlowStyle>, ui: impl FnOnce(&mut Self) -> R) -> State {
        let state = self.flow(style, Flow::Up, false, 0, ui);
        self.layout_stack.pop().expect("Layout underflow");
        state
    }

    pub fn place_left<R>(&mut self, style: impl Into<FlowStyle>, ui: impl FnOnce(&mut Self) -> R) -> State {
        let state = self.flow(style, Flow::Left, false, 0, ui);
        self.layout_stack.pop().expect("Layout underflow");
        state
    }

    pub fn place_down<R>(&mut self, style: impl Into<FlowStyle>, ui: impl FnOnce(&mut Self) -> R) -> State {
        let state = self.flow(style, Flow::Down, false, 0, ui);
        self.layout_stack.pop().expect("Layout underflow");
        state
    }

    pub fn place_right<R>(&mut self, style: impl Into<FlowStyle>, ui: impl FnOnce(&mut Self) -> R) -> State {
        let state = self.flow(style, Flow::Right, false, 0, ui);
        self.layout_stack.pop().expect("Layout underflow");
        state
    }

    pub fn scroll<R>(
        &mut self,
        style: impl Into<FlowStyle>,
        scroll: &mut Scroll,
        ui: impl FnOnce(&mut Self) -> R,
    ) -> ScrollState {
        self.flow_scroll(style, scroll, ui)
    }

    pub fn flow_scroll<R>(
        &mut self,
        style: impl Into<FlowStyle>,
        scroll: &mut Scroll,
        ui: impl FnOnce(&mut Self) -> R,
    ) -> ScrollState {
        //TODO: It's not explicit to the user that this is always clipped.
        let style = style.into().clip(true);
        let elastic = style.layout.elastic;
        self.flow(
            style,
            Flow::Down,
            false,
            (scroll.offset + scroll.stretch).round() as i32,
            ui,
        );

        let frame = self.end_layout();
        let bounds = frame.inner_bounds;
        let content_height = frame.max_child_height;
        let max_scroll = content_height.saturating_sub(bounds.height).max(0);

        let hovered = self.mouse_position().intersects(bounds);
        let state = ScrollState {
            max_scroll,
            content_height,
            scrolled: hovered && !self.scroll_events.is_empty(),
            direction: match self.scroll_events.last() {
                Some(event) if hovered && event.delta.1 != 0.0 => event.delta.1.signum() as i32,
                _ => 0,
            },
        };

        if bounds.width <= 0 || bounds.height <= 0 {
            return state;
        }

        //TODO: Allow for users to disable middle mouse scrolling.
        let dt = self.dt.min(1.0 / 30.0);
        if hovered && self.window.mouse_pressed(Mouse::Middle) {
            scroll.anchor = Some(self.mouse_position().y);
        }
        let anchored = scroll.anchor.is_some();
        if let Some(direction) = scroll.autoscroll(
            self.mouse_position().y,
            self.window.mouse_down(Mouse::Middle),
            max_scroll as f32,
            dt,
        ) {
            self.animating = true;
            self.window.set_cursor_icon(match direction {
                -1 => CursorIcon::AutoScrollUp,
                1 => CursorIcon::AutoScrollDown,
                _ => CursorIcon::AutoScroll,
            });
        } else if anchored {
            self.window.set_cursor_icon(CursorIcon::Arrow);
        }

        if elastic {
            if scroll.elastic(&self.scroll_events, hovered, max_scroll as f32, dt) {
                self.animating = true;
            }
            return state;
        }

        // Trackpad deltas arrive pixel-accurate and land on every frame, so applying them
        // straight through is already smooth. Only a wheel's notches need scaling up.
        if hovered {
            for event in &self.scroll_events {
                scroll.offset -= event.delta.1 as f32 * if event.precise { 1.0 } else { scroll.wheel_step };
            }
        }
        scroll.offset = scroll.offset.clamp(0.0, max_scroll as f32);
        scroll.stretch = 0.0;

        state
    }

    pub fn current_frame(&self) -> &Frame {
        self.layout_stack.last().as_ref().unwrap()
    }

    pub fn resolve_item_layout(&mut self, width: i32, height: i32, style: &Layout) -> (i32, i32, Rect, Rect) {
        if style.bleed {
            self.layout_stack.last_mut().unwrap().bleed = true;
        }
        let frame = self.layout_stack.last().expect("No active frame");
        let gap = style
            .gap
            .map(|gap| self.resolve_size(gap, frame.flow))
            .unwrap_or(frame.gap);
        let clip = frame.clip;
        let layout = self.walk_layout(width, height, gap, style.align);
        let paint_x = style.x.map_or(layout.paint_x, |x| self.resolve_size(x, Flow::Right));
        let paint_y = style.y.map_or(layout.paint_y, |y| self.resolve_size(y, Flow::Down));

        let margin = style.margin.unwrap_or_default();
        let paint_bounds = Rect::new(
            paint_x - margin.left as i32,
            paint_y - margin.top as i32,
            width + margin.axis(Flow::Right),
            height + margin.axis(Flow::Down),
        );

        let rect = if style.x.is_some() || style.y.is_some() {
            paint_bounds
        } else if style.margin.is_none() || layout.size.is_empty() {
            layout.size
        } else {
            paint_bounds.intersection(clip)
        };

        (paint_x, paint_y, rect, paint_bounds)
    }

    pub fn current_frame_bounds(&self) -> Rect {
        let parent = self.layout_stack.last().expect("Layout stack empty");
        let (x, width) = if parent.flow == Flow::Left {
            (parent.inner_bounds.x, parent.cursor_x - parent.inner_bounds.x)
        } else {
            (parent.cursor_x, parent.inner_bounds.right() - parent.cursor_x)
        };
        let (y, height) = if parent.flow == Flow::Up {
            (parent.inner_bounds.y, parent.cursor_y - parent.inner_bounds.y)
        } else {
            (parent.cursor_y, parent.inner_bounds.bottom() - parent.cursor_y)
        };
        Rect::new(x, y, width.max(0), height.max(0))
    }

    /// Restrict painting to a rectangle without reserving or advancing any layout space.
    pub fn clipped(&mut self, clip: Rect, ui: impl FnOnce(&mut Self)) {
        self.begin_layout(Flow::Down, Some(clip));
        ui(self);
        self.layout_stack.pop().expect("Layout underflow");
    }

    pub fn begin_layout(&mut self, flow: Flow, bounds: Option<Rect>) {
        let bounds = if let Some(bounds) = bounds { bounds } else { self.current_frame_bounds() };

        let scope = self.next_scope();
        let parent = self.layout_stack.last().expect("Layout stack empty");
        let new_frame = Frame {
            inner_bounds: bounds,
            outer_bounds: bounds,
            clip: parent.clip.intersection(bounds),
            flow,
            depth: parent.depth,
            cursor_x: if flow == Flow::Left { bounds.right() } else { bounds.x },
            cursor_y: if flow == Flow::Up { bounds.bottom() } else { bounds.y },
            scope,
            ..Default::default()
        };

        self.layout_stack.push(new_frame);
    }

    pub fn end_layout(&mut self) -> Frame {
        let finished = self.layout_stack.pop().expect("Layout underflow");
        if let Some(parent) = self.layout_stack.last_mut() {
            let (frame_w, frame_h) = finished.fitted_size();

            match parent.flow {
                Flow::Down | Flow::Up => {
                    let step = frame_h + parent.gap;
                    parent.cursor_y += if parent.flow.reverse() { -step } else { step };
                    parent.max_child_width = parent.max_child_width.max(frame_w);
                    parent.max_child_height += step;
                }
                Flow::Right | Flow::Left => {
                    let step = frame_w + parent.gap;
                    parent.cursor_x += if parent.flow.reverse() { -step } else { step };
                    parent.max_child_width += step;
                    parent.max_child_height = parent.max_child_height.max(frame_h);
                }
            }
        }

        finished
    }

    fn draw_frame(&mut self) {
        let window = &mut *self.window;
        let state = &mut *self.state;
        let display_scale = window.scale_factor() as f32;
        let (framebuffer_width, framebuffer_height) = window.scaled_size();

        let dirty = state.render_cache.update(
            &self.commands,
            display_scale,
            framebuffer_width,
            framebuffer_height,
            state.clear_color,
        );

        if state.debug_damage {
            let now = std::time::Instant::now();
            state
                .debug_damage_cache
                .retain(|fade| (now - fade.2).as_secs_f32() < state.debug_damage_fade);
            if dirty {
                for rect in state.render_cache.damage() {
                    let seed = DEBUG_DAMAGE_SEED.fetch_add(SCOPE_SEED, std::sync::atomic::Ordering::Relaxed);
                    let hash = (seed ^ (seed >> 29)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                    state.debug_damage_cache.push((*rect, (hash >> 32) as u32, now));
                }
            }

            let repaint: Vec<Rect> = state
                .debug_damage_cache
                .iter()
                .map(|fade| {
                    fade.0
                        .clamp_to_size(framebuffer_width as i32, framebuffer_height as i32)
                })
                .collect();

            if !repaint.is_empty() {
                let buffer = window.framebuffer();
                clear_damage(buffer, framebuffer_width, &repaint, state.clear_color);
                for prepared in state.render_cache.prepared() {
                    let command = &self.commands[prepared.layer][prepared.index];
                    for region in &repaint {
                        if prepared.bounds.intersects(*region) {
                            draw_command(
                                command,
                                *region,
                                buffer,
                                framebuffer_width,
                                framebuffer_height,
                                display_scale,
                                &state.fonts,
                                &state.fallbacks,
                                &mut state.font_bitmaps,
                                &mut state.image_columns,
                                &state.line_breaks,
                            );
                        }
                    }
                }
                for ((_, tint, start), rect) in state.debug_damage_cache.iter().zip(&repaint) {
                    let strength = 0.4 - (now - *start).as_secs_f32() / state.debug_damage_fade;
                    let alpha = (strength.clamp(0.0, 0.4) * 255.0) as u32;
                    if rect.is_empty() {
                        continue;
                    }
                    for y in rect.y as usize..rect.bottom() as usize {
                        let row = y * framebuffer_width;
                        for pixel in &mut buffer[row + rect.x as usize..row + rect.right() as usize] {
                            let mix = |shift: u32| {
                                (((*pixel >> shift) & 0xFF) * (255 - alpha) + ((tint >> shift) & 0xFF) * alpha) / 255
                            };
                            *pixel = mix(16) << 16 | mix(8) << 8 | mix(0);
                        }
                    }
                }
                window.present_damage(&repaint);
            }
        } else if dirty {
            let buffer = window.framebuffer();
            clear_damage(
                buffer,
                framebuffer_width,
                state.render_cache.damage(),
                state.clear_color,
            );
            raster_damage(
                &self.commands,
                &state.render_cache,
                buffer,
                framebuffer_width,
                framebuffer_height,
                display_scale,
                &state.fonts,
                &state.fallbacks,
                &mut state.font_bitmaps,
                &mut state.image_columns,
                &state.line_breaks,
            );
            window.present_damage(state.render_cache.damage());
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
