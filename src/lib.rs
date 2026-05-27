//TODO: Fix this issue in mini
#![allow(unexpected_cfgs)]

pub mod style;
pub use style::*;

pub mod platform;
pub use platform::*;

pub mod shapes;
pub use shapes::*;

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
        radius: usize,
        outline_thickness: usize,
    },
    Triangle {
        a: (usize, usize),
        b: (usize, usize),
        c: (usize, usize),
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

pub fn ctx(title: &str, width: usize, height: usize) -> Context {
    #[cfg(target_os = "windows")]
    let window = create_window(title, 0, 0, width as i32, height as i32, WindowStyle::DEFAULT);

    #[cfg(target_os = "macos")]
    let window = Box::pin(Window::new(title, width, height));

    Context {
        commands: Vec::new(),
        font: Some(fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()),
        window: Some(window),
        layout_stack: Vec::new(),
        glyph_cache: None,
        glyph_cache_subpixel: None,
        default_font_size: 32,
        input_blockers: Vec::new(),
        overlay: false,
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
    pub input_blockers: Vec<Rect>,
    pub overlay: bool,
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
        if !self.can_hit() {
            return false;
        }
        let window = self.window.as_mut().unwrap();
        window.left_mouse.clicked(rect)
    }

    pub fn hovered(&self, rect: Rect) -> bool {
        if !self.can_hit() {
            return false;
        }
        let window = self.window.as_ref().unwrap();
        window.mouse_position.intersects(rect)
    }

    pub fn block_input(&mut self, rect: Rect) {
        self.input_blockers.push(rect);
    }

    pub fn can_hit(&self) -> bool {
        if self.overlay {
            return true;
        }
        let Some(blocker) = self.input_blockers.last() else {
            return true;
        };
        let mouse = self.window.as_ref().unwrap().mouse_position;
        !mouse.intersects(*blocker)
    }

    //TODO: This should really be paint_rect or something.
    //Users should be able to use the layout system to render rectangles.
    pub fn rect(&mut self, rect: Rect, color: u32) {
        self.commands.push(Command::Rect {
            rect,
            color,
            radius: 0,
            outline_thickness: 0,
        });
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
        let clicked = self.clicked(rect);
        let hovered = self.hovered(rect);

        let bg = if hovered && style.hover.is_some() {
            style.hover
        } else {
            style.bg
        };

        if let Some(color) = bg {
            self.commands.push(Command::Rect {
                rect,
                color,
                radius: style.radius.unwrap_or(0),
                outline_thickness: style.outline_thickness.unwrap_or(0),
            });
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
        let clicked = self.clicked(rect);
        let hovered = self.hovered(rect);

        let bg = if selected {
            style.selected
        } else if hovered {
            style.hover
        } else {
            style.bg
        };

        if let Some(color) = bg {
            self.commands.push(Command::Rect {
                rect,
                color,
                radius: style.radius.unwrap_or(0),
                outline_thickness: style.outline_thickness.unwrap_or(0),
            });
        }

        if selected {
            if let Some(color) = style.selected_border {
                self.commands.push(Command::Rect {
                    rect,
                    color,
                    radius: style.radius.unwrap_or(0),
                    outline_thickness: style.outline_thickness.unwrap_or(1),
                });
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
            self.commands.push(Command::Rect {
                rect,
                color,
                radius: style.radius.unwrap_or(0),
                outline_thickness: style.outline_thickness.unwrap_or(0),
            });
        }

        if let Some(color) = style.border {
            self.commands.push(Command::Rect {
                rect,
                color,
                radius: style.radius.unwrap_or(0),
                outline_thickness: style.outline_thickness.unwrap_or(1),
            });
        }
    }

    /// Splits the current frame's remaining space horizontally.
    pub fn split_h(&self, left_width: usize) -> (Rect, Rect) {
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
    pub fn split_v(&self, top_height: usize) -> (Rect, Rect) {
        let frame = self.layout_stack.last().expect("No active frame");

        let total_w = (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x);
        let total_h = (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y);

        let top_h = top_height.min(total_h);
        let bottom_h = total_h.saturating_sub(top_h);

        let top_rect = Rect::new(frame.cursor_x, frame.cursor_y, total_w, top_h);
        let bottom_rect = Rect::new(frame.cursor_x, frame.cursor_y + top_h, total_w, bottom_h);

        (top_rect, bottom_rect)
    }

    pub fn flow_down<R>(&mut self, bounds: Rect, ui: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_layout_with_bounds(Flow::Down, bounds);
        let result = ui(self);
        self.end_layout();
        result
    }

    pub fn flow_right<R>(&mut self, bounds: Rect, ui: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_layout_with_bounds(Flow::Right, bounds);
        let result = ui(self);
        self.end_layout();
        result
    }

    pub fn begin_ui(&mut self, fill_color: u32) {
        let bounds = Rect::new(0, 0, self.width(), self.height());
        self.fill(fill_color);
        self.layout_stack.clear();
        self.input_blockers.clear();
        self.overlay = false;
        self.layout_stack.push(Frame {
            bounds,
            flow: Flow::Down,
            cursor_x: 0,
            cursor_y: 0,
            max_child_width: 0,
            max_child_height: 0,
        });
    }

    pub fn begin_layout_with_bounds(&mut self, flow: Flow, bounds: Rect) {
        let new_frame = Frame {
            bounds: bounds,
            flow,
            cursor_x: bounds.x,
            cursor_y: bounds.y,
            max_child_width: 0,
            max_child_height: 0,
        };
        self.layout_stack.push(new_frame);
    }

    pub fn begin_layout(&mut self, flow: Flow) {
        let parent = self.layout_stack.last().expect("Layout stack empty");
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

    pub fn begin_overlay(&mut self, flow: Flow, explicit_bounds: Rect) {
        self.overlay = true;
        self.begin_layout_with_bounds(flow, explicit_bounds);
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

        self.begin_layout_with_bounds(flow, Rect::new(cell_x, cell_y, cell_w, cell_h));
    }

    pub fn end_overlay(&mut self) {
        self.overlay = false;
        self.end_layout_absolute();
    }

    pub fn fill(&mut self, color: u32) {
        let window = self.window.as_mut().unwrap();
        window.buffer.fill(color);
    }

    pub fn draw(&mut self) {
        let self_width = self.width();
        let self_height = self.height();
        let window = self.window.as_mut().unwrap();

        for cmd in core::mem::take(&mut self.commands) {
            match cmd {
                Command::Rect {
                    rect,
                    color,
                    radius,
                    outline_thickness,
                } => {
                    if outline_thickness == 0 {
                        draw_rounded_rect(
                            &mut window.buffer,
                            rect.x,
                            rect.y,
                            rect.width,
                            rect.height,
                            self_width,
                            self_height,
                            radius,
                            color,
                        )
                    } else {
                        draw_rounded_rect_outline(
                            &mut window.buffer,
                            rect.x,
                            rect.y,
                            rect.width,
                            rect.height,
                            self_width,
                            self_height,
                            radius,
                            outline_thickness,
                            color,
                        )
                    }
                }
                Command::Text {
                    text,
                    x,
                    y,
                    color,
                    size,
                } => {
                    let cache_map = self.glyph_cache_subpixel.get_or_insert_with(HashMap::new);
                    draw_text_subpixel(
                        &text,
                        self.font.as_ref().unwrap(),
                        x,
                        y,
                        size,
                        window.display_scale(),
                        self_width,
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
                    draw_triangle_sdf(
                        &mut window.buffer,
                        self_width,
                        self_height,
                        ax,
                        ay,
                        bx,
                        by,
                        cx,
                        cy,
                        color,
                    )
                }
            };
        }

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

    pub fn exit(&mut self) -> bool {
        let window = self.window.as_mut().unwrap();

        if let Some(event) = window.event() {
            return match event {
                Event::Quit | Event::Input(Key::Escape, _) => true,
                _ => false,
            };
        }
        false
    }
}
