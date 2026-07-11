use divan::black_box;

const HASH_SEED: u64 = 0xcbf2_9ce4_8422_2325;
const HASH_MIX: u64 = 0x9e37_79b9_7f4a_7c15;
const BYTE_LEN_TAG: u64 = 0x6c62_79f5_aa2d_4f1b;

fn main() {
    divan::main();
}

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

#[divan::bench(args = [1, 4, 7, 8, 12, 16, 24, 32, 48, 64, 128, 256])]
fn normal_scalar(bencher: divan::Bencher, len: usize) {
    let bytes = vec![0x5a; len];
    bencher.bench(|| black_box(scalar(black_box(&bytes))));
}

#[divan::bench(args = [1, 4, 7, 8, 12, 16, 24, 32, 48, 64, 128, 256])]
fn normal_chunked(bencher: divan::Bencher, len: usize) {
    let bytes = vec![0x5a; len];
    bencher.bench(|| black_box(chunked(black_box(&bytes))));
}

#[divan::bench(args = [65536])]
fn bad_case_scalar(bencher: divan::Bencher, len: usize) {
    let bytes = vec![0x5a; len];
    bencher.bench(|| black_box(scalar(black_box(&bytes))));
}

#[divan::bench(args = [65536])]
fn bad_case_chunked(bencher: divan::Bencher, len: usize) {
    let bytes = vec![0x5a; len];
    bencher.bench(|| black_box(chunked(black_box(&bytes))));
}
