//! The banded parallel rasterizer must produce byte-identical output to painting
//! every command against every damage rect on the whole framebuffer.

use neoui::*;
use rustc_hash::FxHashMap;

const W: usize = 1920;
const H: usize = 1080;

fn scene() -> [Vec<Command<'static>>; 16] {
    let clip = Rect::new(0, 0, W as i32, H as i32);
    let mut cmds = Vec::new();

    for i in 0..400 {
        let x = ((i % 20) * 96) as i32;
        let y = ((i / 20) * 53) as i32;
        cmds.push(Command::Rect {
            bounds: Rect::new(x, y, 90, 48),
            clip,
            color: 0x20_3040 + i as u32 * 7,
            radius: (i % 12) as usize,
        });
        cmds.push(Command::RectStroke {
            bounds: Rect::new(x + 2, y + 2, 86, 44),
            clip,
            color: 0xff_00aa,
            radius: (i % 9) as usize,
            border_thickness: 1 + (i % 3) as usize,
            border_sides: border::ALL,
        });
        cmds.push(Command::Triangle {
            a: (x + 4, y + 4),
            b: (x + 40, y + 10),
            c: (x + 12, y + 44),
            clip,
            color: 0x00_ff88,
        });
        cmds.push(Command::Text {
            text: format!("row {i} band").into(),
            font_id: 0,
            clip,
            bounds: Rect::new(x + 6, y + 6, 80, 36),
            color: 0xff_ffff,
            size: 18,
        });
    }

    let mut layers = std::array::from_fn(|_| Vec::new());
    layers[0] = cmds;
    layers
}

#[test]
fn banded_raster_matches_whole_frame_raster() {
    let fonts = vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()];
    let commands = scene();

    let mut cache = RenderCache::default();
    assert!(cache.update(&commands, 1.0, W, H, black()));
    // A resize-style full redraw: one damage rect covering everything.
    assert_eq!(cache.damage(), [Rect::new(0, 0, W as i32, H as i32)]);

    let mut glyphs = FxHashMap::default();
    let mut images = FxHashMap::default();
    prepare_caches(&commands, &cache, 1.0, &fonts, &mut glyphs, &mut images);

    let mut reference = vec![0u32; W * H];
    clear_damage(&mut reference, W, cache.damage(), black());
    for prepared in cache.prepared() {
        let command = &commands[prepared.layer][prepared.index];
        for region in cache.damage() {
            if prepared.bounds.intersects(*region) {
                draw_command(command, *region, &mut reference, W, H, 0, 1.0, &fonts, &glyphs, &images);
            }
        }
    }

    let mut banded = vec![0u32; W * H];
    raster_damage(
        &commands,
        &cache,
        &mut banded,
        W,
        H,
        1.0,
        black(),
        &fonts,
        &mut glyphs,
        &mut images,
    );

    assert!(pool::pool().workers() > 1, "test needs a multi-threaded pool");
    assert!(reference.iter().any(|&p| p != black()), "scene painted nothing");

    let mismatch = reference
        .iter()
        .zip(&banded)
        .position(|(a, b)| a != b)
        .map(|i| (i % W, i / W, reference[i], banded[i]));
    assert_eq!(mismatch, None, "banded output differs at (x, y, expected, got)");
}

#[test]
fn banded_raster_matches_on_partial_damage() {
    let fonts = vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()];
    let first = scene();
    let mut cache = RenderCache::default();
    cache.update(&first, 1.0, W, H, black());
    cache.finish();

    // Recolour a scattered subset so damage becomes many disjoint rects.
    let mut second = scene();
    for (i, command) in second[0].iter_mut().enumerate() {
        if i % 37 == 0
            && let Command::Rect { color, .. } = command
        {
            *color ^= 0xff_ffff;
        }
    }

    assert!(cache.update(&second, 1.0, W, H, black()));
    assert!(cache.damage().len() > 1, "expected disjoint damage rects");

    let mut glyphs = FxHashMap::default();
    let mut images = FxHashMap::default();
    prepare_caches(&second, &cache, 1.0, &fonts, &mut glyphs, &mut images);

    let mut reference = vec![0u32; W * H];
    clear_damage(&mut reference, W, cache.damage(), black());
    for prepared in cache.prepared() {
        let command = &second[prepared.layer][prepared.index];
        for region in cache.damage() {
            if prepared.bounds.intersects(*region) {
                draw_command(command, *region, &mut reference, W, H, 0, 1.0, &fonts, &glyphs, &images);
            }
        }
    }

    let mut banded = vec![0u32; W * H];
    raster_damage(
        &second,
        &cache,
        &mut banded,
        W,
        H,
        1.0,
        black(),
        &fonts,
        &mut glyphs,
        &mut images,
    );

    let mismatch = reference
        .iter()
        .zip(&banded)
        .position(|(a, b)| a != b)
        .map(|i| (i % W, i / W, reference[i], banded[i]));
    assert_eq!(mismatch, None, "banded output differs at (x, y, expected, got)");
}
