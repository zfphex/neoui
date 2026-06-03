//TODO: Fix this issue in mini
#![allow(unexpected_cfgs)]

pub mod style;
pub use style::*;

pub mod platform;
pub use platform::*;

pub mod shapes;
pub use shapes::*;

pub use mini::*;

use std::borrow::Cow;
use std::collections::HashMap;

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

#[derive(Debug)]
pub enum Command<'a> {
    Rect {
        rect: Rect,
        clip: Rect,
        color: u32,
        radius: usize,
        outline_thickness: usize,
    },
    Triangle {
        a: (usize, usize),
        b: (usize, usize),
        c: (usize, usize),
        clip: Rect,
        color: u32,
    },
    Text {
        text: Cow<'a, str>,
        clip: Rect,
        x: usize,
        y: usize,
        color: u32,
        size: usize,
    },
}

#[derive(Debug, Clone)]
pub struct State {
    pub clicked: bool,
    pub hovered: bool,
    pub rect: Rect,
}

pub const FONT: &[u8] = include_bytes!("../fonts/Aptos.ttf");

pub fn ui<'a>(title: &str, width: usize, height: usize) -> Context<'a> {
    #[cfg(target_os = "windows")]
    let window = create_window(title, 0, 0, width as i32, height as i32, WindowStyle::DEFAULT);

    #[cfg(target_os = "macos")]
    let window = Box::pin(Window::new(title, width, height));

    Context {
        commands: [const { Vec::new() }; 16],
        font: Some(fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()),
        window: Some(window),
        layout_stack: Vec::new(),
        glyph_cache: None,
        glyph_cache_subpixel: None,
        default_font_size: 32,
    }
}

pub struct Context<'a> {
    pub commands: [Vec<Command<'a>>; 16],
    pub font: Option<fontdue::Font>,
    pub window: Option<std::pin::Pin<Box<Window>>>,
    pub layout_stack: Vec<Frame>,
    pub glyph_cache: Option<HashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    pub glyph_cache_subpixel: Option<HashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    pub default_font_size: usize,
}

impl<'a> Context<'a> {
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

        if frame.scroll_y == 0 {
            return rect;
        }

        // Calculate the scroll position.

        //  0 --------+
        //            |
        //            | <-- `frame.scroll_y`
        //            |
        //  0 ========|========= <--  Screen top
        //            |        |
        //            |        | <-- `y` = prev_y - scroll_y
        //            |        |     (Distance from screen top)
        //  prev_y -> +------+ |
        //            | ITEM | |
        //            +------+ |
        //                     |
        //            VIEWPORT |
        //  ====================
        let y = rect.y as i32 - frame.scroll_y as i32;

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

        if y + height as i32 <= frame.bounds.y as i32 {
            return Rect::new(0, 0, 0, 0);
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

        if y >= (frame.bounds.y + frame.bounds.height) as i32 {
            return Rect::new(0, 0, 0, 0);
        }

        Rect::new(rect.x, y as usize, width, height)
    }

    /// Splits the current frame's remaining space horizontally.
    pub fn split_h(&self, left_width: impl Into<Size>) -> (Rect, Rect) {
        let left_width = self.resolve_size(left_width.into(), true);
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
        let top_height = self.resolve_size(top_height.into().unwrap_or_default(), false);
        let frame = self.layout_stack.last().expect("No active frame");

        let total_w = (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x);
        let total_h = (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y);

        let top_h = top_height.min(total_h);
        let bottom_h = total_h.saturating_sub(top_h);

        let top_rect = Rect::new(frame.cursor_x, frame.cursor_y, total_w, top_h);
        let bottom_rect = Rect::new(frame.cursor_x, frame.cursor_y + top_h, total_w, bottom_h);

        (top_rect, bottom_rect)
    }

    pub fn resolve_size(&self, size: Size, horizontal: bool) -> usize {
        let frame = self.layout_stack.last().expect("No active frame");
        match size {
            Size::Pixel(px) => px,
            Size::Percentage(pct) => {
                let total = if horizontal {
                    frame.bounds.width
                } else {
                    frame.bounds.height
                };
                (total as f32 * pct) as usize
            }
            Size::Fill => {
                if horizontal {
                    (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x)
                } else {
                    (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y)
                }
            }
            Size::FillMinus(sub) => {
                let remaining = if horizontal {
                    (frame.bounds.x + frame.bounds.width).saturating_sub(frame.cursor_x)
                } else {
                    (frame.bounds.y + frame.bounds.height).saturating_sub(frame.cursor_y)
                };
                remaining.saturating_sub(sub.abs() as usize)
            }
            Size::Fit => todo!(),
        }
    }

    pub fn clicked(&mut self, rect: Rect) -> bool {
        let frame = self.layout_stack.last().expect("No active frame");
        let window = self.window.as_mut().unwrap();
        window.left_mouse.clicked(rect) && window.mouse_position.intersects(frame.bounds)
    }

    pub fn hovered(&self, rect: Rect) -> bool {
        let frame = self.layout_stack.last().expect("No active frame");
        let window = self.window.as_ref().unwrap();
        window.mouse_position.intersects(rect) && window.mouse_position.intersects(frame.bounds)
    }

    //TODO: This should really be paint_rect or something.
    //Users should be able to use the layout system to render rectangles.
    pub fn paint_rect(&mut self, rect: Rect, style: Style) {
        let clip = self.layout_stack.last().expect("No active frame").clip;
        let depth = style.depth.unwrap_or(0);
        if let Some(color) = style.bg {
            self.commands[depth].push(Command::Rect {
                rect,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
                outline_thickness: style.border_thickness.unwrap_or(0),
            });
        }
    }

    pub fn paint_triangle(&mut self, a: (usize, usize), b: (usize, usize), c: (usize, usize), style: Style) {
        let clip = self.layout_stack.last().expect("No active frame").clip;
        let depth = style.depth.unwrap_or(0);
        if let Some(color) = style.bg {
            self.commands[depth].push(Command::Triangle { a, b, c, clip, color });
        }
    }

    pub fn measure_text(&mut self, text: &str, font_size: usize) -> Rect {
        let width = self.width();
        let font = self.font.as_ref().expect("Font missing");
        let cache = self.glyph_cache_subpixel.get_or_insert_with(HashMap::new);

        draw_text_subpixel(
            text,
            font,
            0,
            0,
            font_size,
            1.0,
            width,
            &mut [],
            0,
            true,
            cache,
            Rect::new(0, 0, usize::MAX, usize::MAX),
        )
    }

    #[doc(hidden)]
    pub fn text_aligned(
        &mut self,
        dest: Rect,
        text: impl Into<Cow<'a, str>>,
        color: u32,
        font_size: usize,
        alignment: Alignment,
        depth: usize,
    ) {
        if dest.width == 0 || dest.height == 0 {
            return;
        }

        let text = text.into();
        let text_metrics = self.measure_text(&text, font_size);
        let rect = align_rect(dest, text_metrics.width, text_metrics.height, alignment);
        let clip = self.layout_stack.last().expect("No active frame").clip;
        self.commands[depth].push(Command::Text {
            text,
            clip,
            x: rect.x,
            y: rect.y,
            color,
            size: font_size,
        });
    }

    pub fn rect(&mut self, style: Style) -> State {
        self.item("", false, style)
    }

    pub fn text(&mut self, text: impl Into<Cow<'a, str>>, style: Style) -> State {
        self.item(text, false, style)
    }

    pub fn list_item(&mut self, text: impl Into<Cow<'a, str>>, selected: bool, style: Style) -> State {
        //By default use align on the left instead of the center.
        let style = if let Some(pad) = style.padding
            && style.alignment.is_none()
        {
            // TODO: I think there is a clash where we have two padding values for each item?
            // There is alignment padding and regualr padding?
            style.align(Alignment::Left { pad: pad.left })
        } else {
            style
        };
        self.item(text, selected, style)
    }

    pub fn item(&mut self, text: impl Into<Cow<'a, str>>, selected: bool, style: Style) -> State {
        let text = text.into();
        let font_size = style.font_size.unwrap_or(self.default_font_size);
        let text_metrics = if text.is_empty() {
            Rect::default()
        } else {
            self.measure_text(&text, font_size)
        };

        let padding = style.padding.unwrap_or_default();
        let width = style
            .width
            .map(|w| self.resolve_size(w, true))
            .unwrap_or(text_metrics.width + padding.left + padding.right);
        let height = style
            .height
            .map(|h| self.resolve_size(h, false))
            .unwrap_or(text_metrics.height + padding.top + padding.bottom);

        let rect = self.walk_layout(width, height);

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
                rect,
                clip,
                color,
                radius: style.radius.unwrap_or(0),
                outline_thickness: style.border_thickness.unwrap_or(0),
            });
        }

        let border = if selected && style.selected_border.is_some() {
            style.selected_border
        } else if style.border.is_some() {
            style.border
        } else {
            None
        };

        if let Some(border) = border {
            self.commands[depth].push(Command::Rect {
                rect,
                clip,
                color: border,
                radius: style.radius.unwrap_or(0),
                outline_thickness: style.border_thickness.unwrap_or(1),
            });
        }

        if !text.is_empty() {
            self.text_aligned(
                rect,
                text,
                style.fg.unwrap_or(white()),
                font_size,
                style.alignment.unwrap_or(Alignment::Center),
                depth,
            );
        }

        State { clicked, hovered, rect }
    }

    pub fn flow_down<R>(&mut self, bounds: Rect, ui: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_layout(Flow::Down, Some(bounds));
        let result = ui(self);
        self.end_layout();
        result
    }

    pub fn flow_right<R>(&mut self, bounds: Rect, ui: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_layout(Flow::Right, Some(bounds));
        let result = ui(self);
        self.end_layout();
        result
    }

    //Currently no horizontal scroll support.
    pub fn scroll<R>(&mut self, bounds: Option<Rect>, scroll_y: usize, ui: impl FnOnce(&mut Self) -> R) -> usize {
        let parent = self.layout_stack.last().expect("Layout stack empty");
        let bounds = if let Some(bounds) = bounds {
            bounds
        } else {
            Rect::new(
                parent.cursor_x,
                parent.cursor_y,
                parent.bounds.width,
                parent.bounds.height,
            )
        };

        self.begin_scroll_view(bounds, scroll_y);

        let _ = ui(self); //Probably fine.

        self.end_scroll_view()
    }

    pub fn begin_frame(&mut self, fill_color: u32) {
        let bounds = Rect::new(0, 0, self.width(), self.height());
        self.fill(fill_color);
        self.layout_stack.clear();
        self.layout_stack.push(Frame {
            bounds,
            clip: bounds,
            flow: Flow::Down,
            ..Default::default()
        });
    }

    pub fn begin_layout(&mut self, flow: Flow, bounds: Option<Rect>) {
        let parent = self.layout_stack.last().expect("Layout stack empty");

        let bounds = if let Some(bounds) = bounds {
            bounds
        } else {
            Rect::new(
                parent.cursor_x,
                parent.cursor_y,
                parent.bounds.width,
                parent.bounds.height,
            )
        };

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

    pub fn begin_scroll_view(&mut self, bounds: Rect, scroll_y: usize) {
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
    }

    pub fn end_scroll_view(&mut self) -> usize {
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

    pub fn fill(&mut self, color: u32) {
        let window = self.window.as_mut().unwrap();
        window.buffer.fill(color);
    }

    pub fn draw_frame(&mut self) {
        profile!();
        let self_width = self.width();
        let self_height = self.height();
        let window = self.window.as_mut().unwrap();

        for layer in &mut self.commands {
            for cmd in layer.drain(..) {
                match cmd {
                    Command::Rect {
                        rect,
                        clip,
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
                                clip,
                            )
                        } else {
                            //rounded rect outline doesn't work for 1px outlines??
                            draw_rect_outline(
                                &mut window.buffer,
                                rect.x,
                                rect.y,
                                rect.width,
                                rect.height,
                                self_width,
                                color,
                                clip,
                            )
                        }
                    }
                    Command::Text {
                        text,
                        clip,
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
                            clip,
                        );
                    }
                    Command::Triangle {
                        a: (ax, ay),
                        b: (bx, by),
                        c: (cx, cy),
                        clip,
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
                            clip,
                        )
                    }
                };
            }
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
