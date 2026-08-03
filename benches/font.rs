use neoui::*;
use rustc_hash::FxHashMap;

fn main() {
    divan::main();
}

struct BenchContext {
    fonts: Vec<fontdue::Font>,
    buffer: Vec<u32>,
    glyph: FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
    text: String,
}

#[divan::bench(sample_count = 500)]
fn bench_draw_text(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| BenchContext {
            fonts: vec![fontdue::Font::from_bytes(DEFAULT_FONT, fontdue::FontSettings::default()).unwrap()],
            buffer: vec![0u32; 1000usize * 1000 * 4],
            glyph: FxHashMap::default(),
            text: "abcdefghijklmnopqrstuvwxyz1234567890-=!@#$%^&*()_+".repeat(200),
        })
        .bench_refs(|ctx| {
            draw_text(
                &ctx.text,
                &ctx.fonts,
                0,
                &[],
                Rect::new(0, 0, 10000, 32),
                32,
                None,
                Alignment::Left,
                10000,
                &mut ctx.buffer,
                white(),
                &mut ctx.glyph,
                Rect::new(0, 0, 2000, 2000),
            );
        });
}
