use crate::*;

pub const GAMMA_TO_LINEAR: [f32; 256] = const {
    let mut table = [0.0; 256];
    let mut i = 0;
    while i <= 255 {
        let v = i as f32 / 255.0;
        table[i] = v * v;
        i += 1
    }
    table
};

pub const LINEAR_RESOLUTION: usize = 4096;
pub const LINEAR_INDEX: f32 = 4095.0;

/// Maps a linear float back to an 8-bit sRGB value.
/// Requires multiplying the float by (LINEAR_RESOLUTION - 1) to get the index.
pub const LINEAR_TO_GAMMA: [u8; LINEAR_RESOLUTION] = const {
    let mut table = [0; LINEAR_RESOLUTION];
    let mut i = 0;
    while i < LINEAR_RESOLUTION {
        let v = i as f32 / (LINEAR_RESOLUTION - 1) as f32;
        table[i] = (const_sqrt(v) * 255.0).round() as u8;
        i += 1
    }
    table
};

pub const fn const_sqrt(x: f32) -> f32 {
    if x < 0.0 {
        panic!("Cannot calculate square root of a negative number");
    }
    if x == 0.0 || x == 1.0 {
        return x;
    }

    let mut guess = x / 2.0;
    let mut i = 0;

    // Run loop a fixed number of times since dynamic convergence checks
    // can be trickier in restricted const evaluations
    while i < 100 {
        guess = 0.5 * (guess + x / guess);
        i += 1;
    }
    guess
}

pub const fn scale(value: usize, scale: f32) -> usize {
    (value as f32 * scale).round() as usize
}

pub const fn scale_f32(value: f32, scale: f32) -> i32 {
    (value * scale).round() as i32
}

pub const fn blend(color: u8, alpha: u8, bg_color: u8, bg_alpha: u8) -> u8 {
    ((color as f32 * alpha as f32 + bg_color as f32 * bg_alpha as f32) / 255.0).round() as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

pub fn align_rect(
    parent_x: i32,
    parent_y: i32,
    parent_w: i32,
    parent_h: i32,
    child_w: i32,
    child_h: i32,
    alignment: Alignment,
    padding: Padding,
) -> Option<(i32, i32)> {
    let pad_left = padding.left as i32;
    let pad_right = padding.right as i32;
    let pad_top = padding.top as i32;
    let pad_bottom = padding.bottom as i32;

    let available_w = parent_w - pad_left - pad_right;
    let available_h = parent_h - pad_top - pad_bottom;

    if child_w > available_w || child_h > available_h {
        return None;
    }

    let inner_x = parent_x + pad_left;
    let inner_y = parent_y + pad_top;

    let mid_x = inner_x + (available_w / 2) - (child_w / 2);
    let mid_y = inner_y + (available_h / 2) - (child_h / 2);

    let right_edge = inner_x + available_w - child_w;
    let bottom_edge = inner_y + available_h - child_h;

    Some(match alignment {
        Alignment::Left => (inner_x, mid_y),
        Alignment::Center => (mid_x, mid_y),
        Alignment::Right => (right_edge, mid_y),
        Alignment::TopLeft => (inner_x, inner_y),
        Alignment::TopCenter => (mid_x, inner_y),
        Alignment::TopRight => (right_edge, inner_y),
        Alignment::BottomLeft => (inner_x, bottom_edge),
        Alignment::BottomCenter => (mid_x, bottom_edge),
        Alignment::BottomRight => (right_edge, bottom_edge),
    })
}

pub fn draw_rect(
    buffer: &mut [u32],
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    window_width: usize,
    window_height: usize,
    color: u32,
) {
    let (mut width, mut height) = (width as i32, height as i32);
    let (window_width, window_height) = (window_width as i32, window_height as i32);

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
        if pos > 0 {
            let pos = pos as usize;
            if let Some(buffer) = buffer.get_mut(pos..pos + width as usize) {
                buffer.fill(color);
            }
        }
    }
}

pub fn draw_rect_outline(
    buffer: &mut [u32],
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    window_width: usize,
    color: u32,
    clip: Rect,
    sides: u8,
) {
    use border::*;

    if width == 0 || height == 0 || window_width == 0 {
        return;
    }

    let right = x + width.saturating_sub(1) as i32;
    let bottom = y + height.saturating_sub(1) as i32;
    let min_x = x.max(clip.x).max(0).min(window_width as i32) as usize;
    let max_x = (right + 1)
        .min(clip.right())
        .min(window_width as i32)
        .max(0) as usize;
    let min_y = y.max(clip.y).max(0).min((buffer.len() / window_width) as i32) as usize;
    let max_y = (bottom + 1)
        .min(clip.bottom())
        .min((buffer.len() / window_width) as i32)
        .max(0) as usize;

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    if sides & TOP != 0 && y >= min_y as i32 && y < max_y as i32 {
        let start = y as usize * window_width + min_x;
        if let Some(slice) = buffer.get_mut(start..start + max_x - min_x) {
            slice.fill(color);
        }
    }

    if sides & BOTTOM != 0 && bottom >= min_y as i32 && bottom < max_y as i32 {
        let start = bottom as usize * window_width + min_x;
        if let Some(slice) = buffer.get_mut(start..start + max_x - min_x) {
            slice.fill(color);
        }
    }

    for (side, px) in [(LEFT, x), (RIGHT, right)] {
        if sides & side == 0 || px < min_x as i32 || px >= max_x as i32 {
            continue;
        }

        for py in min_y..max_y {
            if let Some(b) = buffer.get_mut(py * window_width + px as usize) {
                *b = color;
            }
        }
    }
}

pub fn draw_rounded_rect(
    buffer: &mut [u32],
    x: i32,
    y: i32,
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
    let min_y = y.max(clip.y).max(0).min(window_height as i32) as usize;
    let max_y = (y + height as i32)
        .min(clip.bottom())
        .min(window_height as i32)
        .max(0) as usize;
    let min_x = x.max(clip.x).max(0).min(window_width as i32) as usize;
    let max_x = (x + width as i32)
        .min(clip.right())
        .min(window_width as i32)
        .max(0) as usize;

    // Fully clipped
    if min_x >= max_x || min_y >= max_y {
        return;
    }

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

    let src_r = ((color >> 16) & 0xFF) as usize;
    let src_g = ((color >> 8) & 0xFF) as usize;
    let src_b = (color & 0xFF) as usize;

    let src_r_lin = GAMMA_TO_LINEAR[src_r];
    let src_g_lin = GAMMA_TO_LINEAR[src_g];
    let src_b_lin = GAMMA_TO_LINEAR[src_b];

    let cx = x as f32 + width as f32 / 2.0;
    let cy = y as f32 + height as f32 / 2.0;
    let half_w = width as f32 / 2.0;
    let half_h = height as f32 / 2.0;
    let r_f32 = radius as f32;

    let y_top_safe = (y + radius as i32).max(0) as usize;
    let y_bottom_safe = (y + height as i32 - radius as i32).max(0) as usize;
    let x_left_safe = (x + radius as i32).max(0) as usize;
    let x_right_safe = (x + width as i32 - radius as i32).max(0) as usize;

    let left_limit = x_left_safe.max(min_x).min(max_x);
    let right_limit = x_right_safe.max(min_x).min(max_x);

    macro_rules! blend_edge {
        ($bg:expr, $alpha:expr) => {
            let a = $alpha.clamp(0.0, 1.0);
            let bg_val = *$bg;
            let bg_r = (bg_val >> 16) & 0xFF;
            let bg_g = (bg_val >> 8) & 0xFF;
            let bg_b = bg_val & 0xFF;

            let bg_r_lin = GAMMA_TO_LINEAR[bg_r as usize];
            let bg_g_lin = GAMMA_TO_LINEAR[bg_g as usize];
            let bg_b_lin = GAMMA_TO_LINEAR[bg_b as usize];

            let out_r_lin = (src_r_lin * a) + (bg_r_lin * (1.0 - a));
            let out_g_lin = (src_g_lin * a) + (bg_g_lin * (1.0 - a));
            let out_b_lin = (src_b_lin * a) + (bg_b_lin * (1.0 - a));

            let out_r = LINEAR_TO_GAMMA[(out_r_lin * LINEAR_INDEX).clamp(0.0, LINEAR_INDEX) as usize] as u32;
            let out_g = LINEAR_TO_GAMMA[(out_g_lin * LINEAR_INDEX).clamp(0.0, LINEAR_INDEX) as usize] as u32;
            let out_b = LINEAR_TO_GAMMA[(out_b_lin * LINEAR_INDEX).clamp(0.0, LINEAR_INDEX) as usize] as u32;

            *$bg = (out_r << 16) | (out_g << 8) | out_b;
        };
    }

    for py in min_y..max_y {
        let row_start = py * window_width;

        if py >= y_top_safe && py < y_bottom_safe {
            if let Some(slice) = buffer.get_mut(row_start + min_x..row_start + max_x) {
                slice.fill(color);
            }
            continue;
        }

        let dy = (py as f32 + 0.5 - cy).abs() - half_h + r_f32;
        let dy_max = dy.max(0.0);
        let dy_sq = dy_max * dy_max;

        if let Some(row_slice) = buffer.get_mut(row_start + min_x..row_start + max_x) {
            let left_len = left_limit - min_x;
            let mid_len = right_limit - left_limit;

            let (left_slice, rest) = row_slice.split_at_mut(left_len);
            let (mid_slice, right_slice) = rest.split_at_mut(mid_len);

            // Left Corner
            for (i, bg) in left_slice.iter_mut().enumerate() {
                let px = min_x + i;
                let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                let dx_max = dx.max(0.0);

                let dist_outer = (dx_max * dx_max + dy_sq).sqrt();
                let dist_inner = dx.max(dy).min(0.0);
                let dist = dist_outer + dist_inner - r_f32;
                let alpha = 0.5 - dist;

                if alpha >= 0.999 {
                    *bg = color;
                } else if alpha > 0.0 {
                    blend_edge!(bg, alpha);
                }
            }

            // Middle Solid Segment
            mid_slice.fill(color);

            // Right Corner
            for (i, bg) in right_slice.iter_mut().enumerate() {
                let px = right_limit + i;
                let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                let dx_max = dx.max(0.0);

                let dist_outer = (dx_max * dx_max + dy_sq).sqrt();
                let dist_inner = dx.max(dy).min(0.0);
                let dist = dist_outer + dist_inner - r_f32;
                let alpha = 0.5 - dist;

                if alpha >= 0.999 {
                    *bg = color;
                } else if alpha > 0.0 {
                    blend_edge!(bg, alpha);
                }
            }
        }
    }
}

pub fn draw_rounded_rect_outline(
    buffer: &mut [u32],
    x: i32,
    y: i32,
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
    let min_y = y.max(clip.y).max(0).min(window_height as i32) as usize;
    let max_y = (y + height as i32)
        .min(clip.bottom())
        .min(window_height as i32)
        .max(0) as usize;
    let min_x = x.max(clip.x).max(0).min(window_width as i32) as usize;
    let max_x = (x + width as i32)
        .min(clip.right())
        .min(window_width as i32)
        .max(0) as usize;

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
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
    color: u32,
    clip: Rect,
) {
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

    if y0 >= window_height as i32 || y2 <= 0 || y0 == y2 {
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

    let height_top = y1 - y0;
    let step_short_top = if height_top > 0 {
        (x1 as f32 - x0 as f32) / height_top as f32
    } else {
        0.0
    };
    let height_bottom = y2 - y1;
    let step_short_bottom = if height_bottom > 0 {
        (x2 as f32 - x1 as f32) / height_bottom as f32
    } else {
        0.0
    };

    let pad_long = 1.5 + step_long.abs();

    let min_y = y0.max(clip.y).max(0);
    let max_y = y2
        .min(clip.bottom().saturating_sub(1))
        .min(window_height.saturating_sub(1) as i32);

    if min_y > max_y {
        return;
    }

    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        let x_long = x0 as f32 + step_long * (y - y0) as f32;
        let (x_short, step_short) = if y < y1 {
            (x0 as f32 + step_short_top * (y - y0) as f32, step_short_top)
        } else {
            (x1 as f32 + step_short_bottom * (y - y1) as f32, step_short_bottom)
        };

        let pad_short = 1.5 + step_short.abs();

        let left_bound = (x_long - pad_long).min(x_short - pad_short);
        let right_bound = (x_long + pad_long).max(x_short + pad_short);

        let min_x = (left_bound.max(0.0) as i32).max(clip.x).max(0).min(window_width as i32) as usize;
        let max_x = (right_bound.max(0.0) as i32)
            .min(clip.right())
            .max(0)
            .min(window_width as i32) as usize;

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
                let idx = y as usize * window_width + x;
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
            let start_idx = y as usize * window_width + solid_start;
            let end_idx = y as usize * window_width + solid_end;
            if let Some(slice) = buffer.get_mut(start_idx..end_idx) {
                slice.fill(color);
            }
        }
    }
}

pub fn apply_lcd_filter(bitmap: &mut [u8], width: usize, height: usize) {
    let stride = width * 3;
    for row in 0..height {
        let offset = row * stride;
        let mut left = 0u16;
        for i in 0..stride {
            let idx = offset + i;
            let center = bitmap[idx] as u16;
            let right = if i == stride - 1 { 0 } else { bitmap[idx + 1] as u16 };
            bitmap[idx] = ((left + (center * 2) + right) / 4) as u8;
            left = center;
        }
    }
}

pub fn draw_text(
    text: &str,
    font: &fontdue::Font,
    x: i32,
    y: i32,
    font_size: usize,
    display_scale: f32,
    window_width: usize,
    buffer: &mut [u32],
    color: u32,
    cache: &mut FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>,
    clip: Rect,
) -> Rect {
    if text.is_empty() || font_size == 0 || window_width == 0 {
        return Rect::default();
    }

    let scaled_font_size = (font_size as f32 * display_scale).round();
    let x_start = (x as f32 * display_scale).round();
    let y_start = (y as f32 * display_scale).round();

    let line_metrics = font.horizontal_line_metrics(scaled_font_size).unwrap();
    let ascent = line_metrics.ascent;

    let (txt_r, txt_g, txt_b) = split(color);
    let txt_r_lin = GAMMA_TO_LINEAR[txt_r as usize];
    let txt_g_lin = GAMMA_TO_LINEAR[txt_g as usize];
    let txt_b_lin = GAMMA_TO_LINEAR[txt_b as usize];

    let mut y_pos = y_start;
    let mut max_x = x_start.max(0.0) as usize;
    let mut max_y = y_start.max(0.0) as usize;

    let clip_x = clip.x;
    let clip_y = clip.y;
    let clip_right = clip.right();
    let clip_bottom = clip.bottom();
    let buffer_height = (buffer.len() / window_width) as i32;

    for line in text.lines() {
        let mut glyph_x = x_start;
        let baseline_y = y_pos + ascent;

        for ch in line.chars() {
            let (metrics, bitmap) = cache.entry((ch, scaled_font_size as usize)).or_insert_with(|| {
                let (metrics, mut bitmap) = font.rasterize_subpixel(ch, scaled_font_size);
                apply_lcd_filter(&mut bitmap, metrics.width, metrics.height);
                (metrics, bitmap)
            });

            let glyph_screen_y = (baseline_y - metrics.height as f32 - metrics.ymin as f32).round() as i32;
            let glyph_screen_x = (glyph_x + metrics.xmin as f32).round() as i32;

            if metrics.width > 0 && metrics.height > 0 {
                let current_max_x = (glyph_screen_x + metrics.width as i32).max(0) as usize;
                let current_max_y = (glyph_screen_y + metrics.height as i32).max(0) as usize;
                max_x = max_x.max(current_max_x);
                max_y = max_y.max(current_max_y);
            }

            // Exact Clip Intersection
            let draw_start_x = glyph_screen_x.max(clip_x).max(0);
            let draw_end_x = (glyph_screen_x + metrics.width as i32)
                .min(clip_right)
                .min(window_width as i32);

            let draw_start_y = glyph_screen_y.max(clip_y).max(0);
            let draw_end_y = (glyph_screen_y + metrics.height as i32)
                .min(clip_bottom)
                .min(buffer_height);

            if draw_start_x < draw_end_x && draw_start_y < draw_end_y {
                let draw_width = (draw_end_x - draw_start_x) as usize;
                let bitmap_offset_x = (draw_start_x - glyph_screen_x) as usize;

                for screen_y in draw_start_y..draw_end_y {
                    let bitmap_y = (screen_y - glyph_screen_y) as usize;

                    // Exact memory slicing (Elides bounds checks)
                    let buffer_start = screen_y as usize * window_width + draw_start_x as usize;
                    // Grab the pixels we will write to in the window buffer
                    let buffer_row = &mut buffer[buffer_start..buffer_start + draw_width];

                    let bitmap_start = (bitmap_y * metrics.width + bitmap_offset_x) * 3;
                    // Grab the subpixel masks for this row
                    let bitmap_row = &bitmap[bitmap_start..bitmap_start + draw_width * 3];

                    const INV_255: f32 = 1.0 / 255.0; // Mult is faster than div (on my machine)

                    // Zip iteration allows the compiler to auto-vectorize
                    for (i, bg) in buffer_row.iter_mut().enumerate() {
                        let mask_idx = i * 3;
                        let m_r = bitmap_row[mask_idx];
                        let m_g = bitmap_row[mask_idx + 1];
                        let m_b = bitmap_row[mask_idx + 2];

                        // Skip empty pixels to save memory bandwidth (avoid dirtying cache lines)
                        if m_r == 0 && m_g == 0 && m_b == 0 {
                            continue;
                        }

                        let mask_r = m_r as f32 * INV_255;
                        let mask_g = m_g as f32 * INV_255;
                        let mask_b = m_b as f32 * INV_255;

                        let (bg_r, bg_g, bg_b) = split(*bg);

                        // LUT lookups
                        let bg_r_lin = GAMMA_TO_LINEAR[bg_r as usize];
                        let bg_g_lin = GAMMA_TO_LINEAR[bg_g as usize];
                        let bg_b_lin = GAMMA_TO_LINEAR[bg_b as usize];

                        // Fused Multiply-Add
                        let out_r_lin = txt_r_lin.mul_add(mask_r, bg_r_lin * (1.0 - mask_r));
                        let out_g_lin = txt_g_lin.mul_add(mask_g, bg_g_lin * (1.0 - mask_g));
                        let out_b_lin = txt_b_lin.mul_add(mask_b, bg_b_lin * (1.0 - mask_b));

                        let out_r = LINEAR_TO_GAMMA[(out_r_lin * LINEAR_INDEX) as usize] as u32;
                        let out_g = LINEAR_TO_GAMMA[(out_g_lin * LINEAR_INDEX) as usize] as u32;
                        let out_b = LINEAR_TO_GAMMA[(out_b_lin * LINEAR_INDEX) as usize] as u32;

                        *bg = (out_r << 16) | (out_g << 8) | out_b;
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

    let x0 = x_start as i32;
    let y0 = y_start as i32;
    Rect {
        x: x0,
        y: y0,
        width: if max_x as i32 >= x0 {
            max_x as i32 + 1 - x0
        } else {
            0
        },
        height: if max_y as i32 >= y0 {
            max_y as i32 + 1 - y0
        } else {
            0
        },
    }
}

pub fn measure_text(
    text: &str,
    font: &fontdue::Font,
    font_size: usize,
    display_scale: f32,
    metrics: &mut FxHashMap<(char, usize), fontdue::Metrics>,
) -> Rect {
    if text.is_empty() || font_size == 0 {
        return Rect::default();
    }

    let scaled_font_size = (font_size as f32 * display_scale).round();
    let line_metrics = font.horizontal_line_metrics(scaled_font_size).unwrap();

    let mut max_width = 0.0f32;
    let mut current_width = 0.0f32;
    let mut lines = 1;

    for ch in text.chars() {
        if ch == '\n' {
            max_width = max_width.max(current_width);
            current_width = 0.0;
            lines += 1;
            continue;
        }

        let metrics = metrics
            .entry((ch, scaled_font_size as usize))
            .or_insert_with(|| font.metrics(ch, scaled_font_size));

        current_width += metrics.advance_width;
    }

    max_width = max_width.max(current_width);

    Rect {
        x: 0,
        y: 0,
        width: (max_width / display_scale).round() as i32,
        height: ((lines as f32 * line_metrics.new_line_size) / display_scale).round() as i32,
    }
}
