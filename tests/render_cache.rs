use neoui::*;

#[test]
fn half_open_bounds_do_not_touch_the_next_tile() {
    let mut cache = RenderCache::default();
    cache.begin_frame(128, 64, 1.0, 0);
    cache.add_command(PreparedCommand {
        layer: 0,
        index: 0,
        bounds: PhysicalRect::new(0, 0, 64, 64),
        hash: 1,
    });
    cache.force_full_redraw = false;
    cache.previous = cache.current.clone();

    cache.begin_frame(128, 64, 1.0, 0);
    cache.add_command(PreparedCommand {
        layer: 0,
        index: 0,
        bounds: PhysicalRect::new(0, 0, 64, 64),
        hash: 2,
    });
    assert_eq!(cache.compute_damage(), &[Rect::new(0, 0, 64, 64)]);
}

#[test]
fn equal_horizontal_runs_merge_vertically() {
    let mut cache = RenderCache::default();
    cache.begin_frame(192, 192, 1.0, 0);
    cache.force_full_redraw = false;
    cache.previous.clone_from(&cache.current);
    cache.current[0] ^= 1;
    cache.current[1] ^= 1;
    cache.current[3] ^= 1;
    cache.current[4] ^= 1;

    assert_eq!(cache.compute_damage(), &[Rect::new(0, 0, 128, 128)]);
}

#[test]
fn changed_dimensions_force_full_damage() {
    let mut cache = RenderCache::default();
    cache.begin_frame(100, 70, 2.0, 0);
    assert_eq!(cache.compute_damage(), &[Rect::new(0, 0, 100, 70)]);
    cache.complete_frame();
    cache.begin_frame(120, 70, 2.0, 0);
    assert_eq!(cache.compute_damage(), &[Rect::new(0, 0, 120, 70)]);
}

#[test]
fn clear_color_and_explicit_invalidation_force_damage() {
    let mut cache = RenderCache::default();
    cache.begin_frame(256, 128, 1.0, 1);
    cache.compute_damage();
    cache.complete_frame();

    cache.begin_frame(256, 128, 1.0, 1);
    assert!(cache.compute_damage().is_empty());
    cache.complete_frame();

    cache.begin_frame(256, 128, 1.0, 2);
    assert_eq!(cache.compute_damage(), &[Rect::new(0, 0, 256, 128)]);
    cache.complete_frame();

    cache.invalidate();
    cache.begin_frame(256, 128, 1.0, 2);
    assert_eq!(cache.compute_damage(), &[Rect::new(0, 0, 256, 128)]);
}

#[test]
fn signed_rectangles_clip_to_the_framebuffer() {
    assert_eq!(
        PhysicalRect::new(-10, -5, 20, 30).clamp_to_framebuffer(15, 25),
        PhysicalRect::new(0, 0, 15, 25)
    );
}
