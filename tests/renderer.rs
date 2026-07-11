use std::borrow::Cow;

use neoui::*;
use rustc_hash::FxHashMap;

fn layers(commands: Vec<Command<'static>>) -> [Vec<Command<'static>>; 16] {
    let mut layers = std::array::from_fn(|_| Vec::new());
    layers[0] = commands;
    layers
}

fn render_full(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    clear: u32,
    commands: &[Vec<Command<'static>>; 16],
    fonts: &[fontdue::Font],
    bitmaps: &mut FxHashMap<usize, FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    image_cache: &mut FxHashMap<ImageKey, ImageEntry>,
) {
    buffer.fill(clear);
    let damage = Rect::new(0, 0, width as i32, height as i32);
    for layer in commands {
        for command in layer {
            draw_command(command, damage, buffer, width, height, 1.0, fonts, bitmaps, image_cache);
        }
    }
}

fn render_cached(
    cache: &mut RenderCache,
    buffer: &mut [u32],
    width: usize,
    height: usize,
    clear: u32,
    commands: &[Vec<Command<'static>>; 16],
    fonts: &[fontdue::Font],
    bitmaps: &mut FxHashMap<usize, FxHashMap<(char, usize), (fontdue::Metrics, Vec<u8>)>>,
    image_cache: &mut FxHashMap<ImageKey, ImageEntry>,
) -> Vec<Rect> {
    let dirty = cache.update(commands, 1.0, width, height, clear);
    let damage = cache.damage().to_vec();
    if dirty {
        clear_damage(buffer, width, &damage, clear);
        raster_damage(
            commands,
            cache.prepared(),
            &damage,
            buffer,
            width,
            height,
            1.0,
            fonts,
            bitmaps,
            image_cache,
        );
    }
    cache.finish();
    damage
}

fn rect(x: i32, y: i32, width: i32, height: i32, color: u32) -> Command<'static> {
    Command::Rect {
        bounds: Rect::new(x, y, width, height),
        clip: Rect::new(0, 0, 256, 192),
        color,
        radius: 0,
    }
}

#[test]
fn cached_rectangles_match_full_redraw_across_changes() {
    let (width, height) = (256, 192);
    let clear = rgb(3, 5, 7);
    let fonts = vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()];
    let mut cache = RenderCache::default();
    let mut cached = vec![0; width * height];
    let mut full = vec![0; width * height];
    let mut cached_bitmaps = FxHashMap::default();
    let mut full_bitmaps = FxHashMap::default();
    let mut cached_images = FxHashMap::default();
    let mut full_images = FxHashMap::default();

    let frames = [
        layers(vec![rect(5, 7, 50, 40, rgb(200, 0, 0))]),
        layers(vec![rect(5, 7, 50, 40, rgb(200, 0, 0))]),
        layers(vec![rect(105, 70, 50, 40, rgb(200, 0, 0))]),
        layers(vec![
            rect(105, 70, 50, 40, rgb(200, 0, 0)),
            rect(120, 80, 70, 50, rgb(0, 120, 220)),
        ]),
        layers(Vec::new()),
    ];

    for (index, commands) in frames.iter().enumerate() {
        let damage = render_cached(
            &mut cache,
            &mut cached,
            width,
            height,
            clear,
            commands,
            &fonts,
            &mut cached_bitmaps,
            &mut cached_images,
        );
        render_full(
            &mut full,
            width,
            height,
            clear,
            commands,
            &fonts,
            &mut full_bitmaps,
            &mut full_images,
        );
        assert_eq!(cached, full, "pixel mismatch on frame {index}");
        if index == 1 {
            assert!(damage.is_empty(), "identical frame should have no damage");
        }
    }
}

#[test]
fn cached_outlines_and_triangles_match_full_redraw() {
    let (width, height) = (256, 192);
    let fonts = vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()];
    let mut cache = RenderCache::default();
    let mut cached = vec![0; width * height];
    let mut full = vec![0; width * height];
    let mut cached_bitmaps = FxHashMap::default();
    let mut full_bitmaps = FxHashMap::default();
    let mut cached_images = FxHashMap::default();
    let mut full_images = FxHashMap::default();

    for offset in [0, 70] {
        let commands = layers(vec![
            Command::RectOutline {
                bounds: Rect::new(20 + offset, 20, 80, 60),
                clip: Rect::new(0, 0, width as i32, height as i32),
                color: rgb(100, 180, 220),
                radius: 0,
                border_thickness: 1,
                border_sides: border::ALL,
            },
            Command::Triangle {
                a: (30 + offset, 100),
                b: (80 + offset, 140),
                c: (110 + offset, 90),
                clip: Rect::new(0, 0, width as i32, height as i32),
                color: rgb(220, 100, 40),
            },
        ]);
        render_cached(
            &mut cache,
            &mut cached,
            width,
            height,
            black(),
            &commands,
            &fonts,
            &mut cached_bitmaps,
            &mut cached_images,
        );
        render_full(
            &mut full,
            width,
            height,
            black(),
            &commands,
            &fonts,
            &mut full_bitmaps,
            &mut full_images,
        );
        assert_eq!(cached, full);
    }
}

#[test]
fn cached_text_and_clipping_match_full_redraw() {
    let (width, height) = (256, 192);
    let fonts = vec![fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap()];
    let mut cache = RenderCache::default();
    let mut cached = vec![0; width * height];
    let mut full = vec![0; width * height];
    let mut cached_bitmaps = FxHashMap::default();
    let mut full_bitmaps = FxHashMap::default();
    let mut cached_images = FxHashMap::default();
    let mut full_images = FxHashMap::default();
    let mut layout_metrics = FxHashMap::default();

    for (text, x, clip) in [
        ("jazz\nÅngström", -2, Rect::new(0, 0, 180, 100)),
        ("jazz!\nÅngström", 50, Rect::new(30, 0, 120, 100)),
    ] {
        let layout_bounds = measure_text(text, &fonts[0], 20, &mut layout_metrics);
        let commands = layers(vec![Command::Text {
            text: Cow::Owned(text.to_owned()),
            font_id: 0,
            clip,
            bounds: Rect::new(x, 5, layout_bounds.width, layout_bounds.height),
            color: white(),
            size: 20,
        }]);
        render_cached(
            &mut cache,
            &mut cached,
            width,
            height,
            black(),
            &commands,
            &fonts,
            &mut cached_bitmaps,
            &mut cached_images,
        );
        render_full(
            &mut full,
            width,
            height,
            black(),
            &commands,
            &fonts,
            &mut full_bitmaps,
            &mut full_images,
        );
        assert_eq!(cached, full);
    }
}

#[test]
fn images_blend_clip_fit_and_match_cached_rendering() {
    let (width, height) = (96, 80);
    let image = Box::leak(Box::new(
        Image::from_rgba8(2, 1, [255, 0, 0, 255, 0, 255, 0, 128]).unwrap(),
    ));
    let fonts = Vec::new();
    let mut cache = RenderCache::default();
    let mut cached = vec![rgb(0, 0, 255); width * height];
    let mut full = cached.clone();
    let mut cached_bitmaps = FxHashMap::default();
    let mut full_bitmaps = FxHashMap::default();
    let mut cached_images = FxHashMap::default();
    let mut full_images = FxHashMap::default();

    for (fit, opacity, radius, x) in [
        (ImageFit::Stretch, 255, 0, -4),
        (ImageFit::Contain, 220, 4, 8),
        (ImageFit::Cover, 160, 12, 18),
    ] {
        let commands = layers(vec![Command::Image {
            image,
            bounds: Rect::new(x, 10, 50, 50),
            clip: Rect::new(3, 4, 70, 62),
            fit,
            opacity,
            radius,
        }]);
        render_cached(
            &mut cache,
            &mut cached,
            width,
            height,
            rgb(0, 0, 255),
            &commands,
            &fonts,
            &mut cached_bitmaps,
            &mut cached_images,
        );
        render_full(
            &mut full,
            width,
            height,
            rgb(0, 0, 255),
            &commands,
            &fonts,
            &mut full_bitmaps,
            &mut full_images,
        );
        assert_eq!(cached, full);
    }
}
