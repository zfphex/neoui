use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use neoui::*;

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

#[derive(Default)]
struct CacheBench {
    cache: RenderCache,
}

impl CacheBench {
    fn frame(&mut self, commands: &[Vec<Command<'static>>; 16], force_full: bool) -> usize {
        if force_full {
            self.cache.invalidate();
        }
        self.cache.update(commands, 1.0, WIDTH as usize, HEIGHT as usize, 0);
        let n = self.cache.damage().len();
        self.cache.finish();
        n
    }
}

struct Alternating {
    cache: CacheBench,
    state: u32,
}

struct Scrolling {
    cache: CacheBench,
    states: [[Vec<Command<'static>>; 16]; 2],
    state: usize,
}

fn bench_cache(c: &mut Criterion) {
    let cmds_full = layers(vec![rect(0, 0, WIDTH, HEIGHT, 1)]);
    c.bench_function("full_initial", |b| {
        b.iter_batched_ref(
            CacheBench::default,
            |cache| black_box(cache.frame(&cmds_full, true)),
            BatchSize::SmallInput,
        );
    });

    let cmds_ident = layers(vec![rect(100, 100, 400, 300, 1)]);
    c.bench_function("identical_frame", |b| {
        b.iter_batched_ref(
            || {
                let mut cache = CacheBench::default();
                cache.frame(&cmds_ident, false);
                cache
            },
            |cache| black_box(cache.frame(&cmds_ident, false)),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("one_small_change", |b| {
        b.iter_batched_ref(
            || Alternating {
                cache: CacheBench::default(),
                state: 0,
            },
            |ctx| {
                ctx.state ^= 1;
                let cmds = layers(vec![rect(300, 300, 40, 24, ctx.state)]);
                black_box(ctx.cache.frame(&cmds, false))
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("moving_control", |b| {
        b.iter_batched_ref(
            || Alternating {
                cache: CacheBench::default(),
                state: 0,
            },
            |ctx| {
                ctx.state ^= 1;
                let x = if ctx.state == 0 { 300 } else { 700 };
                let cmds = layers(vec![rect(x, 300, 40, 24, 1)]);
                black_box(ctx.cache.frame(&cmds, false))
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("scrolling_list", |b| {
        b.iter_batched_ref(
            || {
                let make = |offset: i32| {
                    layers(
                        (0..100)
                            .map(|row| rect(200, row * 28 + offset, 900, 26, row as u32))
                            .collect(),
                    )
                };
                Scrolling {
                    cache: CacheBench::default(),
                    states: [make(0), make(12)],
                    state: 0,
                }
            },
            |ctx| {
                ctx.state ^= 1;
                black_box(ctx.cache.frame(&ctx.states[ctx.state], false))
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("full_screen_change", |b| {
        b.iter_batched_ref(
            || Alternating {
                cache: CacheBench::default(),
                state: 0,
            },
            |ctx| {
                ctx.state ^= 1;
                let cmds = layers(vec![rect(0, 0, WIDTH, HEIGHT, ctx.state)]);
                black_box(ctx.cache.frame(&cmds, false))
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
    targets = bench_cache
}
criterion_main!(benches);
