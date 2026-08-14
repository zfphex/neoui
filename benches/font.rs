use criterion::{Criterion, black_box, criterion_group, criterion_main};
use neoui::*;
use rustc_hash::FxHashMap;

struct BenchContext {
    fonts: Vec<fontdue::Font>,
    buffer: Vec<u32>,
    glyph: FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    text: String,
}

fn bench_draw_text(c: &mut Criterion) {
    let mut ctx = BenchContext {
        fonts: vec![fontdue::Font::from_bytes(DEFAULT_FONT, fontdue::FontSettings::default()).unwrap()],
        buffer: vec![0u32; 1000usize * 1000 * 4],
        glyph: FxHashMap::default(),
        text: "abcdefghijklmnopqrstuvwxyz1234567890-=!@#$%^&*()_+".repeat(200),
    };

    c.bench_function("bench_draw_text", |b| {
        b.iter(|| {
            draw_text(
                black_box(&ctx.text),
                black_box(&ctx.fonts),
                0,
                &[],
                black_box(Rect::new(0, 0, 10000, 32)),
                32,
                None,
                Alignment::Left,
                10000,
                black_box(&mut ctx.buffer),
                black_box(white()),
                black_box(&mut ctx.glyph),
                black_box(Rect::new(0, 0, 2000, 2000)),
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
    targets = bench_draw_text
}
criterion_main!(benches);
