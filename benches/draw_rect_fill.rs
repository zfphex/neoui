use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use neoui::*;

fn bench_draw_rect_fill_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_rect_fill_sizes");
    group.sample_size(1000);
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_millis(700));

    const WIN_W: usize = 1920;
    const WIN_H: usize = 1080;
    let mut buffer = vec![0u32; WIN_W * WIN_H];
    let clip = Rect::new(0, 0, WIN_W as i32, WIN_H as i32);
    let color = red();

    let sizes = [
        ("8x8", 8, 8),
        ("64x64", 64, 64),
        ("300x300", 300, 300),
        ("800x600", 800, 600),
        ("1920x1080", 1920, 1080),
    ];

    for (name, w, h) in sizes {
        let pixels = (w * h) as u64;
        group.throughput(Throughput::Elements(pixels));
        group.bench_function(name, |b| {
            b.iter(|| {
                draw_rect_fill(
                    black_box(&mut buffer),
                    black_box(Rect::new(0, 0, w as i32, h as i32)),
                    WIN_W,
                    WIN_H,
                    black_box(0),
                    black_box(color),
                    black_box(clip),
                );
            });
        });
    }
    group.finish();
}

fn bench_draw_rect_fill_aspect_ratios(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_rect_fill_aspect_ratios");
    group.sample_size(1000);
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_millis(700));

    const WIN_W: usize = 1920;
    const WIN_H: usize = 1080;
    let mut buffer = vec![0u32; WIN_W * WIN_H];
    let clip = Rect::new(0, 0, WIN_W as i32, WIN_H as i32);
    let color = red();

    let cases = [
        ("square_400x400", 400, 400),
        ("wide_banner_1900x50", 1900, 50),
        ("tall_column_50x1000", 50, 1000),
        ("unaligned_width_303x303", 303, 303),
    ];

    for (name, w, h) in cases {
        let pixels = (w * h) as u64;
        group.throughput(Throughput::Elements(pixels));
        group.bench_function(name, |b| {
            b.iter(|| {
                draw_rect_fill(
                    black_box(&mut buffer),
                    black_box(Rect::new(0, 0, w as i32, h as i32)),
                    WIN_W,
                    WIN_H,
                    black_box(8),
                    black_box(color),
                    black_box(clip),
                );
            });
        });
    }
    group.finish();
}

fn bench_draw_rect_fill_radii(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_rect_fill_radii");
    group.sample_size(1000);
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_millis(700));

    const WIN_W: usize = 1920;
    const WIN_H: usize = 1080;
    let mut buffer = vec![0u32; WIN_W * WIN_H];
    let clip = Rect::new(0, 0, WIN_W as i32, WIN_H as i32);
    let color = red();
    let rect = Rect::new(50, 50, 300, 300);

    let radii = [
        ("radius_0_sharp", 0),
        ("radius_8_subtle", 8),
        ("radius_32_medium", 32),
        ("radius_150_max_pill", 150),
        ("radius_500_oversized", 500),
    ];

    for (name, radius) in radii {
        group.bench_function(name, |b| {
            b.iter(|| {
                draw_rect_fill(
                    black_box(&mut buffer),
                    black_box(rect),
                    WIN_W,
                    WIN_H,
                    black_box(radius),
                    black_box(color),
                    black_box(clip),
                );
            });
        });
    }
    group.finish();
}

fn bench_draw_rect_fill_clipping(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_rect_fill_clipping");
    group.sample_size(1000);
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_millis(700));

    const WIN_W: usize = 1920;
    const WIN_H: usize = 1080;
    let mut buffer = vec![0u32; WIN_W * WIN_H];
    let color = red();

    let cases = [
        (
            "unclipped_fullscreen",
            Rect::new(100, 100, 300, 300),
            Rect::new(0, 0, WIN_W as i32, WIN_H as i32),
        ),
        (
            "partial_clip_top_left",
            Rect::new(-150, -150, 300, 300),
            Rect::new(0, 0, WIN_W as i32, WIN_H as i32),
        ),
        (
            "partial_clip_bottom_right",
            Rect::new(1800, 1000, 300, 300),
            Rect::new(0, 0, WIN_W as i32, WIN_H as i32),
        ),
        (
            "interior_viewport_clip",
            Rect::new(0, 0, 800, 800),
            Rect::new(200, 200, 200, 200),
        ),
        (
            "fully_clipped_offscreen",
            Rect::new(3000, 3000, 300, 300),
            Rect::new(0, 0, WIN_W as i32, WIN_H as i32),
        ),
        (
            "zero_size_clip",
            Rect::new(100, 100, 300, 300),
            Rect::new(200, 200, 0, 0),
        ),
    ];

    for (name, rect, clip) in cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                draw_rect_fill(
                    black_box(&mut buffer),
                    black_box(rect),
                    WIN_W,
                    WIN_H,
                    black_box(16),
                    black_box(color),
                    black_box(clip),
                );
            });
        });
    }
    group.finish();
}

fn bench_draw_rect_fill_colors(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_rect_fill_colors");
    group.sample_size(1000);
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_millis(700));

    const WIN_W: usize = 1920;
    const WIN_H: usize = 1080;
    let mut buffer = vec![0u32; WIN_W * WIN_H];
    let clip = Rect::new(0, 0, WIN_W as i32, WIN_H as i32);
    let rect = Rect::new(100, 100, 300, 300);

    let colors = [
        ("solid_opaque_ff0000ff", 0xFF0000FF),
        ("semi_transparent_800000ff", 0x800000FF),
        ("fully_transparent_00000000", 0x00000000),
    ];

    for (name, color) in colors {
        group.bench_function(name, |b| {
            b.iter(|| {
                draw_rect_fill(
                    black_box(&mut buffer),
                    black_box(rect),
                    WIN_W,
                    WIN_H,
                    black_box(12),
                    black_box(color),
                    black_box(clip),
                );
            });
        });
    }
    group.finish();
}

fn bench_draw_rect_fill_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_rect_fill_simd_vs_scalar");
    group.sample_size(1000);
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_millis(700));

    const WIN_W: usize = 1920;
    const WIN_H: usize = 1080;
    let mut buffer = vec![0u32; WIN_W * WIN_H];
    let clip = Rect::new(0, 0, WIN_W as i32, WIN_H as i32);
    let color = red();

    let sizes = [("300x300", 300, 300), ("800x600", 800, 600)];

    for (size_name, w, h) in sizes {
        let rect = Rect::new(50, 50, w, h);
        let pixels = (w * h) as u64;

        group.throughput(Throughput::Elements(pixels));
        group.bench_with_input(BenchmarkId::new("wip", size_name), &rect, |b, &r| {
            b.iter(|| {
                draw_rect_fill_wip(
                    black_box(&mut buffer),
                    black_box(r),
                    WIN_W,
                    WIN_H,
                    black_box(16),
                    black_box(color),
                    black_box(clip),
                );
            });
        });

        group.bench_with_input(BenchmarkId::new("scalar", size_name), &rect, |b, &r| {
            b.iter(|| {
                draw_rect_fill_scalar(
                    black_box(&mut buffer),
                    black_box(r),
                    WIN_W,
                    WIN_H,
                    black_box(16),
                    black_box(color),
                    black_box(clip),
                );
            });
        });

        group.bench_with_input(BenchmarkId::new("current", size_name), &rect, |b, &r| {
            b.iter(|| {
                draw_rect_fill(
                    black_box(&mut buffer),
                    black_box(r),
                    WIN_W,
                    WIN_H,
                    black_box(16),
                    black_box(color),
                    black_box(clip),
                );
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_draw_rect_fill_sizes,
    bench_draw_rect_fill_aspect_ratios,
    bench_draw_rect_fill_radii,
    bench_draw_rect_fill_clipping,
    bench_draw_rect_fill_colors,
    bench_draw_rect_fill_simd_vs_scalar
);
criterion_main!(benches);
