use crate::*;

pub fn draw_rect(
    buffer: &mut [u32],
    x: usize,
    y: usize,
    mut width: usize,
    mut height: usize,
    window_width: usize,
    window_height: usize,
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
    buffer: &mut [u32],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    window_width: usize,
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

pub fn draw_rounded_rect(
    buffer: &mut [u32],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    window_width: usize,
    window_height: usize,
    radius: usize,
    color: u32,
) {
    if width == 0 || height == 0 {
        return;
    }

    let radius = radius.min(width / 2).min(height / 2);
    let min_y = y.min(window_height);
    let max_y = (y + height).min(window_height);
    let min_x = x.min(window_width);
    let max_x = (x + width).min(window_width);

    if radius == 0 {
        for py in min_y..max_y {
            let start = py * window_width + min_x;
            let end = py * window_width + max_x;
            if let Some(slice) = buffer.get_mut(start..end) {
                slice.fill(color);
            }
        }
        return;
    }

    let src_r = ((color >> 16 & 0xFF) as f32 / 255.0).powi(2);
    let src_g = ((color >> 8 & 0xFF) as f32 / 255.0).powi(2);
    let src_b = ((color & 0xFF) as f32 / 255.0).powi(2);

    let cx = x as f32 + width as f32 / 2.0;
    let cy = y as f32 + height as f32 / 2.0;
    let half_w = width as f32 / 2.0;
    let half_h = height as f32 / 2.0;
    let r_f32 = radius as f32;

    for py in min_y..max_y {
        let dy = (py as f32 + 0.5 - cy).abs() - half_h + r_f32;
        let dy_max = dy.max(0.0);
        let dy_sq = dy_max * dy_max;

        let mut solid_start = window_width;
        let mut solid_end = 0;

        for px in min_x..max_x {
            let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;

            let dx_max = dx.max(0.0);
            let dist_outer = (dx_max * dx_max + dy_sq).sqrt();
            let dist_inner = dx.max(dy).min(0.0);
            let dist = dist_outer + dist_inner - r_f32;
            let alpha = 0.5 - dist;

            if alpha >= 0.999 {
                if solid_start == window_width {
                    solid_start = px;
                }
                solid_end = px + 1;
            } else if alpha > 0.0 {
                let idx = py * window_width + px;
                if let Some(bg) = buffer.get_mut(idx) {
                    let a = alpha.clamp(0.0, 1.0);

                    let bg_r = (((*bg >> 16) & 0xFF) as f32 / 255.0).powi(2);
                    let bg_g = (((*bg >> 8) & 0xFF) as f32 / 255.0).powi(2);
                    let bg_b = ((*bg & 0xFF) as f32 / 255.0).powi(2);

                    let out_r = (src_r * a) + (bg_r * (1.0 - a));
                    let out_g = (src_g * a) + (bg_g * (1.0 - a));
                    let out_b = (src_b * a) + (bg_b * (1.0 - a));

                    *bg = ((out_r.sqrt() * 255.0) as u32) << 16
                        | ((out_g.sqrt() * 255.0) as u32) << 8
                        | ((out_b.sqrt() * 255.0) as u32);
                }
            }
        }

        if solid_start < solid_end {
            let start_idx = py * window_width + solid_start;
            let end_idx = py * window_width + solid_end;
            if let Some(slice) = buffer.get_mut(start_idx..end_idx) {
                slice.fill(color);
            }
        }
    }
}

pub fn draw_rounded_rect_outline(
    buffer: &mut [u32],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    window_width: usize,
    window_height: usize,
    radius: usize,
    thickness: usize,
    color: u32,
) {
    if width == 0 || height == 0 || thickness == 0 {
        return;
    }

    if radius == 0 {
        draw_rect_outline(buffer, x, y, width, height, window_width, color);
    }

    let radius = radius.min(width / 2).min(height / 2);
    let min_y = y.min(window_height);
    let max_y = (y + height).min(window_height);
    let min_x = x.min(window_width);
    let max_x = (x + width).min(window_width);

    let t_f32 = thickness as f32;
    let src_r = ((color >> 16 & 0xFF) as f32 / 255.0).powi(2);
    let src_g = ((color >> 8 & 0xFF) as f32 / 255.0).powi(2);
    let src_b = ((color & 0xFF) as f32 / 255.0).powi(2);

    let cx = x as f32 + width as f32 / 2.0;
    let cy = y as f32 + height as f32 / 2.0;
    let half_w = width as f32 / 2.0;
    let half_h = height as f32 / 2.0;
    let r_f32 = radius as f32;

    for py in min_y..max_y {
        let dy = (py as f32 + 0.5 - cy).abs() - half_h + r_f32;
        let dy_max = dy.max(0.0);
        let dy_sq = dy_max * dy_max;

        for px in min_x..max_x {
            let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;

            let dx_max = dx.max(0.0);
            let dist_outer = (dx_max * dx_max + dy_sq).sqrt();
            let dist_inner = dx.max(dy).min(0.0);

            let dist = dist_outer + dist_inner - r_f32;

            let final_dist = dist.max(-(dist + t_f32));

            let alpha = 0.5 - final_dist;

            if alpha >= 0.999 {
                let idx = py * window_width + px;
                if let Some(bg) = buffer.get_mut(idx) {
                    *bg = color;
                }
            } else if alpha > 0.0 {
                let idx = py * window_width + px;
                if let Some(bg) = buffer.get_mut(idx) {
                    let a = alpha.clamp(0.0, 1.0);

                    let bg_r = (((*bg >> 16) & 0xFF) as f32 / 255.0).powi(2);
                    let bg_g = (((*bg >> 8) & 0xFF) as f32 / 255.0).powi(2);
                    let bg_b = ((*bg & 0xFF) as f32 / 255.0).powi(2);

                    let out_r = (src_r * a) + (bg_r * (1.0 - a));
                    let out_g = (src_g * a) + (bg_g * (1.0 - a));
                    let out_b = (src_b * a) + (bg_b * (1.0 - a));

                    *bg = ((out_r.sqrt() * 255.0) as u32) << 16
                        | ((out_g.sqrt() * 255.0) as u32) << 8
                        | ((out_b.sqrt() * 255.0) as u32);
                }
            }
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
    let line_metrics = font.horizontal_line_metrics(font_size as f32).unwrap();

    let mut area = Rect::new(x, y, 0, 0);
    let mut y_pos = area.y;

    let mut max_glyph_x = x as f32;

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

                    let offset = line_metrics.ascent + glyph_y + y_px as f32;

                    if offset < 0.0 {
                        continue;
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

        if glyph_x > max_glyph_x {
            max_glyph_x = glyph_x;
        }

        y_pos += line_metrics.new_line_size.round() as usize;
    }

    area.height = y_pos - area.y;
    area.width = (max_glyph_x - x as f32).round() as usize;
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

    let line_metrics = font.horizontal_line_metrics(font_size as f32).unwrap();

    let mut area = Rect::new(x_start, y_start, 0, 0);
    let mut y_pos = area.y;

    let mut max_glyph_x = x_start as f32;

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
                let offset = line_metrics.ascent + glyph_y + y_px as f32;

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

        if glyph_x > max_glyph_x {
            max_glyph_x = glyph_x;
        }

        y_pos += line_metrics.new_line_size.round() as usize;
    }

    area.height = y_pos - area.y;
    area.width = (max_glyph_x - x_start as f32).round() as usize;

    area
}
