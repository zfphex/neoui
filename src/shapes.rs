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

#[inline(always)]
pub fn linear_to_gamma(v: f32) -> u8 {
    (v.clamp(0.0, 1.0).sqrt() * 255.0 + 0.5) as u8
}

#[inline(always)]
pub fn scale(value: usize, factor: f32) -> usize {
    if factor == 1.0 {
        return value;
    }
    (value as f32 * factor + 0.5) as usize
}

#[inline(always)]
pub fn scale_i32(value: i32, factor: f32) -> i32 {
    if factor == 1.0 {
        return value;
    }
    let v = value as f32 * factor;
    if v >= 0.0 {
        (v + 0.5) as i32
    } else {
        (v - 0.5) as i32
    }
}

/// Returns `None` if fully clipped.
#[inline]
pub fn visible_rect(shape: Rect, clip: Rect, fb_w: i32, fb_h: i32) -> Option<Rect> {
    let r = shape.intersection(clip).clamp_to_size(fb_w, fb_h);
    if r.is_empty() { None } else { Some(r) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
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
    let src = color_linear(color);
    let fill_color = color & 0x00FF_FFFF;
    for py in y0 as usize..y1 as usize {
        let start = py * window_width + x0;
        if let Some(slice) = buffer.get_mut(start..start + len) {
            fill_span_color(slice, fill_color, src);
        }
    }
}

/// Fill a contiguous run with `color`, blending per-pixel when it is translucent.
/// `src` must be `color_linear(color)`.
#[inline]
fn fill_span_color(slice: &mut [u32], fill_color: u32, src: (f32, f32, f32, f32)) {
    if src.3 >= 0.999 {
        slice.fill(fill_color);
    } else if src.3 > 0.0 {
        for bg in slice {
            blend_gamma2(bg, src, src.3);
        }
    }
}

#[inline]
fn solid_fill_rect(buffer: &mut [u32], window_width: usize, area: Rect, color: u32) {
    solid_fill_span(buffer, window_width, area.x, area.right(), area.y, area.bottom(), color);
}

#[inline(always)]
pub(crate) fn rounded_axis(position: f32, centre: f32, inset: f32) -> f32 {
    (position - centre).abs() - inset
}

#[inline(always)]
pub(crate) fn rounded_coverage(distance_x: f32, distance_y: f32, radius: f32) -> f32 {
    (0.5 - rounded_rect_sdf(distance_x, distance_y, radius)).clamp(0.0, 1.0)
}

#[inline]
fn color_linear(color: u32) -> (f32, f32, f32, f32) {
    (
        GAMMA_TO_LINEAR[((color >> 16) & 0xFF) as usize],
        GAMMA_TO_LINEAR[((color >> 8) & 0xFF) as usize],
        GAMMA_TO_LINEAR[(color & 0xFF) as usize],
        alpha(color) as f32 / 255.0,
    )
}

#[inline]
fn lerp_color(from: u32, to: u32, t: f32) -> u32 {
    let (fr, fg, fb) = split_f32(from);
    let (tr, tg, tb) = split_f32(to);
    let r = (fr + (tr - fr) * t + 0.5) as u32;
    let g = (fg + (tg - fg) * t + 0.5) as u32;
    let b = (fb + (tb - fb) * t + 0.5) as u32;
    r << 16 | g << 8 | b
}

#[inline]
fn blend_gamma2(bg: &mut u32, src: (f32, f32, f32, f32), a: f32) {
    let a = a.clamp(0.0, 1.0);
    let bg_val = *bg;
    let inv = 1.0 - a;

    let out_r = src.0 * a + GAMMA_TO_LINEAR[((bg_val >> 16) & 0xFF) as usize] * inv;
    let out_g = src.1 * a + GAMMA_TO_LINEAR[((bg_val >> 8) & 0xFF) as usize] * inv;
    let out_b = src.2 * a + GAMMA_TO_LINEAR[(bg_val & 0xFF) as usize] * inv;

    let r = linear_to_gamma(out_r) as u32;
    let g = linear_to_gamma(out_g) as u32;
    let b = linear_to_gamma(out_b) as u32;
    *bg = (r << 16) | (g << 8) | b;
}

#[inline]
fn apply_coverage(bg: &mut u32, color: u32, src: (f32, f32, f32, f32), coverage: f32) {
    let a = coverage.clamp(0.0, 1.0) * src.3;
    if a >= 0.999 {
        *bg = color & 0x00FF_FFFF;
    } else if a > 0.0 {
        blend_gamma2(bg, src, a);
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

#[allow(unused)]
#[inline(never)]
pub fn draw_rect_fill_wip(
    buffer: &mut [u32],
    bounds: Rect,
    window_width: usize,
    window_height: usize,
    radius: usize,
    color: u32,
    clip: Rect,
) {
    mini::profile!();
    use std::simd::{StdFloat, num::SimdFloat, u32x4, u32x8, u32x16};
    // let (x, y, w, h) = (bounds.x as f32, bounds.y as f32, bounds.width as f32, bounds.height as f32);
    let half_w = bounds.width as f32 * 0.5;
    let half_h = bounds.height as f32 * 0.5;
    let r = (radius as f32).min(half_w).min(half_h);
    let aa_width = 1.0f32;
    let s = aa_width.max(0.001);
    let (bx, by) = (half_w - r, half_h - r);
    let (cx, cy) = (bounds.x as f32 + half_w, bounds.y as f32 + half_h);

    let min_x = (cx - half_w - s).floor() as i32;
    let max_x = (cx + half_w + s).ceil() as i32;
    let min_y = (cy - half_h - s).floor() as i32;
    let max_y = (cy + half_h + s).ceil() as i32;

    let min_x = min_x.clamp(clip.x.max(0), clip.right()) as usize;
    let max_x = max_x.clamp(clip.x.max(0), clip.right()) as usize;
    let min_y = min_y.clamp(clip.y.max(0), clip.bottom()) as usize;
    let max_y = max_y.clamp(clip.y.max(0), clip.bottom()) as usize;

    type Vec8 = std::simd::f32x16;
    const LANES: usize = 16;
    type VecU8 = u32x16;

    // let lane_offsets = Vec8::from_array([0.5, 1.5, 2.5, 3.5]);
    // let lane_offsets = Vec8::from_array([0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5]);
    let lane_offsets = Vec8::from_array([
        0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5, 10.5, 11.5, 12.5, 13.5, 14.5, 15.5,
    ]);

    let cx_vec = Vec8::splat(cx);
    let inner_bx_vec = Vec8::splat(bx);
    let r_vec = Vec8::splat(r);
    let s_vec = Vec8::splat(s);
    let inv_s = 1.0 / s;
    let inv_s_vec = Vec8::splat(inv_s);
    let zero = Vec8::splat(0.0);
    let half = Vec8::splat(0.5);
    let one = Vec8::splat(1.0);
    let color_vec = Vec8::splat(color as f32);
    let len_x = max_x - min_x;
    let simd_chunks = len_x / LANES;
    let x_simd_end = min_x + simd_chunks * LANES;

    let ca_vec = Vec8::splat(((color >> 24) & 0xFF) as f32);
    let cr_vec = Vec8::splat(((color >> 16) & 0xFF) as f32);
    let cg_vec = Vec8::splat(((color >> 8) & 0xFF) as f32);
    let cb_vec = Vec8::splat((color & 0xFF) as f32);

    let shift_24 = VecU8::splat(24);
    let shift_16 = VecU8::splat(16);
    let shift_8 = VecU8::splat(8);

    for y in min_y..max_y {
        let row_start = y * window_width;
        let py = y as f32 + 0.5;
        let dy = (py - cy).abs() - by;

        let dy_vec = Vec8::splat(dy);
        let exty_vec = dy_vec.simd_max(zero);

        let exty_sq = exty_vec * exty_vec;

        let row = &mut buffer[row_start..row_start + window_width];
        let row_span = &mut row[min_x..max_x];
        let mut chunks = row_span.chunks_exact_mut(LANES);

        for (chunk, out) in (&mut chunks).enumerate() {
            let base_x = (min_x + chunk * LANES) as f32;
            let px_vec = Vec8::splat(base_x) + lane_offsets;
            let dx_vec = (px_vec - cx_vec).abs() - inner_bx_vec;

            let extx_vec = dx_vec.simd_max(zero);
            let exterior_dist = (extx_vec * extx_vec + exty_sq).sqrt();
            let interior_dist = dx_vec.simd_max(dy_vec).simd_min(zero);

            let sdf = exterior_dist + interior_dist - r_vec;
            let alpha = (half - sdf / s_vec).simd_clamp(zero, one);
            let alpha = (half - sdf * inv_s_vec).simd_max(zero).simd_min(one);

            let a_u32 = (ca_vec * alpha).cast::<u32>();
            let r_u32 = (cr_vec * alpha).cast::<u32>();
            let g_u32 = (cg_vec * alpha).cast::<u32>();
            let b_u32 = (cb_vec * alpha).cast::<u32>();

            let pixel_u32s = (a_u32 << shift_24) | (r_u32 << shift_16) | (g_u32 << shift_8) | b_u32;
            let dst_offset = row_start + min_x + chunk * LANES;
            let local_offset = min_x + chunk * LANES;
            pixel_u32s.copy_to_slice(out);
        }

        // for x in x_simd_end..max_x {
        //     let px = x as f32 + 0.5;
        //     let dx = (px - cx).abs() - bx;

        //     let extx = dx.max(0.0);
        //     let exty = dy.max(0.0);
        //     let exterior_dist = (extx * extx + exty * exty).sqrt();
        //     let interior_dist = dx.max(dy).min(0.0);

        //     let sdf = exterior_dist + interior_dist - r;
        //     let alpha = (0.5 - sdf / s).clamp(0.0, 1.0);

        //     let dst_offset = row_start + x;

        //     let a = (((color >> 24) & 0xFF) as f32 * alpha) as u32;
        //     let r = (((color >> 16) & 0xFF) as f32 * alpha) as u32;
        //     let g = (((color >> 8) & 0xFF) as f32 * alpha) as u32;
        //     let b = ((color & 0xFF) as f32 * alpha) as u32;

        //     buffer[dst_offset as usize] = (a << 24) | (r << 16) | (g << 8) | b;
        // }
    }
}

#[allow(unused)]
#[rustfmt::skip]
pub fn draw_rect_fill_scalar(
    buffer: &mut [u32],
    bounds: Rect,
    window_width: usize,
    window_height: usize,
    radius: usize,
    color: u32,
    clip: Rect,
) {
    mini::profile!();
    let (x, y, w, h) = (bounds.x as f32, bounds.y as f32, bounds.width as f32, bounds.height as f32);
    let half_w = bounds.width as f32 * 0.5;
    let half_h = bounds.height as f32 * 0.5;
    let r = (radius as f32).min(half_w).min(half_h);
    let aa_width = 1.0f32;
    let s = aa_width.max(0.001);
    let (bx, by) = (half_w - r, half_h - r);
    let (cx, cy) = (x + half_w, y + half_h);

    let min_x = (cx - half_w - s).floor() as i32;
    let max_x = (cx + half_w + s).ceil() as i32;
    let min_y = (cy - half_h - s).floor() as i32;
    let max_y = (cy + half_h + s).ceil() as i32;

    let min_x = min_x.clamp(clip.x.max(0), clip.right()) as usize;
    let max_x = max_x.clamp(clip.x.max(0), clip.right()) as usize;
    let min_y = min_y.clamp(clip.y.max(0), clip.bottom()) as usize;
    let max_y = max_y.clamp(clip.y.max(0), clip.bottom()) as usize;

    for y in min_y..max_y {
        let row_start = y * window_width;
        let py = y as f32 + 0.5;
        let dy = (py - cy).abs() - by;

        for x in min_x..max_x {
            let px = x as f32 + 0.5;
            let dx = (px - cx).abs() - bx;

            let extx = dx.max(0.0);
            let exty = dy.max(0.0);
            let exterior_dist = (extx * extx + exty * exty).sqrt();
            let interior_dist = dx.max(dy).min(0.0);

            let sdf = exterior_dist + interior_dist - r;
            let alpha = (0.5 - sdf / s).clamp(0.0, 1.0);

            let dst_offset = row_start + x;

            let a = (((color >> 24) & 0xFF) as f32 * alpha) as u32;
            let r = (((color >> 16) & 0xFF) as f32 * alpha) as u32;
            let g = (((color >> 8) & 0xFF) as f32 * alpha) as u32;
            let b = ((color & 0xFF) as f32 * alpha) as u32;

            buffer[dst_offset as usize] = (a << 24) | (r << 16) | (g << 8) | b;
        }
    }
}

pub fn draw_rect_fill(
    buffer: &mut [u32],
    bounds: Rect,
    window_width: usize,
    window_height: usize,
    radius: usize,
    color: u32,
    gradient: Option<(u32, u32)>,
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

    if radius == 0 && gradient.is_none() {
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

    if let Some((from, to)) = gradient {
        let (from_a, to_a) = (alpha(from) as f32 / 255.0, alpha(to) as f32 / 255.0);

        for py in min_y..max_y {
            let row_start = py * window_width;
            let t = ((py as f32 + 0.5 - y as f32) / height as f32).clamp(0.0, 1.0);
            let color = lerp_color(from, to, t);
            let (r, g, b, _) = color_linear(color);
            let src = (r, g, b, from_a + (to_a - from_a) * t);

            if py >= y_top_safe && py < y_bottom_safe {
                if let Some(slice) = buffer.get_mut(row_start + min_x..row_start + max_x) {
                    fill_span_color(slice, color, src);
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

                fill_span_color(mid_slice, color, src);

                for (i, bg) in right_slice.iter_mut().enumerate() {
                    let px = right_limit + i;
                    let dx = (px as f32 + 0.5 - cx).abs() - half_w + r_f32;
                    apply_coverage(bg, color, src, 0.5 - rounded_rect_sdf(dx, dy, r_f32));
                }
            }
        }

        return;
    }

    for py in min_y..max_y {
        let row_start = py * window_width;

        if py >= y_top_safe && py < y_bottom_safe {
            if let Some(slice) = buffer.get_mut(row_start + min_x..row_start + max_x) {
                fill_span_color(slice, color, src);
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

            fill_span_color(mid_slice, color, src);

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

            let coverage = cov0 * cov1 * cov2;

            // Only fully-covered pixels of a fully-opaque color take the solid run;
            // a translucent color always blends so it composites over the background.
            if coverage >= 0.999 && src.3 >= 0.999 {
                if solid_start == window_width {
                    solid_start = x;
                }
                solid_end = x + 1;
            } else {
                let a = coverage * src.3;
                if a > 0.0 {
                    let idx = y as usize * window_width + x;
                    if let Some(bg) = buffer.get_mut(idx) {
                        blend_gamma2(bg, src, a);
                    }
                }
            }
        }

        if solid_start < solid_end {
            let start_idx = y as usize * window_width + solid_start;
            let end_idx = y as usize * window_width + solid_end;
            if let Some(slice) = buffer.get_mut(start_idx..end_idx) {
                slice.fill(color & 0x00FF_FFFF);
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

fn glyph<'a>(
    cache: &'a mut FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    fonts: &[fontdue::Font],
    fallbacks: &[usize],
    font_id: usize,
    ch: char,
    font_size: usize,
) -> &'a (fontdue::Metrics, Vec<u8>) {
    cache.entry((font_id, ch, font_size)).or_insert_with(|| {
        let glyph_font = if fallbacks.is_empty() || fonts[font_id].lookup_glyph_index(ch) != 0 {
            font_id
        } else {
            fallbacks
                .iter()
                .copied()
                .find(|f| fonts[*f].lookup_glyph_index(ch) != 0)
                .unwrap_or(font_id)
        };
        let (metrics, mut bitmap) = fonts[glyph_font].rasterize_subpixel(ch, font_size as f32);
        apply_lcd_filter(&mut bitmap, metrics.width, metrics.height);
        (metrics, bitmap)
    })
}

pub fn draw_text(
    text: &str,
    fonts: &[fontdue::Font],
    font_id: usize,
    fallbacks: &[usize],
    bounds: Rect,
    font_size: usize,
    line_height: Option<usize>,
    alignment: Alignment,
    window_width: usize,
    buffer: &mut [u32],
    color: u32,
    cache: &mut FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    clip: Rect,
) -> Rect {
    if text.is_empty() || font_size == 0 || window_width == 0 {
        return Rect::default();
    }

    let font = &fonts[font_id];
    let size = font_size as f32;
    let x_start = bounds.x as f32;
    let y_start = bounds.y as f32;

    let line_metrics = font.horizontal_line_metrics(size).unwrap();
    let line_step = line_height.map_or(line_metrics.new_line_size, |h| h as f32);
    let baseline_offset = line_metrics.ascent + (line_step - line_metrics.ascent + line_metrics.descent) / 2.0;
    // A single line always spans the whole block, and flush-left lines never move.
    let align_lines =
        !matches!(alignment, Alignment::Left | Alignment::TopLeft | Alignment::BottomLeft) && text.contains('\n');

    let (txt_r, txt_g, txt_b, txt_a) = split(color);
    let is_opaque_text = txt_a >= 254;
    let text_color_rgb = ((txt_r as u32) << 16) | ((txt_g as u32) << 8) | (txt_b as u32);
    let txt_r_lin = GAMMA_TO_LINEAR[txt_r as usize];
    let txt_g_lin = GAMMA_TO_LINEAR[txt_g as usize];
    let txt_b_lin = GAMMA_TO_LINEAR[txt_b as usize];
    let txt_a = txt_a as f32 / 255.0;

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
        if align_lines {
            let mut line_width = 0.0;
            for ch in line.chars() {
                line_width += glyph(cache, fonts, fallbacks, font_id, ch, font_size).0.advance_width;
            }
            let slack = bounds.width as f32 - line_width;
            glyph_x += match alignment {
                Alignment::Right | Alignment::TopRight | Alignment::BottomRight => slack,
                _ => slack / 2.0,
            };
        }
        let baseline_y = y_pos + baseline_offset;

        for ch in line.chars() {
            let (metrics, bitmap) = glyph(cache, fonts, fallbacks, font_id, ch, font_size);

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

                    for (i, bg) in buffer_row.iter_mut().enumerate() {
                        let mask_idx = i * 3;
                        let m_r = bitmap_row[mask_idx];
                        let m_g = bitmap_row[mask_idx + 1];
                        let m_b = bitmap_row[mask_idx + 2];

                        if m_r | m_g | m_b == 0 {
                            continue;
                        }

                        if m_r & m_g & m_b == 255 && is_opaque_text {
                            *bg = text_color_rgb;
                            continue;
                        }

                        const INV_255: f32 = 1.0 / 255.0;
                        let mask_r = m_r as f32 * INV_255 * txt_a;
                        let mask_g = m_g as f32 * INV_255 * txt_a;
                        let mask_b = m_b as f32 * INV_255 * txt_a;

                        let (bg_r, bg_g, bg_b, _) = split(*bg);

                        let bg_r_lin = GAMMA_TO_LINEAR[bg_r as usize];
                        let bg_g_lin = GAMMA_TO_LINEAR[bg_g as usize];
                        let bg_b_lin = GAMMA_TO_LINEAR[bg_b as usize];

                        let out_r_lin = txt_r_lin * mask_r + bg_r_lin * (1.0 - mask_r);
                        let out_g_lin = txt_g_lin * mask_g + bg_g_lin * (1.0 - mask_g);
                        let out_b_lin = txt_b_lin * mask_b + bg_b_lin * (1.0 - mask_b);

                        let out_r = linear_to_gamma(out_r_lin) as u32;
                        let out_g = linear_to_gamma(out_g_lin) as u32;
                        let out_b = linear_to_gamma(out_b_lin) as u32;

                        *bg = (out_r << 16) | (out_g << 8) | out_b;
                    }
                }
            }

            glyph_x += metrics.advance_width;

            if glyph_x.round() as usize >= window_width {
                break;
            }
        }
        y_pos += line_step;
    }

    let x0 = bounds.x;
    let y0 = bounds.y;
    Rect {
        x: x0,
        y: y0,
        width: if max_x as i32 >= x0 { max_x as i32 + 1 - x0 } else { 0 },
        height: if max_y as i32 >= y0 { max_y as i32 + 1 - y0 } else { 0 },
    }
}

pub fn measure_text(
    text: &str,
    fonts: &[fontdue::Font],
    font_id: usize,
    fallbacks: &[usize],
    font_size: usize,
    line_height: Option<usize>,
    metrics: &mut FxHashMap<(usize, char, usize), fontdue::Metrics>,
) -> Rect {
    if text.is_empty() || font_size == 0 {
        return Rect::default();
    }

    let font = &fonts[font_id];
    let size = font_size as f32;
    let line_metrics = font.horizontal_line_metrics(size).unwrap();

    let mut max_width = 0.0f32;
    let mut current_width = 0.0f32;
    let mut lines: i32 = 1;

    for ch in text.chars() {
        if ch == '\n' {
            max_width = max_width.max(current_width);
            current_width = 0.0;
            lines += 1;
            continue;
        }

        let metrics = metrics.entry((font_id, ch, font_size)).or_insert_with(|| {
            let glyph_font = if fallbacks.is_empty() || font.lookup_glyph_index(ch) != 0 {
                font_id
            } else {
                fallbacks
                    .iter()
                    .copied()
                    .find(|f| fonts[*f].lookup_glyph_index(ch) != 0)
                    .unwrap_or(font_id)
            };
            fonts[glyph_font].metrics(ch, size)
        });

        current_width += metrics.advance_width;
    }

    max_width = max_width.max(current_width);

    Rect {
        x: 0,
        y: 0,
        width: max_width.round() as i32,
        height: match line_height {
            Some(line_height) => lines * line_height as i32,
            None => (lines as f32 * line_metrics.new_line_size).round() as i32,
        },
    }
}

#[inline(always)]
pub fn fill_u32_fast(slice: &mut [u32], color: u32) {
    use std::simd::prelude::*;
    let (prefix, vectors, suffix) = slice.as_simd_mut::<8>();
    prefix.fill(color);
    let vec_color = u32x8::splat(color);
    vectors.fill(vec_color);
    suffix.fill(color);
}

pub fn clear_damage(buffer: &mut [u32], framebuffer_width: usize, damage: &[Rect], color: u32) {
    let fill_color = color & 0x00FF_FFFF;
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
            if let Some(slice) = buffer.get_mut(start..start + width) {
                fill_u32_fast(slice, fill_color);
            }
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
    fallbacks: &[usize],
    font_bitmaps: &mut FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    image_cache: &mut ImageCache,
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
            gradient,
            clip: _,
        } => draw_rect_fill(
            buffer,
            bounds.scale(display_scale),
            framebuffer_width,
            framebuffer_height,
            scale(*radius, display_scale),
            *color,
            *gradient,
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
            line_height,
            alignment,
            clip: _,
        } => {
            draw_text(
                text,
                fonts,
                *font_id,
                fallbacks,
                bounds.scale(display_scale),
                scale(*size, display_scale),
                line_height.map(|h| scale(h, display_scale)),
                *alignment,
                framebuffer_width,
                buffer,
                *color,
                font_bitmaps,
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
    cache: &RenderCache,
    buffer: &mut [u32],
    framebuffer_width: usize,
    framebuffer_height: usize,
    display_scale: f32,
    fonts: &[fontdue::Font],
    fallbacks: &[usize],
    font_bitmaps: &mut FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    image_cache: &mut ImageCache,
) {
    let damage = cache.damage();

    if damage.len() <= TILE_LOOKUP_MIN {
        for prepared in cache.prepared() {
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
                        fallbacks,
                        font_bitmaps,
                        image_cache,
                    );
                }
            }
        }
        return;
    }

    let mut indices = [0u16; MAX_TILE_LOOKUP];
    for prepared in cache.prepared() {
        let command = &commands[prepared.layer][prepared.index];
        match cache.damage_indices(prepared.bounds, &mut indices) {
            Some(len) => {
                for &d in &indices[..len] {
                    draw_command(
                        command,
                        damage[d as usize],
                        buffer,
                        framebuffer_width,
                        framebuffer_height,
                        display_scale,
                        fonts,
                        fallbacks,
                        font_bitmaps,
                        image_cache,
                    );
                }
            }
            None => {
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
                            fallbacks,
                            font_bitmaps,
                            image_cache,
                        );
                    }
                }
            }
        }
    }
}
