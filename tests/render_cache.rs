use neoui::*;

fn layers(commands: Vec<Command<'static>>) -> [Vec<Command<'static>>; 16] {
    let mut layers = std::array::from_fn(|_| Vec::new());
    layers[0] = commands;
    layers
}

fn rect(x: i32, y: i32, w: i32, h: i32, color: u32) -> Command<'static> {
    Command::Rect {
        bounds: Rect::new(x, y, w, h),
        clip: Rect::new(0, 0, 1920, 1080),
        color,
        radius: 0,
    }
}

#[test]
fn half_open_bounds_do_not_touch_the_next_tile() {
    let mut cache = RenderCache::default();
    let a = layers(vec![rect(0, 0, 64, 64, 1)]);
    assert!(cache.update(&a, 1.0, 128, 64, 0));
    cache.finish();

    let b = layers(vec![rect(0, 0, 64, 64, 2)]);
    assert!(cache.update(&b, 1.0, 128, 64, 0));
    assert_eq!(cache.damage(), &[Rect::new(0, 0, 64, 64)]);
}

#[test]
fn equal_horizontal_runs_merge_vertically() {
    // Two rows of two dirty tiles each → one vertically merged damage rect.
    let mut cache = RenderCache::default();
    let empty = layers(Vec::new());
    assert!(cache.update(&empty, 1.0, 192, 192, 0));
    cache.finish();

    // Stamp tiles (0,0),(1,0),(0,1),(1,1) via a 128×128 rect.
    let filled = layers(vec![rect(0, 0, 128, 128, 1)]);
    assert!(cache.update(&filled, 1.0, 192, 192, 0));
    assert_eq!(cache.damage(), &[Rect::new(0, 0, 128, 128)]);
}

#[test]
fn changed_dimensions_force_full_damage() {
    let mut cache = RenderCache::default();
    let empty = layers(Vec::new());
    assert!(cache.update(&empty, 1.0, 100, 70, 0));
    assert_eq!(cache.damage(), &[Rect::new(0, 0, 100, 70)]);
    cache.finish();

    assert!(cache.update(&empty, 1.0, 120, 70, 0));
    assert_eq!(cache.damage(), &[Rect::new(0, 0, 120, 70)]);
}

#[test]
fn clear_color_and_explicit_invalidation_force_damage() {
    let mut cache = RenderCache::default();
    let empty = layers(Vec::new());

    assert!(cache.update(&empty, 1.0, 256, 128, 1));
    cache.finish();

    assert!(!cache.update(&empty, 1.0, 256, 128, 1));
    cache.finish();

    assert!(cache.update(&empty, 1.0, 256, 128, 2));
    assert_eq!(cache.damage(), &[Rect::new(0, 0, 256, 128)]);
    cache.finish();

    cache.invalidate();
    assert!(cache.update(&empty, 1.0, 256, 128, 2));
    assert_eq!(cache.damage(), &[Rect::new(0, 0, 256, 128)]);
}

#[test]
fn signed_rectangles_clip_to_the_framebuffer() {
    assert_eq!(
        Rect::from_xyxy(-10, -5, 20, 30).clamp_to_size(15, 25),
        Rect::new(0, 0, 15, 25)
    );
}

#[test]
fn image_identity_and_style_changes_invalidate_only_occupied_tiles() {
    let image_a = Image::from_rgba8(1, 1, [255, 0, 0, 255]).unwrap();
    let image_b = Image::from_rgba8(1, 1, [0, 255, 0, 255]).unwrap();
    let image_command = |image: &'static Image, opacity| Command::Image {
        image,
        bounds: Rect::new(4, 4, 40, 40),
        clip: Rect::new(0, 0, 128, 64),
        fit: ImageFit::Stretch,
        opacity,
        radius: 0,
    };
    let image_a = Box::leak(Box::new(image_a));
    let image_b = Box::leak(Box::new(image_b));
    let mut cache = RenderCache::default();

    let first = layers(vec![image_command(image_a, 255)]);
    assert!(cache.update(&first, 1.0, 128, 64, 0));
    cache.finish();
    let identical = layers(vec![image_command(image_a, 255)]);
    assert!(!cache.update(&identical, 1.0, 128, 64, 0));
    cache.finish();
    let replaced = layers(vec![image_command(image_b, 255)]);
    assert!(cache.update(&replaced, 1.0, 128, 64, 0));
    assert_eq!(cache.damage(), &[Rect::new(0, 0, 64, 64)]);
    cache.finish();
    let faded = layers(vec![image_command(image_b, 128)]);
    assert!(cache.update(&faded, 1.0, 128, 64, 0));
}
