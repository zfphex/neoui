use divan::black_box;
use neoui::*;

fn main() {
    divan::main();
}

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
        self.cache
            .update(commands, 1.0, WIDTH as usize, HEIGHT as usize, 0);
        let n = self.cache.damage().len();
        self.cache.finish();
        n
    }
}

#[divan::bench]
fn full_initial(bencher: divan::Bencher) {
    let cmds = layers(vec![rect(0, 0, WIDTH, HEIGHT, 1)]);
    bencher
        .with_inputs(CacheBench::default)
        .bench_refs(|cache| black_box(cache.frame(&cmds, true)));
}

#[divan::bench]
fn identical_frame(bencher: divan::Bencher) {
    let cmds = layers(vec![rect(100, 100, 400, 300, 1)]);
    bencher
        .with_inputs(|| {
            let mut cache = CacheBench::default();
            cache.frame(&cmds, false);
            cache
        })
        .bench_refs(|cache| black_box(cache.frame(&cmds, false)));
}

struct Alternating {
    cache: CacheBench,
    state: u32,
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
            let cmds = layers(vec![rect(300, 300, 40, 24, ctx.state)]);
            black_box(ctx.cache.frame(&cmds, false))
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
            let cmds = layers(vec![rect(x, 300, 40, 24, 1)]);
            black_box(ctx.cache.frame(&cmds, false))
        });
}

struct Scrolling {
    cache: CacheBench,
    states: [[Vec<Command<'static>>; 16]; 2],
    state: usize,
}

#[divan::bench]
fn scrolling_list(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
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
        })
        .bench_refs(|ctx| {
            ctx.state ^= 1;
            black_box(ctx.cache.frame(&ctx.states[ctx.state], false))
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
            let cmds = layers(vec![rect(0, 0, WIDTH, HEIGHT, ctx.state)]);
            black_box(ctx.cache.frame(&cmds, false))
        });
}
