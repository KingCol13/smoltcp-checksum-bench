use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let data = vec![254u8; 4096];

    c.bench_function("original 64", |b| b.iter(|| checksum::checksum_original(black_box(&data[..64]))));
    c.bench_function("original 1024", |b| b.iter(|| checksum::checksum_original(black_box(&data[..1024]))));

    c.bench_function("indexed 64", |b| b.iter(|| checksum::checksum_indexed(black_box(&data[..64]))));
    c.bench_function("indexed 1024", |b| b.iter(|| checksum::checksum_indexed(black_box(&data[..1024]))));

    c.bench_function("chunks_exact 64", |b| b.iter(|| checksum::checksum_chunks_exact(black_box(&data[..64]))));
    c.bench_function("chunks_exact 1024", |b| b.iter(|| checksum::checksum_chunks_exact(black_box(&data[..1024]))));

    c.bench_function("chunks_exact_no_bigchunk 64", |b| b.iter(|| checksum::checksum_chunks_exact_no_bigchunk(black_box(&data[..64]))));
    c.bench_function("chunks_exact_no_bigchunk 1024", |b| b.iter(|| checksum::checksum_chunks_exact_no_bigchunk(black_box(&data[..1024]))));

    c.bench_function("sliced_ne 64", |b| b.iter(|| checksum::checksum_sliced_ne(black_box(&data[..64]))));
    c.bench_function("sliced_ne 1024", |b| b.iter(|| checksum::checksum_sliced_ne(black_box(&data[..1024]))));

    c.bench_function("sliced_ne_sep 64", |b| b.iter(|| checksum::checksum_sliced_ne_sep(black_box(&data[..64]))));
    c.bench_function("sliced_ne_sep 1024", |b| b.iter(|| checksum::checksum_sliced_ne_sep(black_box(&data[..1024]))));

    c.bench_function("sliced_ne_u16 64", |b| b.iter(|| checksum::checksum_sliced_ne_u16(black_box(&data[..64]))));
    c.bench_function("sliced_ne_u16 1024", |b| b.iter(|| checksum::checksum_sliced_ne_u16(black_box(&data[..1024]))));

    c.bench_function("chunks_ne_u16 64", |b| b.iter(|| checksum::checksum_chunks_ne_u16(black_box(&data[..64]))));
    c.bench_function("chunks_ne_u16 1024", |b| b.iter(|| checksum::checksum_chunks_ne_u16(black_box(&data[..1024]))));

    c.bench_function("wide 64", |b| b.iter(|| checksum::checksum_wide(black_box(&data[..64]))));
    c.bench_function("wide 1024", |b| b.iter(|| checksum::checksum_wide(black_box(&data[..1024]))));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
