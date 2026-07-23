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

pub const fn scale(value: usize, factor: f32) -> usize {
    (value as f32 * factor).round() as usize
}

pub const fn scale_i32(value: i32, factor: f32) -> i32 {
    (value as f32 * factor).round() as i32
}

/// Returns `None` if fully clipped.
#[inline]
pub fn visible_rect(shape: Rect, clip: Rect, fb_w: i32, fb_h: i32) -> Option<Rect> {
    let r = shape.intersection(clip).clamp_to_size(fb_w, fb_h);
    if r.is_empty() { None } else { Some(r) }
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

#[rustfmt::skip]
pub fn align_rect(
    parent: Rect,
    child_w: i32,
    child_h: i32,
    alignment: Alignment,
    padding: Padding,
) -> Option<(i32, i32)> {
    let pad_left = padding.left as i32;
    let pad_right = padding.right as i32;
    let pad_top = padding.top as i32;
    let pad_bottom = padding.bottom as i32;

    let available_w = parent.width - pad_left - pad_right;
    let available_h = parent.height - pad_top - pad_bottom;

    if available_w <= 0 || available_h <= 0 {
        return None;
    }

    let inner_x = parent.x + pad_left;
    let inner_y = parent.y + pad_top;

    let align_x = if child_w >= available_w { inner_x } else { inner_x + (available_w - child_w) / 2 };
    let align_y = if child_h >= available_h { inner_y } else { inner_y + (available_h - child_h) / 2 };
    let right_x = if child_w >= available_w { inner_x } else { inner_x + available_w - child_w };
    let bottom_y = if child_h >= available_h { inner_y } else { inner_y + available_h - child_h };

    Some(match alignment {
        Alignment::Left => (inner_x, align_y),
        Alignment::Center => (align_x, align_y),
        Alignment::Right => (right_x, align_y),
        Alignment::TopLeft => (inner_x, inner_y),
        Alignment::TopCenter => (align_x, inner_y),
        Alignment::TopRight => (right_x, inner_y),
        Alignment::BottomLeft => (inner_x, bottom_y),
        Alignment::BottomCenter => (align_x, bottom_y),
        Alignment::BottomRight => (right_x, bottom_y),
    })
}

#[inline]
fn solid_fill_span(buffer: &mut [u32], window_width: usize, x0: i32, x1: i32, y0: i32, y1: i32, color: u32) {
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let x0 = x0 as usize;
    let x1 = x1 as usize;
    let len = x1 - x0;
    for py in y0 as usize..y1 as usize {
        let start = py * window_width + x0;
        if let Some(slice) = buffer.get_mut(start..start + len) {
            slice.fill(color);
        }
    }
}

#[inline]
fn solid_fill_rect(buffer: &mut [u32], window_width: usize, area: Rect, color: u32) {
    solid_fill_span(buffer, window_width, area.x, area.right(), area.y, area.bottom(), color);
}

#[inline]
fn color_linear(color: u32) -> (f32, f32, f32) {
    (
        GAMMA_TO_LINEAR[((color >> 16) & 0xFF) as usize],
        GAMMA_TO_LINEAR[((color >> 8) & 0xFF) as usize],
        GAMMA_TO_LINEAR[(color & 0xFF) as usize],
    )
}

#[inline]
fn blend_srgb(bg: &mut u32, src: (f32, f32, f32), alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    let bg_val = *bg;
    let inv = 1.0 - a;

    let out_r = src.0 * a + GAMMA_TO_LINEAR[((bg_val >> 16) & 0xFF) as usize] * inv;
    let out_g = src.1 * a + GAMMA_TO_LINEAR[((bg_val >> 8) & 0xFF) as usize] * inv;
    let out_b = src.2 * a + GAMMA_TO_LINEAR[(bg_val & 0xFF) as usize] * inv;

    let r = LINEAR_TO_GAMMA[(out_r * LINEAR_INDEX).clamp(0.0, LINEAR_INDEX) as usize] as u32;
    let g = LINEAR_TO_GAMMA[(out_g * LINEAR_INDEX).clamp(0.0, LINEAR_INDEX) as usize] as u32;
    let b = LINEAR_TO_GAMMA[(out_b * LINEAR_INDEX).clamp(0.0, LINEAR_INDEX) as usize] as u32;
    *bg = (r << 16) | (g << 8) | b;
}

#[inline]
fn apply_coverage(bg: &mut u32, color: u32, src: (f32, f32, f32), alpha: f32) {
    if alpha >= 0.999 {
        *bg = color;
    } else if alpha > 0.0 {
        blend_srgb(bg, src, alpha);
    }
}

#[inline]
fn rounded_rect_sdf(dx: f32, dy: f32, r: f32) -> f32 {
    let dx_max = dx.max(0.0);
    let dy_max = dy.max(0.0);
    let dist_outer = (dx_max * dx_max + dy_max * dy_max).sqrt();
    let dist_inner = dx.max(dy).min(0.0);
    dist_outer + dist_inner - r
}

#[inline]
fn stroke_sdf(dist: f32, thickness: f32) -> f32 {
    dist.max(-(dist + thickness))
}

#[inline]
fn rounded_stroke_has_side(px: i32, py: i32, bounds: Rect, radius: i32, thickness: i32, sides: u8) -> bool {
    use border::*;

    let x = bounds.x;
    let y = bounds.y;
    let right = bounds.right();
    let bottom = bounds.bottom();
    // Include one extra pixel for the antialiased inner edge.
    let edge = thickness + 1;
    let in_corner_x = px < x + radius || px >= right - radius;
    let in_corner_y = py < y + radius || py >= bottom - radius;

    (sides & TOP != 0 && (py < y + edge || (py < y + radius && in_corner_x)))
        || (sides & BOTTOM != 0 && (py >= bottom - edge || (py >= bottom - radius && in_corner_x)))
        || (sides & LEFT != 0 && (px < x + edge || (px < x + radius && in_corner_y)))
        || (sides & RIGHT != 0 && (px >= right - edge || (px >= right - radius && in_corner_y)))
}

fn draw_axis_aligned_stroke(
    buffer: &mut [u32],
    bounds: Rect,
    window_width: usize,
    window_height: usize,
    thickness: usize,
    color: u32,
    clip: Rect,
    sides: u8,
) {
    use border::*;

    if bounds.is_empty() || thickness == 0 || window_width == 0 || sides == NONE {
        return;
    }

    let Some(vis) = visible_rect(bounds, clip, window_width as i32, window_height as i32) else {
        return;
    };

    let t = thickness as i32;
    let x0 = bounds.x;
    let y0 = bounds.y;
    let x1 = bounds.right();
    let y1 = bounds.bottom();

    if sides & TOP != 0 {
        solid_fill_rect(
            buffer,
            window_width,
            Rect::from_xyxy(x0, y0, x1, y0 + t).intersection(vis),
            color,
        );
    }
    if sides & BOTTOM != 0 {
        solid_fill_rect(
            buffer,
            window_width,
            Rect::from_xyxy(x0, y1 - t, x1, y1).intersection(vis),
            color,
        );
    }
    if sides & LEFT != 0 {
        solid_fill_rect(
            buffer,
            window_width,
            Rect::from_xyxy(x0, y0, x0 + t, y1).intersection(vis),
            color,
        );
    }
    if sides & RIGHT != 0 {
        solid_fill_rect(
            buffer,
            window_width,
            Rect::from_xyxy(x1 - t, y0, x1, y1).intersection(vis),
            color,
        );
    }
}

pub fn draw_rect_fill(
    buffer: &mut [u32],
    bounds: Rect,
    window_width: usize,
    window_height: usize,
    radius: usize,
    color: u32,
    clip: Rect,
) {
    if bounds.is_empty() || window_width == 0 {
        return;
    }

    let radius = radius.min(bounds.width as usize / 2).min(bounds.height as usize / 2);
    let Some(vis) = visible_rect(bounds, clip, window_width as i32, window_height as i32) else {
        return;
    };

    let min_x = vis.x as usize;
    let max_x = vis.right() as usize;
    let min_y = vis.y as usize;
    let max_y = vis.bottom() as usize;

    if radius == 0 {
        solid_fill_span(buffer, window_width, vis.x, vis.right(), vis.y, vis.bottom(), color);
        return;
    }

    let src = color_linear(color);
    let x = bounds.x;
    let y = bounds.y;
    let width = bounds.width as usize;
    let height = bounds.height as usize;
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

    for py in min_y..max_y {
        let row_start = py * window_width;

        if py >= y_top_safe && py < y_bottom_safe {
            if let Some(slice) = buffer.get_mut(row_start + min_x..row_start + max_x) {
                slice.fill(color);
            }
            continue;
        }

        let dy = (py as f32 + 0.5 - cy).abs() - half_h + r_f32;

        if let Some(row_slice) = buffer.get_mut(row_start + min_x..row_start + max_x) {
            let left_len = left_limit - min_x;
            let mid_len = right_limit - left_limit;
            let (left_slice, rest) = row_slice.split_at_mut(left_len);
            let (mid_slice, right_slice) = rest.split_at_mut(mid_len);

            for (i, bg) in left_slice.iter_mut().enumerate() {
                let px = min_x + i;
                let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                apply_coverage(bg, color, src, 0.5 - rounded_rect_sdf(dx, dy, r_f32));
            }

            mid_slice.fill(color);

            for (i, bg) in right_slice.iter_mut().enumerate() {
                let px = right_limit + i;
                let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                apply_coverage(bg, color, src, 0.5 - rounded_rect_sdf(dx, dy, r_f32));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_rect_stroke(
    buffer: &mut [u32],
    bounds: Rect,
    window_width: usize,
    window_height: usize,
    radius: usize,
    thickness: usize,
    color: u32,
    clip: Rect,
    sides: u8,
) {
    if bounds.is_empty() || thickness == 0 || window_width == 0 || sides == border::NONE {
        return;
    }

    let radius = radius.min(bounds.width as usize / 2).min(bounds.height as usize / 2);
    if radius == 0 {
        draw_axis_aligned_stroke(
            buffer,
            bounds,
            window_width,
            window_height,
            thickness,
            color,
            clip,
            sides,
        );
        return;
    }

    let Some(vis) = visible_rect(bounds, clip, window_width as i32, window_height as i32) else {
        return;
    };

    let x = bounds.x;
    let y = bounds.y;
    let w = bounds.width;
    let h = bounds.height;
    let r = radius as i32;
    let t = thickness as i32;
    let t_f32 = thickness as f32;
    let r_f32 = radius as f32;

    let cx = x as f32 + w as f32 / 2.0;
    let cy = y as f32 + h as f32 / 2.0;
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    let src = color_linear(color);

    let min_x = vis.x as usize;
    let max_x = vis.right() as usize;
    let min_y = vis.y as usize;
    let max_y = vis.bottom() as usize;

    if sides != border::ALL {
        for py in min_y..max_y {
            let dy = (py as f32 + 0.5 - cy).abs() - half_h + r_f32;
            let row = py * window_width;
            for px in min_x..max_x {
                if !rounded_stroke_has_side(px as i32, py as i32, bounds, r, t, sides) {
                    continue;
                }
                let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                let alpha = 0.5 - stroke_sdf(rounded_rect_sdf(dx, dy, r_f32), t_f32);
                if let Some(bg) = buffer.get_mut(row + px) {
                    apply_coverage(bg, color, src, alpha);
                }
            }
        }
        return;
    }

    let y_top_safe = (y + r).max(0) as usize;
    let y_bottom_safe = (y + h - r).max(0) as usize;
    let x_left_safe = (x + r).max(0) as usize;
    let x_right_safe = (x + w - r).max(0) as usize;
    let left_limit = x_left_safe.max(min_x).min(max_x);
    let right_limit = x_right_safe.max(min_x).min(max_x);

    let edge_w = (t + 1).min(w);
    let x_left_edge = (x + edge_w).max(0) as usize;
    let x_right_edge = (x + w - edge_w).max(0) as usize;
    let left_edge_limit = x_left_edge.max(min_x).min(max_x);
    let right_edge_limit = x_right_edge.max(min_x).min(max_x);

    let y_top_edge = (y + (t + 1).min(h)).max(0) as usize;
    let y_bottom_edge = (y + h - (t + 1).min(h)).max(0) as usize;

    for py in min_y..max_y {
        let dy = (py as f32 + 0.5 - cy).abs() - half_h + r_f32;
        let row = py * window_width;

        let in_mid_y = py >= y_top_safe && py < y_bottom_safe;
        let in_top_edge = py < y_top_edge;
        let in_bottom_edge = py >= y_bottom_edge;

        if in_mid_y && !in_top_edge && !in_bottom_edge {
            if left_edge_limit >= right_edge_limit {
                for px in min_x..max_x {
                    let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                    let alpha = 0.5 - stroke_sdf(rounded_rect_sdf(dx, dy, r_f32), t_f32);
                    if let Some(bg) = buffer.get_mut(row + px) {
                        apply_coverage(bg, color, src, alpha);
                    }
                }
            } else {
                for px in min_x..left_edge_limit {
                    let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                    let alpha = 0.5 - stroke_sdf(rounded_rect_sdf(dx, dy, r_f32), t_f32);
                    if let Some(bg) = buffer.get_mut(row + px) {
                        apply_coverage(bg, color, src, alpha);
                    }
                }
                for px in right_edge_limit..max_x {
                    let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                    let alpha = 0.5 - stroke_sdf(rounded_rect_sdf(dx, dy, r_f32), t_f32);
                    if let Some(bg) = buffer.get_mut(row + px) {
                        apply_coverage(bg, color, src, alpha);
                    }
                }
            }
            continue;
        }

        if in_mid_y && (in_top_edge || in_bottom_edge) {
            for px in min_x..max_x {
                let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                let alpha = 0.5 - stroke_sdf(rounded_rect_sdf(dx, dy, r_f32), t_f32);
                if let Some(bg) = buffer.get_mut(row + px) {
                    apply_coverage(bg, color, src, alpha);
                }
            }
            continue;
        }

        for px in min_x..left_limit {
            let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
            let alpha = 0.5 - stroke_sdf(rounded_rect_sdf(dx, dy, r_f32), t_f32);
            if let Some(bg) = buffer.get_mut(row + px) {
                apply_coverage(bg, color, src, alpha);
            }
        }

        if in_top_edge || in_bottom_edge {
            for px in left_limit..right_limit {
                let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                let alpha = 0.5 - stroke_sdf(rounded_rect_sdf(dx, dy, r_f32), t_f32);
                if let Some(bg) = buffer.get_mut(row + px) {
                    apply_coverage(bg, color, src, alpha);
                }
            }
        }

        for px in right_limit..max_x {
            let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
            let alpha = 0.5 - stroke_sdf(rounded_rect_sdf(dx, dy, r_f32), t_f32);
            if let Some(bg) = buffer.get_mut(row + px) {
                apply_coverage(bg, color, src, alpha);
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

    let src = color_linear(color);

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

    let y_span = Rect::from_xyxy(0, y0, window_width as i32, y2 + 1);
    let Some(vis_y) = visible_rect(y_span, clip, window_width as i32, window_height as i32) else {
        return;
    };
    let min_y = vis_y.y;
    let max_y = vis_y.bottom() - 1;
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

        let x_span = Rect::from_xyxy(left_bound.max(0.0) as i32, y, right_bound.max(0.0) as i32, y + 1);
        let Some(vis_x) = visible_rect(x_span, clip, window_width as i32, window_height as i32) else {
            continue;
        };
        let min_x = vis_x.x as usize;
        let max_x = vis_x.right() as usize;

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
                    blend_srgb(bg, src, alpha);
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
            let right = if i + 1 < stride { bitmap[idx + 1] as u16 } else { 0 };
            bitmap[idx] = ((left + center * 2 + right) / 4) as u8;
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
    window_width: usize,
    buffer: &mut [u32],
    color: u32,
    cache: &mut FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>,
    clip: Rect,
) -> Rect {
    if text.is_empty() || font_size == 0 || window_width == 0 {
        return Rect::default();
    }

    let size = font_size as f32;
    let x_start = x as f32;
    let y_start = y as f32;

    let line_metrics = font.horizontal_line_metrics(size).unwrap();
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
            let (metrics, bitmap) = cache.entry((ch, font_size)).or_insert_with(|| {
                let (metrics, mut bitmap) = font.rasterize_subpixel(ch, size);
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

                    let buffer_start = screen_y as usize * window_width + draw_start_x as usize;
                    let buffer_row = &mut buffer[buffer_start..buffer_start + draw_width];

                    let bitmap_start = (bitmap_y * metrics.width + bitmap_offset_x) * 3;
                    let bitmap_row = &bitmap[bitmap_start..bitmap_start + draw_width * 3];

                    const INV_255: f32 = 1.0 / 255.0;

                    for (i, bg) in buffer_row.iter_mut().enumerate() {
                        let mask_idx = i * 3;
                        let m_r = bitmap_row[mask_idx];
                        let m_g = bitmap_row[mask_idx + 1];
                        let m_b = bitmap_row[mask_idx + 2];

                        if m_r | m_g | m_b == 0 {
                            continue;
                        }

                        let mask_r = m_r as f32 * INV_255;
                        let mask_g = m_g as f32 * INV_255;
                        let mask_b = m_b as f32 * INV_255;

                        let (bg_r, bg_g, bg_b) = split(*bg);

                        let bg_r_lin = GAMMA_TO_LINEAR[bg_r as usize];
                        let bg_g_lin = GAMMA_TO_LINEAR[bg_g as usize];
                        let bg_b_lin = GAMMA_TO_LINEAR[bg_b as usize];

                        let out_r_lin = txt_r_lin * mask_r + bg_r_lin * (1.0 - mask_r);
                        let out_g_lin = txt_g_lin * mask_g + bg_g_lin * (1.0 - mask_g);
                        let out_b_lin = txt_b_lin * mask_b + bg_b_lin * (1.0 - mask_b);

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

    let x0 = x;
    let y0 = y;
    Rect {
        x: x0,
        y: y0,
        width: if max_x as i32 >= x0 { max_x as i32 + 1 - x0 } else { 0 },
        height: if max_y as i32 >= y0 { max_y as i32 + 1 - y0 } else { 0 },
    }
}

pub fn measure_text(
    text: &str,
    font: &fontdue::Font,
    font_size: usize,
    metrics: &mut FxHashMap<(char, usize), fontdue::Metrics>,
) -> Rect {
    if text.is_empty() || font_size == 0 {
        return Rect::default();
    }

    let size = font_size as f32;
    let line_metrics = font.horizontal_line_metrics(size).unwrap();

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

        let metrics = metrics.entry((ch, font_size)).or_insert_with(|| font.metrics(ch, size));

        current_width += metrics.advance_width;
    }

    max_width = max_width.max(current_width);

    Rect {
        x: 0,
        y: 0,
        width: max_width.round() as i32,
        height: (lines as f32 * line_metrics.new_line_size).round() as i32,
    }
}

pub fn clear_damage(buffer: &mut [u32], framebuffer_width: usize, damage: &[Rect], color: u32) {
    for rect in damage {
        if rect.is_empty() {
            continue;
        }
        let x = rect.x.max(0) as usize;
        let y0 = rect.y.max(0) as usize;
        let y1 = rect.bottom().max(0) as usize;
        let width = rect.width.max(0) as usize;
        for y in y0..y1 {
            let start = y * framebuffer_width + x;
            buffer[start..start + width].fill(color);
        }
    }
}

pub fn draw_command(
    command: &Command<'_>,
    damage: Rect,
    buffer: &mut [u32],
    framebuffer_width: usize,
    framebuffer_height: usize,
    display_scale: f32,
    fonts: &[fontdue::Font],
    font_bitmaps: &mut FxHashMap<usize, FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    image_cache: &mut FxHashMap<ImageKey, ImageEntry>,
) {
    let clip = command.clip().scale(display_scale).intersection(damage);
    if clip.is_empty() {
        return;
    }

    match command {
        Command::Rect {
            bounds,
            color,
            radius,
            clip: _,
        } => draw_rect_fill(
            buffer,
            bounds.scale(display_scale),
            framebuffer_width,
            framebuffer_height,
            scale(*radius, display_scale),
            *color,
            clip,
        ),
        Command::RectStroke {
            bounds,
            color,
            border_sides,
            border_thickness,
            radius,
            clip: _,
        } => draw_rect_stroke(
            buffer,
            bounds.scale(display_scale),
            framebuffer_width,
            framebuffer_height,
            scale(*radius, display_scale),
            scale(*border_thickness, display_scale).max(1),
            *color,
            clip,
            *border_sides,
        ),
        Command::Text {
            text,
            bounds,
            color,
            size,
            font_id,
            clip: _,
        } => {
            let bitmap = font_bitmaps.entry(*font_id).or_default();
            let origin = bounds.scale(display_scale);
            draw_text(
                text,
                &fonts[*font_id],
                origin.x,
                origin.y,
                scale(*size, display_scale),
                framebuffer_width,
                buffer,
                *color,
                bitmap,
                clip,
            );
        }
        Command::Triangle {
            a,
            b,
            c,
            color,
            clip: _,
        } => draw_triangle_sdf(
            buffer,
            framebuffer_width,
            framebuffer_height,
            scale_i32(a.0, display_scale),
            scale_i32(a.1, display_scale),
            scale_i32(b.0, display_scale),
            scale_i32(b.1, display_scale),
            scale_i32(c.0, display_scale),
            scale_i32(c.1, display_scale),
            *color,
            clip,
        ),
        Command::Image {
            image,
            bounds,
            fit,
            opacity,
            radius,
            clip: _,
        } => draw_image(
            buffer,
            framebuffer_width,
            framebuffer_height,
            image,
            *bounds,
            clip,
            *fit,
            *opacity,
            *radius,
            display_scale,
            image_cache,
        ),
    }
}

pub fn raster_damage(
    commands: &[Vec<Command<'_>>; 16],
    prepared: &[PreparedCommand],
    damage: &[Rect],
    buffer: &mut [u32],
    framebuffer_width: usize,
    framebuffer_height: usize,
    display_scale: f32,
    fonts: &[fontdue::Font],
    font_bitmaps: &mut FxHashMap<usize, FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    image_cache: &mut FxHashMap<ImageKey, ImageEntry>,
) {
    for prepared in prepared {
        let command = &commands[prepared.layer][prepared.index];
        for region in damage {
            if prepared.bounds.intersects(*region) {
                draw_command(
                    command,
                    *region,
                    buffer,
                    framebuffer_width,
                    framebuffer_height,
                    display_scale,
                    fonts,
                    font_bitmaps,
                    image_cache,
                );
            }
        }
    }
}
