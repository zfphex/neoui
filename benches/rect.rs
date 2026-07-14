use neoui::*;

fn main() {
    divan::main();
}

struct BenchContext {
    buffer: Vec<u32>,
}

fn create_context() -> BenchContext {
    BenchContext {
        buffer: vec![0u32; 1000 * 1000],
    }
}

#[divan::bench(sample_count = 100)]
fn bench_draw_rect_no_radius(bencher: divan::Bencher) {
    bencher.with_inputs(create_context).bench_refs(|ctx| {
        draw_rect_fill(
            &mut ctx.buffer,
            Rect::new(0, 0, 300, 300),
            1000,
            1000,
            0,
            red(),
            Rect::new(0, 0, 1000, 1000),
        );
    });
}

#[divan::bench(sample_count = 100)]
fn bench_draw_rect_pill_shape(bencher: divan::Bencher) {
    bencher.with_inputs(create_context).bench_refs(|ctx| {
        draw_rect_fill(
            &mut ctx.buffer,
            Rect::new(0, 0, 300, 100),
            1000,
            1000,
            150,
            red(),
            Rect::new(0, 0, 1000, 1000),
        );
    });
}

#[divan::bench(sample_count = 100)]
fn bench_draw_rect_fully_clipped(bencher: divan::Bencher) {
    bencher.with_inputs(create_context).bench_refs(|ctx| {
        draw_rect_fill(
            &mut ctx.buffer,
            Rect::new(0, 0, 300, 300),
            1000,
            1000,
            12,
            red(),
            Rect::new(500, 500, 0, 0), // Zero size clip box
        );
    });
}

#[divan::bench(sample_count = 100)]
fn bench_draw_rect_partially_clipped(bencher: divan::Bencher) {
    bencher.with_inputs(create_context).bench_refs(|ctx| {
        draw_rect_fill(
            &mut ctx.buffer,
            Rect::new(-50, -50, 300, 300),
            1000,
            1000,
            12,
            red(),
            Rect::new(0, 0, 1000, 1000), // Screen bounds force clipping of negative coordinates
        );
    });
}

#[divan::bench(sample_count = 100)]
fn bench_draw_stroke_sharp(bencher: divan::Bencher) {
    bencher.with_inputs(create_context).bench_refs(|ctx| {
        draw_rect_stroke(
            &mut ctx.buffer,
            Rect::new(0, 0, 300, 300),
            1000,
            1000,
            0,
            2,
            red(),
            Rect::new(0, 0, 1000, 1000),
            border::ALL,
        );
    });
}

#[divan::bench(sample_count = 100)]
fn bench_draw_stroke_rounded(bencher: divan::Bencher) {
    bencher.with_inputs(create_context).bench_refs(|ctx| {
        draw_rect_stroke(
            &mut ctx.buffer,
            Rect::new(0, 0, 300, 300),
            1000,
            1000,
            24,
            2,
            red(),
            Rect::new(0, 0, 1000, 1000),
            border::ALL,
        );
    });
}
