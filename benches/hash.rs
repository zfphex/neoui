use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

const HASH_SEED: u64 = 0xcbf2_9ce4_8422_2325;
const HASH_MIX: u64 = 0x9e37_79b9_7f4a_7c15;
const BYTE_LEN_TAG: u64 = 0x6c62_79f5_aa2d_4f1b;

#[inline]
fn mix(h: u64, v: u64) -> u64 {
    h.wrapping_mul(HASH_MIX).wrapping_add(v).rotate_left(27)
}

fn scalar(bytes: &[u8]) -> u64 {
    let mut hash = HASH_SEED;
    for &byte in bytes {
        hash = mix(hash, byte as u64);
    }
    mix(hash, BYTE_LEN_TAG ^ bytes.len() as u64)
}

fn chunked(bytes: &[u8]) -> u64 {
    let mut hash = HASH_SEED;
    let mut chunks = bytes.chunks_exact(8);
    for chunk in chunks.by_ref() {
        hash = mix(hash, u64::from_le_bytes(chunk.try_into().unwrap()));
    }
    for &byte in chunks.remainder() {
        hash = mix(hash, byte as u64);
    }
    mix(hash, BYTE_LEN_TAG ^ bytes.len() as u64)
}

fn bench_hash(c: &mut Criterion) {
    let sizes = [1, 4, 7, 8, 12, 16, 24, 32, 48, 64, 128, 256];

    let mut group = c.benchmark_group("normal_scalar");
    for &len in &sizes {
        let bytes = vec![0x5a; len];
        group.bench_with_input(BenchmarkId::from_parameter(len), &bytes, |b, bytes| {
            b.iter(|| black_box(scalar(black_box(bytes))));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("normal_chunked");
    for &len in &sizes {
        let bytes = vec![0x5a; len];
        group.bench_with_input(BenchmarkId::from_parameter(len), &bytes, |b, bytes| {
            b.iter(|| black_box(chunked(black_box(bytes))));
        });
    }
    group.finish();

    let bytes = vec![0x5a; 65536];
    c.bench_function("bad_case_scalar/65536", |b| {
        b.iter(|| black_box(scalar(black_box(&bytes))));
    });
    c.bench_function("bad_case_chunked/65536", |b| {
        b.iter(|| black_box(chunked(black_box(&bytes))));
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
    targets = bench_hash
}
criterion_main!(benches);
