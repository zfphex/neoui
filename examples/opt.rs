use neoui::*;
use rustc_hash::FxHashMap;
use std::hint::black_box;

fn main() {
    defer_results!();
    let font = fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap();
    let mut buffer = vec![0u32; 1000usize * 1000 * 4];
    let mut fxcache = FxHashMap::default();

    let text = black_box("abcdefghijklmnopqrstuvwxyz1234567890-=!@#$%^&*()_+".repeat(200));

    //Without writing anything 2.15ms
    //Non-gamma corrected      2.29ms
    //Const gamma corrected    2.45ms
    //Gamma corrected          2.45ms
    //New                      149.65us

    for _ in 0..1000 {
        draw_text(
            &text,
            &font,
            0,
            0,
            32,
            1.0,
            10000,
            &mut buffer,
            white(),
            false,
            &mut fxcache,
            Rect::new(0, 0, 2000, 2000),
        );
    }
}
