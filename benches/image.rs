use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use neoui::*;

const FB_W: usize = 1920;
const FB_H: usize = 1080;

fn pixels(width: usize, height: usize, alpha: u8) -> Vec<u32> {
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            pixels.push(premultiply((x * 7) as u8, (y * 5) as u8, (x ^ y) as u8, alpha));
        }
    }
    pixels
}

fn bench_draw(c: &mut Criterion) {
    let mut group = c.benchmark_group("image");
    let full = Rect::new(0, 0, FB_W as i32, FB_H as i32);

    let mut case = |name: &str, src: (usize, usize), dst: i32, alpha: u8, opacity: u8, radius: usize| {
        let data = pixels(src.0, src.1, alpha);
        let image = Image::new(src.0, src.1, &data);
        group.bench_function(name, |b| {
            let mut columns = Vec::new();
            b.iter_batched_ref(
                || vec![0u32; FB_W * FB_H],
                |buffer| {
                    draw_image(
                        buffer,
                        FB_W,
                        FB_H,
                        black_box(image),
                        0,
                        0,
                        black_box(dst as usize),
                        dst as usize,
                        full,
                        opacity,
                        radius,
                        &mut columns,
                    )
                },
                BatchSize::SmallInput,
            );
        });
    };

    case("icon_32", (32, 32), 32, 255, 255, 0);
    case("icon_32_scaled", (64, 64), 32, 255, 255, 0);
    case("icon_32_radius", (32, 32), 32, 255, 255, 8);
    case("blit_1to1", (512, 512), 512, 255, 255, 0);
    case("blit_alpha", (512, 512), 512, 128, 255, 0);
    case("opacity", (512, 512), 512, 255, 128, 0);
    case("radius", (512, 512), 512, 255, 255, 32);
    case("downscale_4x", (2048, 2048), 512, 255, 255, 0);
    case("upscale_4x", (128, 128), 512, 255, 255, 0);
    case("large", (1024, 1024), 1024, 255, 255, 0);

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
    targets = bench_draw
}
criterion_main!(benches);
