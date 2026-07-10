use crate::*;
use rustc_hash::FxHashMap;

pub const TILE_SIZE: usize = 64;
const FULL_REDRAW_PERCENT: usize = 60;
const MAX_DAMAGE_RECTS: usize = 128;
const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalRect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl PhysicalRect {
    pub const fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self { x0, y0, x1, y1 }
    }

    pub fn from_rect(rect: Rect) -> Self {
        Self::new(
            rect.x.min(i32::MAX as usize) as i32,
            rect.y.min(i32::MAX as usize) as i32,
            rect.x.saturating_add(rect.width).min(i32::MAX as usize) as i32,
            rect.y.saturating_add(rect.height).min(i32::MAX as usize) as i32,
        )
    }

    pub const fn is_empty(self) -> bool {
        self.x0 >= self.x1 || self.y0 >= self.y1
    }

    pub fn intersection(self, other: Self) -> Self {
        Self::new(
            self.x0.max(other.x0),
            self.y0.max(other.y0),
            self.x1.min(other.x1),
            self.y1.min(other.y1),
        )
    }

    pub fn intersects(self, other: Self) -> bool {
        !self.intersection(other).is_empty()
    }

    pub fn clamp_to_framebuffer(self, width: usize, height: usize) -> Self {
        self.intersection(Self::new(
            0,
            0,
            width.min(i32::MAX as usize) as i32,
            height.min(i32::MAX as usize) as i32,
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreparedCommand {
    pub layer: usize,
    pub index: usize,
    pub bounds: PhysicalRect,
    pub hash: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheFrameStats {
    pub dirty_tiles: usize,
    pub damage_rects: usize,
    pub damaged_pixels: usize,
    pub full_redraw: bool,
}

#[derive(Debug)]
pub struct RenderCache {
    pub current: Vec<u64>,
    pub previous: Vec<u64>,
    pub dirty: Vec<bool>,
    pub damage: Vec<Rect>,
    pub prepared: Vec<PreparedCommand>,
    pub cols: usize,
    pub rows: usize,
    pub width: usize,
    pub height: usize,
    pub scale_bits: u64,
    pub force_full_redraw: bool,
    pub initialized: bool,
    pub stats: CacheFrameStats,
}

impl Default for RenderCache {
    fn default() -> Self {
        Self {
            current: Vec::new(),
            previous: Vec::new(),
            dirty: Vec::new(),
            damage: Vec::new(),
            prepared: Vec::new(),
            cols: 0,
            rows: 0,
            width: 0,
            height: 0,
            scale_bits: 0,
            force_full_redraw: true,
            initialized: false,
            stats: CacheFrameStats::default(),
        }
    }
}

impl RenderCache {
    pub fn invalidate(&mut self) {
        self.force_full_redraw = true;
    }

    pub fn begin_frame(&mut self, width: usize, height: usize, scale: f64, clear_color: u32) {
        let cols = width.div_ceil(TILE_SIZE);
        let rows = height.div_ceil(TILE_SIZE);
        let scale_bits = scale.to_bits();
        let changed =
            !self.initialized || self.width != width || self.height != height || self.scale_bits != scale_bits;

        if changed {
            self.width = width;
            self.height = height;
            self.cols = cols;
            self.rows = rows;
            self.scale_bits = scale_bits;
            let len = cols.saturating_mul(rows);
            self.current.resize(len, 0);
            self.previous.resize(len, 0);
            self.dirty.resize(len, false);
            self.force_full_redraw = true;
            self.initialized = true;
        }

        let mut background = Fnv1a::new();
        background.write_u8(0xff);
        background.write_u32(clear_color);
        self.current.fill(background.finish());
        self.prepared.clear();
        self.damage.clear();
        self.stats = CacheFrameStats::default();
    }

    pub fn add_command(&mut self, command: PreparedCommand) {
        let bounds = command.bounds.clamp_to_framebuffer(self.width, self.height);
        if bounds.is_empty() || self.cols == 0 || self.rows == 0 {
            return;
        }

        let x0 = bounds.x0 as usize / TILE_SIZE;
        let y0 = bounds.y0 as usize / TILE_SIZE;
        let x1 = (bounds.x1 as usize - 1) / TILE_SIZE;
        let y1 = (bounds.y1 as usize - 1) / TILE_SIZE;
        for y in y0..=y1.min(self.rows - 1) {
            for x in x0..=x1.min(self.cols - 1) {
                let cell = &mut self.current[x + y * self.cols];
                *cell = fnv_mix_u64(*cell, command.hash);
            }
        }
        self.prepared.push(PreparedCommand { bounds, ..command });
    }

    pub fn compute_damage(&mut self) -> &[Rect] {
        if self.current.is_empty() || self.width == 0 || self.height == 0 {
            self.stats.dirty_tiles = 0;
            self.stats.damage_rects = 0;
            return &self.damage;
        }

        let mut dirty_tiles = 0;
        for i in 0..self.current.len() {
            let dirty = self.force_full_redraw || self.current[i] != self.previous[i];
            self.dirty[i] = dirty;
            dirty_tiles += usize::from(dirty);
        }
        self.stats.dirty_tiles = dirty_tiles;

        if dirty_tiles == 0 {
            return &self.damage;
        }

        if dirty_tiles.saturating_mul(100) >= self.current.len().saturating_mul(FULL_REDRAW_PERCENT) {
            self.set_full_damage();
            return &self.damage;
        }

        for y in 0..self.rows {
            let mut x = 0;
            while x < self.cols {
                if !self.dirty[x + y * self.cols] {
                    x += 1;
                    continue;
                }

                let start = x;
                while x < self.cols && self.dirty[x + y * self.cols] {
                    x += 1;
                }
                let px = start * TILE_SIZE;
                let py = y * TILE_SIZE;
                let right = (x * TILE_SIZE).min(self.width);
                let bottom = ((y + 1) * TILE_SIZE).min(self.height);
                let mut merged = false;
                for rect in self.damage.iter_mut().rev() {
                    if rect.bottom() < py {
                        break;
                    }
                    if rect.x == px && rect.width == right - px && rect.bottom() == py {
                        rect.height = bottom - rect.y;
                        merged = true;
                        break;
                    }
                }
                if !merged {
                    self.damage.push(Rect::new(px, py, right - px, bottom - py));
                }
            }
        }

        if self.damage.len() > MAX_DAMAGE_RECTS {
            self.set_full_damage();
        } else {
            self.finish_stats(false);
        }
        &self.damage
    }

    fn set_full_damage(&mut self) {
        self.damage.clear();
        self.damage.push(Rect::new(0, 0, self.width, self.height));
        self.finish_stats(true);
    }

    fn finish_stats(&mut self, full_redraw: bool) {
        self.stats.damage_rects = self.damage.len();
        self.stats.damaged_pixels = self
            .damage
            .iter()
            .map(|rect| rect.width.saturating_mul(rect.height))
            .sum();
        self.stats.full_redraw = full_redraw;
    }

    pub fn take_damage(&mut self) -> Vec<Rect> {
        std::mem::take(&mut self.damage)
    }

    pub fn recycle_damage(&mut self, mut damage: Vec<Rect>) {
        damage.clear();
        self.damage = damage;
    }

    pub fn complete_frame(&mut self) {
        std::mem::swap(&mut self.current, &mut self.previous);
        self.force_full_redraw = false;
        self.prepared.clear();
    }
}

pub struct Fnv1a(u64);

impl Fnv1a {
    pub const fn new() -> Self {
        Self(FNV_OFFSET)
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= *byte as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.write(&[value]);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.write(&value.to_le_bytes());
    }

    pub fn write_usize(&mut self, value: usize) {
        self.write(&(value as u64).to_le_bytes());
    }

    pub const fn finish(&self) -> u64 {
        self.0
    }
}

fn fnv_mix_u64(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= byte as u64;
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

fn hash_rect(hasher: &mut Fnv1a, rect: Rect) {
    hasher.write_usize(rect.x);
    hasher.write_usize(rect.y);
    hasher.write_usize(rect.width);
    hasher.write_usize(rect.height);
}

pub fn command_hash(command: &Command<'_>, layer: usize) -> u64 {
    let mut hasher = Fnv1a::new();
    hasher.write_usize(layer);
    match command {
        Command::Rect {
            x,
            y,
            width,
            height,
            clip,
            color,
            radius,
        } => {
            hasher.write_u8(0);
            hasher.write_i32(*x);
            hasher.write_i32(*y);
            hasher.write_usize(*width);
            hasher.write_usize(*height);
            hash_rect(&mut hasher, *clip);
            hasher.write_u32(*color);
            hasher.write_usize(*radius);
        }
        Command::RectOutline {
            x,
            y,
            width,
            height,
            clip,
            color,
            radius,
            border_thickness,
            border_sides,
        } => {
            hasher.write_u8(1);
            hasher.write_i32(*x);
            hasher.write_i32(*y);
            hasher.write_usize(*width);
            hasher.write_usize(*height);
            hash_rect(&mut hasher, *clip);
            hasher.write_u32(*color);
            hasher.write_usize(*radius);
            hasher.write_usize(*border_thickness);
            hasher.write_u8(*border_sides);
        }
        Command::Triangle { a, b, c, clip, color } => {
            hasher.write_u8(2);
            hasher.write_i32(a.0);
            hasher.write_i32(a.1);
            hasher.write_i32(b.0);
            hasher.write_i32(b.1);
            hasher.write_i32(c.0);
            hasher.write_i32(c.1);
            hash_rect(&mut hasher, *clip);
            hasher.write_u32(*color);
        }
        Command::Text {
            text,
            font_id,
            clip,
            x,
            y,
            width,
            height,
            color,
            size,
        } => {
            hasher.write_u8(3);
            hasher.write_usize(text.len());
            hasher.write(text.as_bytes());
            hasher.write_usize(*font_id);
            hash_rect(&mut hasher, *clip);
            hasher.write_i32(*x);
            hasher.write_i32(*y);
            hasher.write_usize(*width);
            hasher.write_usize(*height);
            hasher.write_u32(*color);
            hasher.write_usize(*size);
        }
    }
    hasher.finish()
}

fn scaled_box(x: i32, y: i32, width: usize, height: usize, scale_factor: f32) -> PhysicalRect {
    let x0 = scale_f32(x as f32, scale_factor);
    let y0 = scale_f32(y as f32, scale_factor);
    PhysicalRect::new(
        x0,
        y0,
        x0.saturating_add(scale(width, scale_factor).min(i32::MAX as usize) as i32),
        y0.saturating_add(scale(height, scale_factor).min(i32::MAX as usize) as i32),
    )
}

fn command_clip(command: &Command<'_>) -> Rect {
    match command {
        Command::Rect { clip, .. }
        | Command::RectOutline { clip, .. }
        | Command::Triangle { clip, .. }
        | Command::Text { clip, .. } => *clip,
    }
}

pub fn command_bounds(
    command: &Command<'_>,
    scale_factor: f32,
    framebuffer_width: usize,
    framebuffer_height: usize,
) -> PhysicalRect {
    let bounds = match command {
        Command::Rect {
            x, y, width, height, ..
        }
        | Command::RectOutline {
            x, y, width, height, ..
        } => scaled_box(*x, *y, *width, *height, scale_factor),
        Command::Triangle { a, b, c, .. } => {
            let ax = scale_f32(a.0 as f32, scale_factor);
            let ay = scale_f32(a.1 as f32, scale_factor);
            let bx = scale_f32(b.0 as f32, scale_factor);
            let by = scale_f32(b.1 as f32, scale_factor);
            let cx = scale_f32(c.0 as f32, scale_factor);
            let cy = scale_f32(c.1 as f32, scale_factor);
            PhysicalRect::new(
                ax.min(bx).min(cx).saturating_sub(1),
                ay.min(by).min(cy).saturating_sub(1),
                ax.max(bx).max(cx).saturating_add(2),
                ay.max(by).max(cy).saturating_add(2),
            )
        }
        Command::Text {
            x,
            y,
            width,
            height,
            size,
            ..
        } => {
            let padding = (scale(*size, scale_factor) / 2).max(4).min(i32::MAX as usize) as i32;
            let bounds = scaled_box(*x, *y, *width, *height, scale_factor);
            PhysicalRect::new(
                bounds.x0.saturating_sub(padding),
                bounds.y0.saturating_sub(padding),
                bounds.x1.saturating_add(padding),
                bounds.y1.saturating_add(padding),
            )
        }
    };

    bounds
        .intersection(PhysicalRect::from_rect(command_clip(command).scale(scale_factor)))
        .clamp_to_framebuffer(framebuffer_width, framebuffer_height)
}

pub fn prepare_commands(
    commands: &[Vec<Command<'_>>; 16],
    cache: &mut RenderCache,
    display_scale: f32,
    framebuffer_width: usize,
    framebuffer_height: usize,
) {
    crate::profile!();
    for (layer_index, layer) in commands.iter().enumerate() {
        for (command_index, command) in layer.iter().enumerate() {
            let bounds = command_bounds(command, display_scale, framebuffer_width, framebuffer_height);
            cache.add_command(PreparedCommand {
                layer: layer_index,
                index: command_index,
                bounds,
                hash: command_hash(command, layer_index),
            });
        }
    }
}

pub fn compute_damage(cache: &mut RenderCache) {
    crate::profile!();
    cache.compute_damage();
}

pub fn clear_damage(buffer: &mut [u32], framebuffer_width: usize, damage: &[Rect], color: u32) {
    crate::profile!();
    for rect in damage {
        for y in rect.y..rect.bottom() {
            let start = y * framebuffer_width + rect.x;
            buffer[start..start + rect.width].fill(color);
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
) {
    let clip = command_clip(command).scale(display_scale).intersection(damage);
    if clip.width == 0 || clip.height == 0 {
        return;
    }

    match command {
        Command::Rect {
            x,
            y,
            width,
            height,
            color,
            radius,
            ..
        } => draw_rounded_rect(
            buffer,
            scale_f32(*x as f32, display_scale),
            scale_f32(*y as f32, display_scale),
            scale(*width, display_scale),
            scale(*height, display_scale),
            framebuffer_width,
            framebuffer_height,
            scale(*radius, display_scale),
            *color,
            clip,
        ),
        Command::RectOutline {
            x,
            y,
            width,
            height,
            color,
            border_sides,
            ..
        } => draw_rect_outline(
            buffer,
            scale_f32(*x as f32, display_scale),
            scale_f32(*y as f32, display_scale),
            scale(*width, display_scale),
            scale(*height, display_scale),
            framebuffer_width,
            *color,
            clip,
            *border_sides,
        ),
        Command::Text {
            text,
            x,
            y,
            color,
            size,
            font_id,
            ..
        } => {
            let bitmap = font_bitmaps.entry(*font_id).or_default();
            draw_text(
                text,
                &fonts[*font_id],
                *x,
                *y,
                *size,
                display_scale,
                framebuffer_width,
                buffer,
                *color,
                bitmap,
                clip,
            );
        }
        Command::Triangle { a, b, c, color, .. } => draw_triangle_sdf(
            buffer,
            framebuffer_width,
            framebuffer_height,
            scale_f32(a.0 as f32, display_scale),
            scale_f32(a.1 as f32, display_scale),
            scale_f32(b.0 as f32, display_scale),
            scale_f32(b.1 as f32, display_scale),
            scale_f32(c.0 as f32, display_scale),
            scale_f32(c.1 as f32, display_scale),
            *color,
            clip,
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
) {
    crate::profile!();
    for prepared in prepared {
        let command = &commands[prepared.layer][prepared.index];
        for region in damage {
            if prepared.bounds.intersects(PhysicalRect::from_rect(*region)) {
                draw_command(
                    command,
                    *region,
                    buffer,
                    framebuffer_width,
                    framebuffer_height,
                    display_scale,
                    fonts,
                    font_bitmaps,
                );
            }
        }
    }
}

pub fn present_damage(window: &Window, damage: &[Rect]) {
    crate::profile!();
    window.present_regions(damage);
}
