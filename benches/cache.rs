use divan::black_box;
use neoui::Rect;
use neoui::render_cache::{PhysicalRect, PreparedCommand, RenderCache};

fn main() {
    divan::main();
}

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

#[derive(Default)]
struct CacheBench {
    cache: RenderCache,
}

impl CacheBench {
    fn frame(&mut self, width: usize, height: usize, commands: &[(Rect, u64)], force_full: bool) -> (usize, usize) {
        if force_full {
            self.cache.invalidate();
        }
        self.cache.begin_frame(width, height, 1.0, 0);
        for (index, (bounds, hash)) in commands.iter().enumerate() {
            self.cache.add_command(PreparedCommand {
                layer: 0,
                index,
                bounds: PhysicalRect::from_rect(*bounds),
                hash: *hash,
            });
        }
        self.cache.compute_damage();
        let result = (self.cache.stats.damage_rects, self.cache.stats.damaged_pixels);
        self.cache.complete_frame();
        result
    }
}

#[divan::bench]
fn full_initial(bencher: divan::Bencher) {
    bencher
        .with_inputs(CacheBench::default)
        .bench_refs(|cache| black_box(cache.frame(WIDTH, HEIGHT, &[(Rect::new(0, 0, WIDTH, HEIGHT), 1)], true)));
}

#[divan::bench]
fn identical_frame(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let mut cache = CacheBench::default();
            cache.frame(WIDTH, HEIGHT, &[(Rect::new(100, 100, 400, 300), 1)], false);
            cache
        })
        .bench_refs(|cache| black_box(cache.frame(WIDTH, HEIGHT, &[(Rect::new(100, 100, 400, 300), 1)], false)));
}

struct Alternating {
    cache: CacheBench,
    state: u64,
}

#[divan::bench]
fn one_small_change(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| Alternating {
            cache: CacheBench::default(),
            state: 0,
        })
        .bench_refs(|ctx| {
            ctx.state ^= 1;
            black_box(
                ctx.cache
                    .frame(WIDTH, HEIGHT, &[(Rect::new(300, 300, 40, 24), ctx.state)], false),
            )
        });
}

#[divan::bench]
fn moving_control(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| Alternating {
            cache: CacheBench::default(),
            state: 0,
        })
        .bench_refs(|ctx| {
            ctx.state ^= 1;
            let x = if ctx.state == 0 { 300 } else { 700 };
            black_box(ctx.cache.frame(WIDTH, HEIGHT, &[(Rect::new(x, 300, 40, 24), 1)], false))
        });
}

struct Scrolling {
    cache: CacheBench,
    states: [Vec<(Rect, u64)>; 2],
    state: usize,
}

#[divan::bench]
fn scrolling_list(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let make = |offset: usize| {
                (0..100)
                    .map(|row| (Rect::new(200, row * 28 + offset, 900, 26), row as u64))
                    .collect()
            };
            Scrolling {
                cache: CacheBench::default(),
                states: [make(0), make(12)],
                state: 0,
            }
        })
        .bench_refs(|ctx| {
            ctx.state ^= 1;
            black_box(ctx.cache.frame(WIDTH, HEIGHT, &ctx.states[ctx.state], false))
        });
}

#[divan::bench]
fn full_screen_change(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| Alternating {
            cache: CacheBench::default(),
            state: 0,
        })
        .bench_refs(|ctx| {
            ctx.state ^= 1;
            black_box(
                ctx.cache
                    .frame(WIDTH, HEIGHT, &[(Rect::new(0, 0, WIDTH, HEIGHT), ctx.state)], false),
            )
        });
}
