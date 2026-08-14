use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use neoui::*;
use rustc_hash::FxHashMap;

const FB_W: usize = 1920;
const FB_H: usize = 1080;

fn full() -> Rect {
    Rect::new(0, 0, FB_W as i32, FB_H as i32)
}

fn image(width: usize, height: usize, alpha: u8) -> Image {
    let mut pixels = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&[(x * 7) as u8, (y * 5) as u8, (x ^ y) as u8, alpha]);
        }
    }
    Image::from_rgba8(width, height, &pixels)
}

struct Bed {
    buffer: Vec<u32>,
    cache: ImageCache,
    fonts: Vec<fontdue::Font>,
    glyphs: FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
}

impl Bed {
    fn new() -> Self {
        Self {
            buffer: vec![0; FB_W * FB_H],
            cache: ImageCache::new(),
            fonts: Vec::new(),
            glyphs: FxHashMap::default(),
        }
    }

    fn draw(&mut self, image: &Image, bounds: Rect, fit: ImageFit, opacity: u8, radius: usize) {
        let command = Command::Image {
            id: image.id,
            width: image.width,
            height: image.height,
            opaque: image.opaque,
            pixels: &image.pixels,
            bounds,
            clip: full(),
            fit,
            opacity,
            radius,
        };
        draw_command(
            black_box(&command),
            full(),
            &mut self.buffer,
            FB_W,
            FB_H,
            1.0,
            &self.fonts,
            &[],
            &mut self.glyphs,
            &mut self.cache,
        );
    }
}

fn bench_warm(c: &mut Criterion) {
    let mut group = c.benchmark_group("warm");

    let bench_warm_case = |group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
                           name: &str,
                           src: (usize, usize),
                           dst: i32,
                           fit: ImageFit,
                           opacity: u8,
                           radius: usize| {
        let img = image(src.0, src.1, 255);
        let bounds = Rect::new(0, 0, dst, dst);
        group.bench_function(name, |b| {
            b.iter_batched_ref(
                || {
                    let mut bed = Bed::new();
                    bed.draw(&img, bounds, fit, opacity, radius);
                    bed
                },
                |bed| bed.draw(&img, bounds, fit, opacity, radius),
                BatchSize::SmallInput,
            );
        });
    };

    bench_warm_case(&mut group, "blit_1to1", (512, 512), 512, ImageFit::Stretch, 255, 0);

    let img_alpha = image(512, 512, 128);
    let bounds_512 = Rect::new(0, 0, 512, 512);
    group.bench_function("blit_1to1_alpha", |b| {
        b.iter_batched_ref(
            || {
                let mut bed = Bed::new();
                bed.draw(&img_alpha, bounds_512, ImageFit::Stretch, 255, 0);
                bed
            },
            |bed| bed.draw(&img_alpha, bounds_512, ImageFit::Stretch, 255, 0),
            BatchSize::SmallInput,
        );
    });

    bench_warm_case(
        &mut group,
        "blit_1to1_opacity",
        (512, 512),
        512,
        ImageFit::Stretch,
        128,
        0,
    );
    bench_warm_case(
        &mut group,
        "blit_1to1_radius",
        (512, 512),
        512,
        ImageFit::Stretch,
        255,
        16,
    );
    bench_warm_case(&mut group, "downscale_4x", (2048, 2048), 512, ImageFit::Stretch, 255, 0);
    bench_warm_case(&mut group, "upscale_4x", (128, 128), 512, ImageFit::Stretch, 255, 0);

    group.finish();
}

fn bench_cold(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold");

    let bench_cold_case = |group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
                           name: &str,
                           src: (usize, usize),
                           dst: i32,
                           fit: ImageFit,
                           radius: usize| {
        let img = image(src.0, src.1, 255);
        let bounds = Rect::new(0, 0, dst, dst);
        group.bench_function(name, |b| {
            b.iter_batched_ref(
                Bed::new,
                |bed| {
                    bed.cache.entries.clear();
                    bed.cache.bytes = 0;
                    bed.draw(&img, bounds, fit, 255, radius);
                },
                BatchSize::SmallInput,
            );
        });
    };

    bench_cold_case(&mut group, "blit_1to1", (512, 512), 512, ImageFit::Stretch, 0);
    bench_cold_case(&mut group, "downscale_4x", (2048, 2048), 512, ImageFit::Stretch, 0);
    bench_cold_case(
        &mut group,
        "downscale_4x_radius",
        (2048, 2048),
        512,
        ImageFit::Stretch,
        16,
    );
    bench_cold_case(&mut group, "upscale_4x", (128, 128), 512, ImageFit::Stretch, 0);
    bench_cold_case(&mut group, "cover_crop", (2048, 1024), 512, ImageFit::Cover, 0);

    group.finish();
}

fn bench_animated(c: &mut Criterion) {
    let mut group = c.benchmark_group("animated");

    let img = image(512, 512, 255);
    let bounds = Rect::new(0, 0, 512, 512);

    group.bench_function("fade", |b| {
        let mut frame = 0u32;
        b.iter_batched_ref(
            Bed::new,
            |bed| {
                frame = frame.wrapping_add(1);
                let opacity = (frame % 255) as u8 + 1;
                bed.draw(&img, bounds, ImageFit::Stretch, opacity, 0);
            },
            BatchSize::SmallInput,
        );
    });

    let img_1024 = image(1024, 1024, 255);
    group.bench_function("resize", |b| {
        let mut frame = 0u32;
        b.iter_batched_ref(
            Bed::new,
            |bed| {
                frame = frame.wrapping_add(1);
                let side = 256 + (frame % 256) as i32;
                bed.draw(&img_1024, Rect::new(0, 0, side, side), ImageFit::Stretch, 255, 0);
            },
            BatchSize::SmallInput,
        );
    });

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
    targets = bench_warm, bench_cold, bench_animated
}
criterion_main!(benches);
