use criterion::{criterion_group, criterion_main, Criterion};

use lib::days::day15::hash_generic;

pub fn bench_hash_u16(c: &mut Criterion) {
    c.bench_function("hash u16", |b| {
        b.iter(|| hash_generic::<u16>("hello, world."))
    });
}

pub fn bench_hash_u32(c: &mut Criterion) {
    c.bench_function("hash u32", |b| {
        b.iter(|| hash_generic::<u32>("hello, world."))
    });
}

pub fn bench_hash_u64(c: &mut Criterion) {
    c.bench_function("hash u64", |b| {
        b.iter(|| hash_generic::<u64>("hello, world."))
    });
}

criterion_group!(benches, bench_hash_u16, bench_hash_u32, bench_hash_u64);
criterion_main!(benches);
