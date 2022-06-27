#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn bench(c: &mut Criterion) {
    c.bench_function("genTable", |b| {
        b.iter(|| black_box(smbc_incomplete::genTable()))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
