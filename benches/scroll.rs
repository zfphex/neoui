use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use neoui::*;
use rustc_hash::FxHashMap;

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

fn scroll_frame(scroll_y: i32, line_breaks: &mut Vec<u32>) -> [Vec<Command<'static>>; 16] {
    let clip = Rect::new(VIEW_X, VIEW_Y, VIEW_W, VIEW_H);
    let mut cmds = Vec::with_capacity((ROW_COUNT * 2) as usize);

    cmds.push(Command::Rect {
        bounds: clip,
        clip: Rect::new(0, 0, FB_W as i32, FB_H as i32),
        color: rgb(18, 18, 18),
        radius: 0,
    });

    for row in 0..ROW_COUNT {
        let y = VIEW_Y + row * ROW_STRIDE - scroll_y;
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
        let text = format!("{row:03}. Song title example row for scroll bench");
        let breaks = (line_breaks.len() as u32, line_breaks.len() as u32 + 2);
        line_breaks.push(0);
        line_breaks.push(text.len() as u32);
        cmds.push(Command::Text {
            text: std::borrow::Cow::Owned(text),
            font_id: 0,
            clip,
            bounds: Rect::new(bounds.x + 8, bounds.y, bounds.width - 16, bounds.height),
            color: white(),
            size: 16,
            line_height: None,
            alignment: Alignment::Left,
            breaks,
        });
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
    bitmaps: FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    columns: Vec<u32>,
    line_breaks: Vec<u32>,
    frames: [[Vec<Command<'static>>; 16]; 2],
    step: usize,
    last_damage_rects: usize,
    last_damaged_pixels: usize,
}

impl ScrollBench {
    fn new() -> Self {
        let mut line_breaks = Vec::new();
        let frames = [
            scroll_frame(0, &mut line_breaks),
            scroll_frame(SCROLL_STEP, &mut line_breaks),
        ];
        let mut cache = RenderCache::default();
        cache.update(&frames[0], 1.0, FB_W, FB_H, black());
        cache.finish();
        Self {
            cache,
            buffer: vec![0u32; FB_W * FB_H],
            fonts: vec![fontdue::Font::from_bytes(DEFAULT_FONT, fontdue::FontSettings::default()).unwrap()],
            bitmaps: FxHashMap::default(),
            columns: Vec::new(),
            line_breaks,
            frames,
            step: 0,
            last_damage_rects: 0,
            last_damaged_pixels: 0,
        }
    }

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
                &[],
                &mut self.bitmaps,
                &mut self.columns,
                &self.line_breaks,
            );
        }
        self.cache.finish();
        (damage_rects, damaged_pixels)
    }

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

fn bench_scroll(c: &mut Criterion) {
    c.bench_function("scroll_up_down_full", |b| {
        b.iter_batched_ref(
            ScrollBench::new,
            |ctx| black_box(ctx.scroll_once()),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("scroll_up_down_cache_only", |b| {
        b.iter_batched_ref(
            ScrollBench::new,
            |ctx| black_box(ctx.scroll_cache_only()),
            BatchSize::SmallInput,
        );
    });

    let offsets = [0, 30, 60, 90, 60, 30];
    let mut line_breaks = Vec::new();
    let sequence: Vec<_> = offsets
        .into_iter()
        .map(|offset| scroll_frame(offset, &mut line_breaks))
        .collect();

    c.bench_function("scroll_path_six_steps_full", |b| {
        b.iter_batched_ref(
            || {
                let mut cache = RenderCache::default();
                let buffer = vec![0u32; FB_W * FB_H];
                let fonts = vec![fontdue::Font::from_bytes(DEFAULT_FONT, fontdue::FontSettings::default()).unwrap()];
                let bitmaps = FxHashMap::default();
                let columns = Vec::new();
                cache.update(&sequence[0], 1.0, FB_W, FB_H, black());
                cache.finish();
                (cache, buffer, fonts, bitmaps, columns, 0usize)
            },
            |(cache, buffer, fonts, bitmaps, columns, idx)| {
                *idx = (*idx + 1) % sequence.len();
                let commands = &sequence[*idx];
                if cache.update(commands, 1.0, FB_W, FB_H, black()) {
                    clear_damage(buffer, FB_W, cache.damage(), black());
                    raster_damage(
                        commands,
                        cache,
                        buffer,
                        FB_W,
                        FB_H,
                        1.0,
                        fonts,
                        &[],
                        bitmaps,
                        columns,
                        &line_breaks,
                    );
                }
                cache.finish();
                black_box(*idx);
            },
            BatchSize::SmallInput,
        );
    });
}

fn fast_criterion() -> Criterion {
    Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(100))
        .measurement_time(std::time::Duration::from_millis(300))
        .sample_size(100)
}

criterion_group! {
    name = benches;
    config = fast_criterion();
    targets = bench_scroll
}
criterion_main!(benches);
