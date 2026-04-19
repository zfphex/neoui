use window::*;

pub fn draw_window(window: &mut Window, fill: u32) {
    window.draw();
    window.buffer.fill(fill);
    window.vsync();
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

pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

pub const fn split(color: u32) -> (u8, u8, u8) {
    (
        (color >> 16 & 0xFF) as u8,
        (color >> 8 & 0xFF) as u8,
        (color & 0xFF) as u8,
    )
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
