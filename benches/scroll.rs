//! Scroll-view style workload: many list rows whose Y positions shift each frame
//! (scroll up/down), then clear + raster only the damaged regions.
//!
//! This matches real scrolling cost better than cache-only benches: more damage
//! rects can make rasterization dominate even if hashing is cheaper.

use divan::black_box;
use neoui::*;
use rustc_hash::FxHashMap;

fn main() {
    divan::main();
}

const FB_W: usize = 1920;
const FB_H: usize = 1080;
const VIEW_X: i32 = 200;
const VIEW_Y: i32 = 80;
const VIEW_W: i32 = 900;
const VIEW_H: i32 = 900;
const ROW_H: i32 = 28;
const ROW_GAP: i32 = 2;
const ROW_STRIDE: i32 = ROW_H + ROW_GAP;
const ROW_COUNT: i32 = 120;
const SCROLL_STEP: i32 = 30;

fn layers(commands: Vec<Command<'static>>) -> [Vec<Command<'static>>; 16] {
    let mut layers = std::array::from_fn(|_| Vec::new());
    layers[0] = commands;
    layers
}

/// Build the draw list for a scrollable track list at `scroll_y`.
fn scroll_frame(scroll_y: i32) -> [Vec<Command<'static>>; 16] {
    let clip = Rect::new(VIEW_X, VIEW_Y, VIEW_W, VIEW_H);
    let mut cmds = Vec::with_capacity((ROW_COUNT * 2) as usize);

    // Panel background (stable).
    cmds.push(Command::Rect {
        bounds: clip,
        clip: Rect::new(0, 0, FB_W as i32, FB_H as i32),
        color: rgb(18, 18, 18),
        radius: 0,
    });

    for row in 0..ROW_COUNT {
        let y = VIEW_Y + row * ROW_STRIDE - scroll_y;
        // Skip fully clipped rows (same idea as walk_layout scroll culling).
        if y + ROW_H <= VIEW_Y || y >= VIEW_Y + VIEW_H {
            continue;
        }
        let bounds = Rect::new(VIEW_X + 8, y, VIEW_W - 16, ROW_H);
        let color = if row % 2 == 0 { rgb(35, 35, 35) } else { rgb(28, 28, 28) };
        cmds.push(Command::Rect {
            bounds,
            clip,
            color,
            radius: 0,
        });
        // Track title text (dominant cost when scrolling real UIs).
        cmds.push(Command::Text {
            text: std::borrow::Cow::Owned(format!("{row:03}. Song title example row for scroll bench")),
            font_id: 0,
            clip,
            bounds: Rect::new(bounds.x + 8, bounds.y, bounds.width - 16, bounds.height),
            color: white(),
            size: 16,
        });
        // Outline like selected/hover chrome.
        if row % 7 == 0 {
            cmds.push(Command::RectStroke {
                bounds,
                clip,
                color: rgb(90, 90, 90),
                radius: 0,
                border_thickness: 1,
                border_sides: border::ALL,
            });
        }
    }

    layers(cmds)
}

struct ScrollBench {
    cache: RenderCache,
    buffer: Vec<u32>,
    fonts: Vec<fontdue::Font>,
    bitmaps: FxHashMap<usize, FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    image_cache: ImageCache,
    frames: [[Vec<Command<'static>>; 16]; 2],
    step: usize,
    /// last frame metrics (for reporting, not timed)
    last_damage_rects: usize,
    last_damaged_pixels: usize,
}

impl ScrollBench {
    fn new() -> Self {
        // Two scroll positions: "down" and further "down" (then we alternate).
        let frames = [scroll_frame(0), scroll_frame(SCROLL_STEP)];
        let mut cache = RenderCache::default();
        // Warm up so first timed sample is a scroll delta, not cold resize.
        cache.update(&frames[0], 1.0, FB_W, FB_H, black());
        cache.finish();
        Self {
            cache,
            buffer: vec![0u32; FB_W * FB_H],
            fonts: vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()],
            bitmaps: FxHashMap::default(),
            image_cache: ImageCache::new(),
            frames,
            step: 0,
            last_damage_rects: 0,
            last_damaged_pixels: 0,
        }
    }

    /// One frame of scrolling: toggle between two scroll offsets.
    fn scroll_once(&mut self) -> (usize, usize) {
        self.step ^= 1;
        let commands = &self.frames[self.step];
        let dirty = self.cache.update(commands, 1.0, FB_W, FB_H, black());
        let damage = self.cache.damage();
        let damage_rects = damage.len();
        let damaged_pixels = damage
            .iter()
            .map(|r| (r.width.max(0) as usize).saturating_mul(r.height.max(0) as usize))
            .sum();
        self.last_damage_rects = damage_rects;
        self.last_damaged_pixels = damaged_pixels;

        if dirty {
            clear_damage(&mut self.buffer, FB_W, damage, black());
            raster_damage(
                commands,
                &self.cache,
                &mut self.buffer,
                FB_W,
                FB_H,
                1.0,
                &self.fonts,
                &mut self.bitmaps,
                &mut self.image_cache,
            );
        }
        self.cache.finish();
        (damage_rects, damaged_pixels)
    }

    /// Cache update only (no clear/raster).
    fn scroll_cache_only(&mut self) -> (usize, usize) {
        self.step ^= 1;
        let commands = &self.frames[self.step];
        self.cache.update(commands, 1.0, FB_W, FB_H, black());
        let damage = self.cache.damage();
        let damage_rects = damage.len();
        let damaged_pixels = damage
            .iter()
            .map(|r| (r.width.max(0) as usize).saturating_mul(r.height.max(0) as usize))
            .sum();
        self.last_damage_rects = damage_rects;
        self.last_damaged_pixels = damaged_pixels;
        self.cache.finish();
        (damage_rects, damaged_pixels)
    }
}

#[divan::bench(sample_count = 100)]
fn scroll_up_down_full(bencher: divan::Bencher) {
    bencher
        .with_inputs(ScrollBench::new)
        .bench_refs(|ctx| black_box(ctx.scroll_once()));
}

#[divan::bench(sample_count = 100)]
fn scroll_up_down_cache_only(bencher: divan::Bencher) {
    bencher
        .with_inputs(ScrollBench::new)
        .bench_refs(|ctx| black_box(ctx.scroll_cache_only()));
}

/// Multi-step scroll: 0 → 30 → 60 → 90 → 60 → 30 (up and down through a range).
#[divan::bench(sample_count = 50)]
fn scroll_path_six_steps_full(bencher: divan::Bencher) {
    let offsets = [0, 30, 60, 90, 60, 30];
    let sequence: Vec<_> = offsets.into_iter().map(scroll_frame).collect();

    bencher
        .with_inputs(|| {
            let mut cache = RenderCache::default();
            let buffer = vec![0u32; FB_W * FB_H];
            let fonts = vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()];
            let bitmaps = FxHashMap::default();
            let image_cache = ImageCache::new();
            cache.update(&sequence[0], 1.0, FB_W, FB_H, black());
            cache.finish();
            (cache, buffer, fonts, bitmaps, image_cache, 0usize)
        })
        .bench_refs(|(cache, buffer, fonts, bitmaps, image_cache, idx)| {
            *idx = (*idx + 1) % sequence.len();
            let commands = &sequence[*idx];
            if cache.update(commands, 1.0, FB_W, FB_H, black()) {
                clear_damage(buffer, FB_W, cache.damage(), black());
                raster_damage(commands, cache, buffer, FB_W, FB_H, 1.0, fonts, bitmaps, image_cache);
            }
            cache.finish();
            black_box(*idx);
        });
}
