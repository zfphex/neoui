use crate::*;

pub const fn scale(value: usize, scale: f32) -> usize {
    (value as f32 * scale).round() as usize
}

pub const fn blend(color: u8, alpha: u8, bg_color: u8, bg_alpha: u8) -> u8 {
    ((color as f32 * alpha as f32 + bg_color as f32 * bg_alpha as f32) / 255.0).round() as u8
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
    let mid_x = parent.x + (parent.width / 2) - (child_w / 2);
    let mid_y = parent.y + (parent.height / 2) - (child_h / 2);
    let right_edge = parent.x + parent.width - child_w;
    let bottom_edge = parent.y + parent.height - child_h;

    // hmmm
    // let mid_x = (parent.x as i32 + (parent.width as i32 / 2) - (child_w as i32 / 2)) as usize;
    // let mid_y = (parent.y as i32 + (parent.height as i32 / 2) - (child_h as i32 / 2)) as usize;
    // let right_edge = (parent.x as i32 + parent.width as i32 - child_w as i32) as usize;
    // let bottom_edge = (parent.y as i32 + parent.height as i32 - child_h as i32) as usize;

    let (x, y) = match alignment {
        Alignment::Left { pad } => (parent.x + pad, mid_y),
        Alignment::Center => (mid_x, mid_y),
        Alignment::Right { pad } => (right_edge.saturating_sub(pad), mid_y),
        Alignment::TopLeft { padh, padv } => (parent.x + padh, parent.y + padv),
        Alignment::TopCenter { pad } => (mid_x, parent.y + pad),
        Alignment::TopRight { padh, padv } => (right_edge.saturating_sub(padh), parent.y + padv),
        Alignment::BottomLeft { padh, padv } => (parent.x + padh, bottom_edge.saturating_sub(padv)),
        Alignment::BottomCenter { pad } => (mid_x, bottom_edge.saturating_sub(pad)),
        Alignment::BottomRight { padh, padv } => (right_edge.saturating_sub(padh), bottom_edge.saturating_sub(padv)),
    };

    Rect::new(x, y, child_w, child_h)
}

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
    clip: Rect,
    sides: u8,
) {
    use border::*;

    if height == 0 || width == 0 {
        return;
    }

    let min_y = y.max(clip.y);
    let max_y = (y + height).min(clip.y + clip.height);
    let min_x = x.max(clip.x);
    let max_x = (x + width).min(clip.x + clip.width);

    for py in min_y..max_y {
        if py == y {
            if sides & TOP != 0 && min_x < max_x {
                let row_start = py * window_width + min_x;
                let row_end = py * window_width + max_x;
                if let Some(slice) = buffer.get_mut(row_start..row_end) {
                    slice.fill(color);
                }
            }
        } else if py == y + height.saturating_sub(1) {
            if sides & BOTTOM != 0 && min_x < max_x {
                let row_start = py * window_width + min_x;
                let row_end = py * window_width + max_x;
                if let Some(slice) = buffer.get_mut(row_start..row_end) {
                    slice.fill(color);
                }
            }
        } else {
            if sides & LEFT != 0 && x >= min_x && x < max_x {
                if let Some(b) = buffer.get_mut(py * window_width + x) {
                    *b = color;
                }
            }
            let right_x = x + width.saturating_sub(1);
            if sides & RIGHT != 0 && right_x >= min_x && right_x < max_x {
                if let Some(b) = buffer.get_mut(py * window_width + right_x) {
                    *b = color;
                }
            }
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
    clip: Rect,
) {
    if width == 0 || height == 0 {
        return;
    }

    let radius = radius.min(width / 2).min(height / 2);
    let min_y = y.max(clip.y).min(window_height);
    let max_y = (y + height).min(clip.y + clip.height).min(window_height);
    let min_x = x.max(clip.x).min(window_width);
    let max_x = (x + width).min(clip.x + clip.width).min(window_width);

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
    clip: Rect,
) {
    if width == 0 || height == 0 || thickness == 0 {
        return;
    }

    if radius == 0 {
        draw_rect_outline(buffer, x, y, width, height, window_width, color, clip, border::ALL);
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
    profile!();
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
    clip: Rect,
) {
    profile!();
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

        if y < clip.y || y >= clip.y + clip.height {
            x_long += step_long;
            x_short += step_short;
            continue;
        }

        let pad_short = 1.5 + step_short.abs();

        let left_bound = (x_long - pad_long).min(x_short - pad_short);
        let right_bound = (x_long + pad_long).max(x_short + pad_short);

        let min_x = (left_bound.max(0.0) as usize).max(clip.x).min(window_width);
        let max_x = (right_bound.max(0.0) as usize)
            .min(clip.x + clip.width)
            .min(window_width);

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
    clip: Rect,
) -> Rect {
    fn apply_lcd_filter(bitmap: &mut [u8], width: usize, height: usize) {
        let stride = width * 3;

        for row in 0..height {
            let offset = row * stride;
            let mut left = 0u16;

            for i in 0..stride {
                let idx = offset + i;
                let center = bitmap[idx] as u16;
                let right = if i == stride - 1 { 0 } else { bitmap[idx + 1] as u16 };
                // Apply the [1, 2, 1] filter
                bitmap[idx] = ((left + (center * 2) + right) / 4) as u8;
                // The unfiltered center becomes the left for the next iteration
                left = center;
            }
        }
    }

    if text.is_empty() || font_size == 0 {
        return Rect::default();
    }

    let scaled_font_size = (font_size as f32 * display_scale).round();
    let x_start = (x as f32 * display_scale).round();
    let y_start = (y as f32 * display_scale).round();

    let line_metrics = font.horizontal_line_metrics(scaled_font_size).unwrap();
    let ascent = line_metrics.ascent;

    let (txt_r, txt_g, txt_b) = split_f32(color);

    // Gamma corrected
    let txt_r = (txt_r / 255.0).powi(2);
    let txt_g = (txt_g / 255.0).powi(2);
    let txt_b = (txt_b / 255.0).powi(2);

    let mut y_pos = y_start;
    let mut max_x = x_start as usize;
    let mut max_y = y_start as usize;

    for line in text.lines() {
        let mut glyph_x = x_start;
        let baseline_y = y_pos + ascent;

        for ch in line.chars() {
            let (metrics, bitmap) = cache_map.entry((ch, scaled_font_size as usize)).or_insert_with(|| {
                let (metrics, mut bitmap) = font.rasterize_subpixel(ch, scaled_font_size);
                apply_lcd_filter(&mut bitmap, metrics.width, metrics.height);
                (metrics, bitmap)
            });

            let glyph_screen_y = baseline_y - metrics.height as f32 - metrics.ymin as f32;
            let glyph_screen_x = glyph_x + metrics.xmin as f32;

            // Calculate bounding box
            // Note: Text bounds should ignore screen clipping
            if metrics.width > 0 && metrics.height > 0 {
                let current_max_x = (glyph_screen_x + metrics.width as f32).round() as usize;
                let current_max_y = (glyph_screen_y + metrics.height as f32).round() as usize;

                max_x = max_x.max(current_max_x);
                max_y = max_y.max(current_max_y);
            }

            // Draw the text
            if !skip_draw {
                for y_px in 0..metrics.height {
                    let screen_y = (glyph_screen_y + y_px as f32).round() as i32;

                    if screen_y < 0 || screen_y < clip.y as i32 || screen_y >= (clip.y + clip.height) as i32 {
                        continue;
                    }
                    let screen_y_usize = screen_y as usize;

                    'x_loop: for x_px in 0..metrics.width {
                        let screen_x = (glyph_screen_x + x_px as f32).round() as i32;

                        if screen_x < 0 || screen_x < clip.x as i32 || screen_x >= (clip.x + clip.width) as i32 {
                            continue;
                        }

                        let screen_x_usize = screen_x as usize;
                        if screen_x_usize >= window_width {
                            continue;
                        }

                        let glyph_idx = (y_px * metrics.width + x_px) * 3;
                        let mask_r = bitmap[glyph_idx] as f32 / 255.0;
                        let mask_g = bitmap[glyph_idx + 1] as f32 / 255.0;
                        let mask_b = bitmap[glyph_idx + 2] as f32 / 255.0;

                        if mask_r == 0.0 && mask_g == 0.0 && mask_b == 0.0 {
                            continue;
                        }

                        let buffer_idx = screen_x_usize + (window_width * screen_y_usize);
                        if buffer_idx >= buffer.len() {
                            break 'x_loop;
                        }

                        // Gamma correction.
                        if let Some(bg) = buffer.get_mut(buffer_idx) {
                            let (bg_r, bg_g, bg_b) = split_f32(*bg);

                            // Convert background to linear space (0.0 to 1.0)
                            let bg_r_lin = (bg_r / 255.0).powi(2);
                            let bg_g_lin = (bg_g / 255.0).powi(2);
                            let bg_b_lin = (bg_b / 255.0).powi(2);

                            // Blend background and text in linear space
                            let out_r_lin = (txt_r * mask_r) + (bg_r_lin * (1.0 - mask_r));
                            let out_g_lin = (txt_g * mask_g) + (bg_g_lin * (1.0 - mask_g));
                            let out_b_lin = (txt_b * mask_b) + (bg_b_lin * (1.0 - mask_b));

                            // Convert back to sRGB (0 to 255)
                            let out_r = (out_r_lin.sqrt() * 255.0) as u8;
                            let out_g = (out_g_lin.sqrt() * 255.0) as u8;
                            let out_b = (out_b_lin.sqrt() * 255.0) as u8;

                            *bg = rgb(out_r, out_g, out_b);
                        }

                        // Non-gamma corrected

                        // if let Some(bg) = buffer.get_mut(buffer_idx) {
                        //     let (bg_r, bg_g, bg_b) = split_f32(*bg);
                        //     let out_r = (txt_r * mask_r) + (bg_r * (1.0 - mask_r));
                        //     let out_g = (txt_g * mask_g) + (bg_g * (1.0 - mask_g));
                        //     let out_b = (txt_b * mask_b) + (bg_b * (1.0 - mask_b));
                        //     *bg = rgb(out_r as u8, out_g as u8, out_b as u8);
                        // }
                    }
                }
            }

            glyph_x += metrics.advance_width;

            if glyph_x.round() as usize >= window_width {
                break;
            }
        }
        y_pos += line_metrics.new_line_size;
    }

    Rect {
        x: x_start as usize,
        y: y_start as usize,
        width: if max_x >= x_start as usize {
            max_x + 1 - x_start as usize
        } else {
            0
        },
        height: if max_y >= y_start as usize {
            max_y + 1 - y_start as usize
        } else {
            0
        },
    }
}
