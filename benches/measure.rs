use criterion::{Criterion, black_box, criterion_group, criterion_main};
use neoui::*;
use rustc_hash::FxHashMap;

const LABELS: [&str; 16] = [
    "File",
    "Edit",
    "View",
    "Playback",
    "Library",
    "Help",
    "Play / Pause",
    "Stop",
    "Volume",
    "Shuffle",
    "Repeat",
    "Queue",
    "Settings",
    "About",
    "Search",
    "Now Playing",
];

const PARAGRAPH: &str = "Text wraps to the width the style asks for. A run that has no explicit width hugs its content instead, so wrapping only ever costs you what you opted into.";

fn bench_measure(c: &mut Criterion) {
    let fonts = vec![fontdue::Font::from_bytes(DEFAULT_FONT, fontdue::FontSettings::default()).unwrap()];
    let mut metrics = FxHashMap::default();
    let mut breaks = Vec::with_capacity(4096);

    for label in LABELS {
        measure_text(label, &fonts, 0, &[], 16, None, i32::MAX, &mut metrics, &mut breaks);
    }
    measure_text(PARAGRAPH, &fonts, 0, &[], 16, None, i32::MAX, &mut metrics, &mut breaks);

    // A frame's worth of text: mostly short labels, a few paragraphs.
    c.bench_function("measure_frame", |b| {
        b.iter(|| {
            breaks.clear();
            for _ in 0..8 {
                for label in LABELS {
                    black_box(measure_text(
                        black_box(label),
                        &fonts,
                        0,
                        &[],
                        16,
                        None,
                        i32::MAX,
                        &mut metrics,
                        &mut breaks,
                    ));
                }
            }
            for _ in 0..8 {
                black_box(measure_text(
                    black_box(PARAGRAPH),
                    &fonts,
                    0,
                    &[],
                    16,
                    None,
                    i32::MAX,
                    &mut metrics,
                    &mut breaks,
                ));
            }
        });
    });

    c.bench_function("measure_frame_wrapped", |b| {
        b.iter(|| {
            breaks.clear();
            for _ in 0..8 {
                black_box(measure_text(
                    black_box(PARAGRAPH),
                    &fonts,
                    0,
                    &[],
                    16,
                    None,
                    300,
                    &mut metrics,
                    &mut breaks,
                ));
            }
        });
    });
}

fn fast_criterion() -> Criterion {
    Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(3))
        .sample_size(200)
}

criterion_group! {
    name = benches;
    config = fast_criterion();
    targets = bench_measure
}
criterion_main!(benches);
