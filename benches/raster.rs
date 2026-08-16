use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use neoui::*;
use rustc_hash::FxHashMap;
use std::cell::RefCell;

const WIDTH: i32 = 1920;
const HEIGHT: i32 = 1080;

fn layers(commands: Vec<Command<'static>>) -> [Vec<Command<'static>>; 16] {
    let mut layers = std::array::from_fn(|_| Vec::new());
    layers[0] = commands;
    layers
}

fn rect(x: i32, y: i32, w: i32, h: i32, color: u32) -> Command<'static> {
    Command::Rect {
        bounds: Rect::new(x, y, w, h),
        clip: Rect::new(0, 0, WIDTH, HEIGHT),
        color,
        radius: 0,
    }
}

struct Scene {
    cache: RenderCache,
    commands: [Vec<Command<'static>>; 16],
    buffer: Vec<u32>,
    fonts: Vec<fontdue::Font>,
    font_bitmaps: FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    columns: Vec<u32>,
}

fn scene(static_count: usize, spots: usize) -> Scene {
    let build = |tint: u32| {
        let mut cmds: Vec<Command<'static>> = (0..static_count)
            .map(|i| {
                let x = ((i % 64) * 30) as i32;
                let y = ((i / 64) * 17 % 1060) as i32;
                rect(x, y, 26, 15, 7)
            })
            .collect();
        for s in 0..spots {
            let ix = (s % 15) as i32;
            let iy = (s / 15) as i32;
            cmds.push(rect(ix * 128 + 16, iy * 128 + 16, 32, 24, tint));
        }
        layers(cmds)
    };

    let mut cache = RenderCache::default();
    let first = build(1);
    cache.update(&first, 1.0, WIDTH as usize, HEIGHT as usize, 0);
    cache.finish();

    let commands = build(2);
    cache.update(&commands, 1.0, WIDTH as usize, HEIGHT as usize, 0);

    Scene {
        cache,
        commands,
        buffer: vec![0; (WIDTH * HEIGHT) as usize],
        fonts: Vec::new(),
        font_bitmaps: FxHashMap::default(),
        columns: Vec::new(),
    }
}

fn old_path(s: &mut Scene) {
    for prepared in s.cache.prepared() {
        let command = &s.commands[prepared.layer][prepared.index];
        for region in s.cache.damage() {
            if prepared.bounds.intersects(*region) {
                draw_command(
                    command,
                    *region,
                    &mut s.buffer,
                    WIDTH as usize,
                    HEIGHT as usize,
                    1.0,
                    &s.fonts,
                    &[],
                    &mut s.font_bitmaps,
                    &mut s.columns,
                    &[],
                );
            }
        }
    }
}

fn new_path(s: &mut Scene) {
    raster_damage(
        &s.commands,
        &s.cache,
        &mut s.buffer,
        WIDTH as usize,
        HEIGHT as usize,
        1.0,
        &s.fonts,
        &[],
        &mut s.font_bitmaps,
        &mut s.columns,
        &[],
    );
}

const CASES: [(usize, usize); 8] = [
    (3000, 120),
    (3000, 32),
    (3000, 16),
    (3000, 8),
    (3000, 4),
    (3000, 1),
    (300, 120),
    (300, 8),
];

fn bench_raster(c: &mut Criterion) {
    let mut group = c.benchmark_group("raster");

    for &(statics, spots) in &CASES {
        let param = format!("{statics}x{spots}");

        group.bench_with_input(BenchmarkId::new("old", &param), &(statics, spots), |b, &(st, sp)| {
            let s = RefCell::new(scene(st, sp));
            b.iter(|| black_box(old_path(&mut s.borrow_mut())));
        });

        group.bench_with_input(BenchmarkId::new("new", &param), &(statics, spots), |b, &(st, sp)| {
            let s = RefCell::new(scene(st, sp));
            b.iter(|| black_box(new_path(&mut s.borrow_mut())));
        });
    }

    group.finish();
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
    targets = bench_raster
}
criterion_main!(benches);
