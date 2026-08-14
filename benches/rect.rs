use criterion::{Criterion, black_box, criterion_group, criterion_main};
use neoui::*;

fn bench_draw_rect_no_radius(c: &mut Criterion) {
    let mut buffer = vec![0u32; 1000 * 1000];
    c.bench_function("bench_draw_rect_no_radius", |b| {
        b.iter(|| {
            draw_rect_fill(
                black_box(&mut buffer),
                black_box(Rect::new(0, 0, 300, 300)),
                1000,
                1000,
                black_box(0),
                black_box(red()),
                black_box(Rect::new(0, 0, 1000, 1000)),
            );
        });
    });
}

fn bench_draw_rect_pill_shape(c: &mut Criterion) {
    let mut buffer = vec![0u32; 1000 * 1000];
    c.bench_function("bench_draw_rect_pill_shape", |b| {
        b.iter(|| {
            draw_rect_fill(
                black_box(&mut buffer),
                black_box(Rect::new(0, 0, 300, 100)),
                1000,
                1000,
                black_box(150),
                black_box(red()),
                black_box(Rect::new(0, 0, 1000, 1000)),
            );
        });
    });
}

fn bench_draw_rect_fully_clipped(c: &mut Criterion) {
    let mut buffer = vec![0u32; 1000 * 1000];
    c.bench_function("bench_draw_rect_fully_clipped", |b| {
        b.iter(|| {
            draw_rect_fill(
                black_box(&mut buffer),
                black_box(Rect::new(0, 0, 300, 300)),
                1000,
                1000,
                black_box(12),
                black_box(red()),
                black_box(Rect::new(500, 500, 0, 0)),
            );
        });
    });
}

fn bench_draw_rect_partially_clipped(c: &mut Criterion) {
    let mut buffer = vec![0u32; 1000 * 1000];
    c.bench_function("bench_draw_rect_partially_clipped", |b| {
        b.iter(|| {
            draw_rect_fill(
                black_box(&mut buffer),
                black_box(Rect::new(-50, -50, 300, 300)),
                1000,
                1000,
                black_box(12),
                black_box(red()),
                black_box(Rect::new(0, 0, 1000, 1000)),
            );
        });
    });
}

fn bench_draw_stroke_sharp(c: &mut Criterion) {
    let mut buffer = vec![0u32; 1000 * 1000];
    c.bench_function("bench_draw_stroke_sharp", |b| {
        b.iter(|| {
            draw_rect_stroke(
                black_box(&mut buffer),
                black_box(Rect::new(0, 0, 300, 300)),
                1000,
                1000,
                black_box(0),
                black_box(2),
                black_box(red()),
                black_box(Rect::new(0, 0, 1000, 1000)),
                border::ALL,
            );
        });
    });
}

fn bench_draw_stroke_rounded(c: &mut Criterion) {
    let mut buffer = vec![0u32; 1000 * 1000];
    c.bench_function("bench_draw_stroke_rounded", |b| {
        b.iter(|| {
            draw_rect_stroke(
                black_box(&mut buffer),
                black_box(Rect::new(0, 0, 300, 300)),
                1000,
                1000,
                black_box(24),
                black_box(2),
                black_box(red()),
                black_box(Rect::new(0, 0, 1000, 1000)),
                border::ALL,
            );
        });
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
    targets =
        bench_draw_rect_no_radius,
        bench_draw_rect_pill_shape,
        bench_draw_rect_fully_clipped,
        bench_draw_rect_partially_clipped,
        bench_draw_stroke_sharp,
        bench_draw_stroke_rounded
}
criterion_main!(benches);
