use window::*;

pub mod helper;
pub mod style;
pub use helper::*;
pub use style::*;

pub enum Command<'a> {
    Rect {
        rect: Rect,
        color: u32,
    },
    Text {
        text: &'a str,
        pos: (usize, usize),
        color: u32,
    },
}

pub const FONT: &[u8] = include_bytes!("../fonts/Aptos.ttf");

pub struct Context<'a> {
    pub hovered_id: Option<u64>,
    pub active_id: Option<u64>,
    pub commands: Vec<Command<'a>>,
    pub font: Option<fontdue::Font>,
    pub window: Option<std::pin::Pin<Box<Window>>>,
}

pub static mut CTX: Context<'static> = Context {
    hovered_id: None,
    active_id: None,
    commands: Vec::new(),
    font: None,
    window: None,
};

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

// pub fn ctx_handle_event<'a>(ctx: &mut Context<'a>, window: &mut Window) {}

pub fn draw_cmd<'a>() {
    let ctx = unsafe { &mut *(&raw mut CTX) };
    let window = ctx.window.as_mut().unwrap();

    for cmd in core::mem::take(&mut ctx.commands) {
        match cmd {
            Command::Rect { rect, color } => draw_rect(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                window.width(),
                window.height(),
                &mut window.buffer,
                color,
            ),
            Command::Text {
                text,
                pos: (x, y),
                color,
            } => {
                draw_text(
                    text,
                    ctx.font.as_ref().unwrap(),
                    x,
                    y,
                    32,
                    window.display_scale(),
                    window.width(),
                    &mut window.buffer,
                    color,
                    false,
                );
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
    //The rectangle is malformed and out of bounds.
    if x > window_width || y > window_height {
        return;
    }

    //Do not allow rectangles to be larger than the viewport
    //the user should not crash for this.
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

fn get_id(ctx: &Context, label: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    label.hash(&mut hasher);
    hasher.finish()
}

pub fn button<'a: 'b, 'b>(label: &'a str, style: Style) -> bool {
    let ctx = unsafe { &mut *(&raw mut CTX) };
    let ctx = unsafe { core::mem::transmute::<&mut Context<'static>, &mut Context<'b>>(ctx) };

    let id = get_id(ctx, label);
    let window = ctx.window.as_mut().unwrap();

    //TODO: This can easily be cached.
    let rect = draw_text(
        label,
        ctx.font.as_ref().unwrap(),
        0,
        0,
        32,
        1.0,
        window.width(),
        &mut [],
        white(),
        true,
    );

    let hovered = window.mouse_position.intersects(rect);

    if hovered {
        ctx.hovered_id = Some(id);
        if window.left_mouse.pressed {
            ctx.active_id = Some(id);
        }
    }

    let mut clicked = false;
    if window.left_mouse.clicked(rect) {
        clicked = true;
    }

    ctx.commands.push(Command::Rect {
        rect,
        color: style.bg.unwrap_or_default(),
    });

    ctx.commands.push(Command::Text {
        text: label,
        pos: (rect.x, rect.y),
        color: style.fg.unwrap_or(white()),
    });

    clicked
}

pub fn draw_window(window: &mut Window, fill: u32) {
    window.draw();
    window.buffer.fill(fill);
    window.vsync();
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
    // window: Rect,
    window_width: usize,
    buffer: &mut [u32],
    color: u32,
    skip_draw: bool,
) -> Rect {
    if text.is_empty() || font_size == 0 {
        return Rect::new(0, 0, 0, 0);
    }

    let x = scale(x, display_scale);
    let y = scale(y, display_scale);
    let font_size = scale(font_size, display_scale);

    let mut area = Rect::new(x, y, 0, 0);
    let mut y = area.y;
    let x = area.x;

    let mut max_x = 0;
    let mut max_y = 0;

    let (r1, g1, b1) = split(color);

    'line: for line in text.lines() {
        let mut glyph_x = x;

        for char in line.chars() {
            let (metrics, bitmap) = font.rasterize(char, font_size as f32);

            let glyph_y =
                y as f32 - (metrics.height as f32 - metrics.advance_height) - metrics.ymin as f32;

            for y in 0..metrics.height {
                'x: for x in 0..metrics.width {
                    //Text doesn't fit on the screen.
                    if (x + glyph_x) >= window_width {
                        continue;
                    }

                    //TODO: Metrics.bounds determines the bounding are of the glyph.
                    //Currently the whole bitmap bounding box is drawn.
                    let alpha = bitmap[x + y * metrics.width];
                    if alpha == 0 {
                        continue;
                    }

                    //Should the text really be offset by the font size?
                    //This allows the user to draw text at (0, 0).
                    let offset = font_size as f32 + glyph_y + y as f32;

                    //We can't render off of the screen, mkay?
                    if offset < 0.0 {
                        continue;
                    }

                    if max_x < x + glyph_x {
                        max_x = x + glyph_x;
                    }

                    if max_y < offset as usize {
                        max_y = offset as usize;
                    }

                    if skip_draw {
                        continue;
                    }

                    let i = x + glyph_x + window_width * offset as usize;

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

            glyph_x += metrics.advance_width as usize;

            //Check if the glyph position is off the screen.
            if glyph_x >= window_width {
                break 'line;
            }
        }

        y += font_size;
    }

    //Not sure why these are one off.
    area.height = max_y + 1 - area.y;
    area.width = max_x + 1 - area.x;
    area
}
