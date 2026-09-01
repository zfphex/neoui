use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use neoui::*;
use std::hash::Hasher;
use rustc_hash::FxHasher;

fn fxhash32(text: &str) -> u32 {
    let mut hasher = FxHasher::default();
    hasher.write(text.as_bytes());
    hasher.finish() as u32
}

fn generate_test_nodes(count: usize) -> Vec<SemanticNode> {
    let cols = 10;
    (0..count)
        .map(|i| {
            let row = i / cols;
            let col = i % cols;
            let bounds = Rect::new((col * 100) as i32, (row * 40) as i32, 90, 32);
            let label = format!("Button {i}");
            SemanticNode::new(
                bounds,
                0..8,
                RoleFlags::BUTTON,
                StateFlags::NONE,
                0,
                fxhash32(&label),
            )
        })
        .collect()
}

fn bench_sspa_tree_emission(c: &mut Criterion) {
    let counts = [10, 100, 1000];

    let mut group = c.benchmark_group("sspa_tree_emission");
    for &count in &counts {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &n| {
            b.iter(|| {
                let mut state = AccessabilityState::new();
                for i in 0..n {
                    let bounds = Rect::new((i % 10 * 100) as i32, (i / 10 * 40) as i32, 90, 32);
                    let start = state.text_arena.len() as u32;
                    state.text_arena.push_str("Button Label");
                    let end = state.text_arena.len() as u32;
                    state.current_nodes.push(SemanticNode::new(
                        bounds,
                        start..end,
                        RoleFlags::BUTTON,
                        StateFlags::NONE,
                        0,
                        0xAABBCCDD,
                    ));
                }
                black_box(state);
            });
        });
    }
    group.finish();
}

fn bench_sspa_snap_tiers(c: &mut Criterion) {
    let counts = [10, 100, 1000];

    // Tier 1: Exact containment hit test
    let mut group = c.benchmark_group("sspa_snap_tier_1_containment");
    for &count in &counts {
        let nodes = generate_test_nodes(count);
        let mid = count / 2;
        let target = &nodes[mid];
        let cursor_template = SpatialCursor::new(target.centroid(), target.role, target.text_signature, mid, 0);

        group.bench_with_input(BenchmarkId::from_parameter(count), &nodes, |b, nodes| {
            b.iter(|| {
                let mut cursor = cursor_template;
                black_box(snap_focus(black_box(nodes), &mut cursor, 200.0, None));
                black_box(cursor);
            });
        });
    }
    group.finish();

    // Tier 2: Shifted element within radius R (e.g. list reflow)
    let mut group = c.benchmark_group("sspa_snap_tier_2_shift");
    for &count in &counts {
        let nodes = generate_test_nodes(count);
        let mid = count / 2;
        let target = &nodes[mid];
        // Cursor is 50px away from target's shifted centroid
        let shifted_pt = (target.centroid().0 - 50.0, target.centroid().1);
        let cursor_template = SpatialCursor::new(shifted_pt, target.role, target.text_signature, mid, 0);

        group.bench_with_input(BenchmarkId::from_parameter(count), &nodes, |b, nodes| {
            b.iter(|| {
                let mut cursor = cursor_template;
                black_box(snap_focus(black_box(nodes), &mut cursor, 200.0, None));
                black_box(cursor);
            });
        });
    }
    group.finish();

    // Tier 3: Deletion fallback (nearest neighbor search)
    let mut group = c.benchmark_group("sspa_snap_tier_3_deletion");
    for &count in &counts {
        let nodes = generate_test_nodes(count);
        let mid = count / 2;
        let target = &nodes[mid];
        // Deleted item has a non-matching signature and is not contained in any node
        let deleted_pt = (target.centroid().0 + 5.0, target.centroid().1 + 5.0);
        let cursor_template = SpatialCursor::new(deleted_pt, target.role, 0xDEADBEEF, mid, 0);

        group.bench_with_input(BenchmarkId::from_parameter(count), &nodes, |b, nodes| {
            b.iter(|| {
                let mut cursor = cursor_template;
                black_box(snap_focus(black_box(nodes), &mut cursor, 200.0, None));
                black_box(cursor);
            });
        });
    }
    group.finish();
}

fn bench_sspa_navigation(c: &mut Criterion) {
    let counts = [10, 100, 1000];

    // Directional 2D navigation (Right Arrow)
    let mut group = c.benchmark_group("sspa_navigate_directional");
    for &count in &counts {
        let nodes = generate_test_nodes(count);
        let start_node = &nodes[0];
        let cursor_template = SpatialCursor::new(start_node.centroid(), start_node.role, start_node.text_signature, 0, 0);

        group.bench_with_input(BenchmarkId::from_parameter(count), &nodes, |b, nodes| {
            b.iter(|| {
                let mut cursor = cursor_template;
                black_box(navigate_directional(black_box(nodes), &mut cursor, Direction::Right, 2.0, None));
                black_box(cursor);
            });
        });
    }
    group.finish();

    // Sequential Tab navigation
    let mut group = c.benchmark_group("sspa_navigate_sequential");
    for &count in &counts {
        let nodes = generate_test_nodes(count);
        let start_node = &nodes[0];
        let cursor_template = SpatialCursor::new(start_node.centroid(), start_node.role, start_node.text_signature, 0, 0);

        group.bench_with_input(BenchmarkId::from_parameter(count), &nodes, |b, nodes| {
            b.iter(|| {
                let mut cursor = cursor_template;
                black_box(navigate_sequential(black_box(nodes), &mut cursor, true, None));
                black_box(cursor);
            });
        });
    }
    group.finish();
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
    targets = bench_sspa_tree_emission, bench_sspa_snap_tiers, bench_sspa_navigation
}
criterion_main!(benches);
