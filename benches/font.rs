use neoui::*;
use rustc_hash::FxHashMap;

fn main() {
    divan::main();
}

struct BenchContext {
    font: fontdue::Font,
    buffer: Vec<u32>,
    glyph: FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>,
    text: String,
}

#[divan::bench(sample_count = 500)]
fn bench_draw_text(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let font = fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap();
            let text = "abcdefghijklmnopqrstuvwxyz1234567890-=!@#$%^&*()_+".repeat(200);
            let mut glyph = FxHashMap::default();
            prepare_glyphs(&text, &font, 32, &mut glyph);
            BenchContext {
                font,
                buffer: vec![0u32; 1000usize * 1000 * 4],
                glyph,
                text,
            }
        })
        .bench_refs(|ctx| {
            draw_text(
                &ctx.text,
                &ctx.font,
                0,
                0,
                32,
                10000,
                &mut ctx.buffer,
                white(),
                &ctx.glyph,
                Rect::new(0, 0, 2000, 2000),
            );
        });
}
