use crate::*;
use std::hash::{Hash, Hasher};

pub const TILE_SIZE: usize = 64;
const FULL_REDRAW_PERCENT: usize = 60;
const MAX_DAMAGE_RECTS: usize = 128;
const NO_DAMAGE: u16 = u16::MAX;
pub const MAX_TILE_LOOKUP: usize = 8;
pub const TILE_LOOKUP_MIN: usize = 12;

/// Hash seed (FNV-1a 64-bit offset basis).
const HASH_SEED: u64 = 0xcbf2_9ce4_8422_2325;
/// Multiplier for mixing (2^64 / φ, truncated).
const HASH_MIX: u64 = 0x9e37_79b9_7f4a_7c15;
/// Separates byte slices of different lengths after word-at-a-time hashing.
const BYTE_LEN_TAG: u64 = 0x6c62_79f5_aa2d_4f1b;
/// Seed tag so clear-color hashes don't collide with empty tiles.
const CLEAR_HASH_TAG: u64 = 0xff;

struct MixHasher(u64);

impl MixHasher {
    fn new() -> Self {
        Self(HASH_SEED)
    }
}

impl Hasher for MixHasher {
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in chunks.by_ref() {
            let v = u64::from_le_bytes(chunk.try_into().unwrap());
            self.0 = mix(self.0, v);
        }
        for &b in chunks.remainder() {
            self.0 = mix(self.0, b as u64);
        }
        self.0 = mix(self.0, BYTE_LEN_TAG ^ bytes.len() as u64);
    }

    fn write_u8(&mut self, i: u8) {
        self.0 = mix(self.0, i as u64);
    }

    fn write_u32(&mut self, i: u32) {
        self.0 = mix(self.0, i as u64);
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = mix(self.0, i);
    }

    fn write_usize(&mut self, i: usize) {
        self.0 = mix(self.0, i as u64);
    }

    fn write_i32(&mut self, i: i32) {
        // Match prior field hashing: reinterpret bits as unsigned, no sign-extend.
        self.0 = mix(self.0, i as u32 as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreparedCommand {
    pub layer: usize,
    pub index: usize,
    pub bounds: Rect,
}

#[derive(Debug, Default)]
pub struct RenderCache {
    current: Vec<u64>,
    previous: Vec<u64>,
    prepared: Vec<PreparedCommand>,
    damage: Vec<Rect>,
    tile_damage: Vec<u16>,
    cols: usize,
    rows: usize,
    width: usize,
    height: usize,
    force_full: bool,
}

impl RenderCache {
    pub fn invalidate(&mut self) {
        self.force_full = true;
    }

    /// Hash the command list into tiles. Returns true if any damage must be redrawn.
    pub fn update(
        &mut self,
        commands: &[Vec<Command<'_>>; 16],
        scale: f32,
        width: usize,
        height: usize,
        clear: u32,
    ) -> bool {
        let cols = width.div_ceil(TILE_SIZE);
        let rows = height.div_ceil(TILE_SIZE);
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.cols = cols;
            self.rows = rows;
            let len = cols.saturating_mul(rows);
            self.current.resize(len, 0);
            self.previous.resize(len, 0);
            self.tile_damage.resize(len, NO_DAMAGE);
            self.force_full = true;
        }

        let bg = mix(CLEAR_HASH_TAG, clear as u64);
        self.current.fill(bg);
        self.prepared.clear();
        self.damage.clear();

        for (layer, cmds) in commands.iter().enumerate() {
            for (index, command) in cmds.iter().enumerate() {
                let bounds = command_bounds(command, scale, width, height);
                if bounds.is_empty() || cols == 0 || rows == 0 {
                    continue;
                }
                let mut hasher = MixHasher::new();
                layer.hash(&mut hasher);
                command.hash(&mut hasher);
                let hash = hasher.finish();
                let x0 = bounds.x.max(0) as usize / TILE_SIZE;
                let y0 = bounds.y.max(0) as usize / TILE_SIZE;
                let x1 = (bounds.right().max(1) as usize - 1) / TILE_SIZE;
                let y1 = (bounds.bottom().max(1) as usize - 1) / TILE_SIZE;
                for y in y0..=y1.min(rows - 1) {
                    for x in x0..=x1.min(cols - 1) {
                        let cell = &mut self.current[x + y * cols];
                        *cell = mix(*cell, hash);
                    }
                }
                self.prepared.push(PreparedCommand { layer, index, bounds });
            }
        }

        self.build_damage(cols, rows);
        !self.damage.is_empty()
    }

    pub fn prepared(&self) -> &[PreparedCommand] {
        &self.prepared
    }

    pub fn damage(&self) -> &[Rect] {
        &self.damage
    }

    pub fn finish(&mut self) {
        std::mem::swap(&mut self.current, &mut self.previous);
        self.force_full = false;
        self.prepared.clear();
    }

    pub fn damage_indices(&self, bounds: Rect, out: &mut [u16; MAX_TILE_LOOKUP]) -> Option<usize> {
        if self.cols == 0 || self.rows == 0 || self.tile_damage.is_empty() || bounds.is_empty() {
            return Some(0);
        }
        let x0 = bounds.x.max(0) as usize / TILE_SIZE;
        let y0 = bounds.y.max(0) as usize / TILE_SIZE;
        let x1 = ((bounds.right().max(1) as usize - 1) / TILE_SIZE).min(self.cols - 1);
        let y1 = ((bounds.bottom().max(1) as usize - 1) / TILE_SIZE).min(self.rows - 1);
        if x0 > x1 || y0 > y1 {
            return Some(0);
        }

        let mut len = 0;
        for y in y0..=y1 {
            let row = y * self.cols;
            for x in x0..=x1 {
                let d = self.tile_damage[x + row];
                if d == NO_DAMAGE || out[..len].contains(&d) {
                    continue;
                }
                if len == MAX_TILE_LOOKUP {
                    return None;
                }
                out[len] = d;
                len += 1;
            }
        }
        Some(len)
    }

    fn build_damage(&mut self, cols: usize, rows: usize) {
        self.tile_damage.fill(NO_DAMAGE);
        if self.current.is_empty() || self.width == 0 || self.height == 0 {
            return;
        }

        if self.force_full {
            self.full_damage();
            return;
        }

        let mut dirty = 0usize;
        for i in 0..self.current.len() {
            dirty += usize::from(self.current[i] != self.previous[i]);
        }
        if dirty == 0 {
            return;
        }
        if dirty.saturating_mul(100) >= self.current.len().saturating_mul(FULL_REDRAW_PERCENT) {
            self.full_damage();
            return;
        }

        for y in 0..rows {
            let mut x = 0;
            while x < cols {
                let i = x + y * cols;
                if self.current[i] == self.previous[i] {
                    x += 1;
                    continue;
                }
                let start = x;
                x += 1;
                while x < cols && self.current[x + y * cols] != self.previous[x + y * cols] {
                    x += 1;
                }
                let px = (start * TILE_SIZE) as i32;
                let py = (y * TILE_SIZE) as i32;
                let right = (x * TILE_SIZE).min(self.width) as i32;
                let bottom = ((y + 1) * TILE_SIZE).min(self.height) as i32;
                let width = right - px;

                // Grow a run upward when the same horizontal span sits on the row above.
                let mut merged = None;
                for (d, rect) in self.damage.iter_mut().enumerate().rev() {
                    if rect.bottom() < py {
                        break;
                    }
                    if rect.x == px && rect.width == width && rect.bottom() == py {
                        rect.height = bottom - rect.y;
                        merged = Some(d);
                        break;
                    }
                }
                let d = match merged {
                    Some(d) => d,
                    None => {
                        self.damage.push(Rect::new(px, py, width, bottom - py));
                        if self.damage.len() > MAX_DAMAGE_RECTS {
                            self.full_damage();
                            return;
                        }
                        self.damage.len() - 1
                    }
                };
                for tx in start..x {
                    self.tile_damage[tx + y * cols] = d as u16;
                }
            }
        }
    }

    fn full_damage(&mut self) {
        self.damage.clear();
        self.damage.push(Rect::new(0, 0, self.width as i32, self.height as i32));
        self.tile_damage.fill(0);
    }
}

/// Order-dependent fold used for both command hashing and per-tile accumulation.
#[inline]
fn mix(h: u64, v: u64) -> u64 {
    h.wrapping_mul(HASH_MIX).wrapping_add(v).rotate_left(27)
}

#[inline]
pub fn command_bounds(command: &Command<'_>, scale_factor: f32, fb_w: usize, fb_h: usize) -> Rect {
    if scale_factor == 1.0 {
        let b = match command {
            Command::Rect { bounds, .. } | Command::RectStroke { bounds, .. } => *bounds,
            Command::Triangle { a, b, c, .. } => Rect::from_xyxy(
                a.0.min(b.0).min(c.0).saturating_sub(1),
                a.1.min(b.1).min(c.1).saturating_sub(1),
                a.0.max(b.0).max(c.0).saturating_add(2),
                a.1.max(b.1).max(c.1).saturating_add(2),
            ),
            Command::Text { bounds, size, .. } => {
                let pad = ((*size) / 2).max(4) as i32;
                Rect::new(
                    bounds.x.saturating_sub(pad),
                    bounds.y.saturating_sub(pad),
                    bounds.width.saturating_add(pad * 2),
                    bounds.height.saturating_add(pad * 2),
                )
            }
            Command::Image {
                image,
                bounds,
                fit,
                radius,
                ..
            } => {
                let fitted = if bounds.is_empty() {
                    *bounds
                } else {
                    place(image.width, image.height, *bounds, *fit).1
                }
                .intersection(*bounds);
                if *radius == 0 {
                    fitted
                } else {
                    Rect::new(
                        fitted.x.saturating_sub(1),
                        fitted.y.saturating_sub(1),
                        fitted.width.saturating_add(2),
                        fitted.height.saturating_add(2),
                    )
                }
            }
        };
        return b.intersection(command.clip()).clamp_to_size(fb_w as i32, fb_h as i32);
    }

    let b = match command {
        Command::Rect { bounds, .. } | Command::RectStroke { bounds, .. } => bounds.scale(scale_factor),
        Command::Triangle { a, b, c, .. } => {
            let ax = scale_i32(a.0, scale_factor);
            let ay = scale_i32(a.1, scale_factor);
            let bx = scale_i32(b.0, scale_factor);
            let by = scale_i32(b.1, scale_factor);
            let cx = scale_i32(c.0, scale_factor);
            let cy = scale_i32(c.1, scale_factor);
            Rect::from_xyxy(
                ax.min(bx).min(cx).saturating_sub(1),
                ay.min(by).min(cy).saturating_sub(1),
                ax.max(bx).max(cx).saturating_add(2),
                ay.max(by).max(cy).saturating_add(2),
            )
        }
        Command::Text { bounds, size, .. } => {
            let pad = (scale(*size, scale_factor) / 2).max(4) as i32;
            let b = bounds.scale(scale_factor);
            Rect::new(
                b.x.saturating_sub(pad),
                b.y.saturating_sub(pad),
                b.width.saturating_add(pad * 2),
                b.height.saturating_add(pad * 2),
            )
        }
        Command::Image {
            image,
            bounds,
            fit,
            radius,
            ..
        } => {
            let fitted = if bounds.is_empty() {
                *bounds
            } else {
                place(image.width, image.height, *bounds, *fit).1
            }
            .intersection(*bounds)
            .scale(scale_factor);
            if *radius == 0 {
                fitted
            } else {
                Rect::new(
                    fitted.x.saturating_sub(1),
                    fitted.y.saturating_sub(1),
                    fitted.width.saturating_add(2),
                    fitted.height.saturating_add(2),
                )
            }
        }
    };
    b.intersection(command.clip().scale(scale_factor)).clamp_to_size(fb_w as i32, fb_h as i32)
}
