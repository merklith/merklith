use criterion::{criterion_group, criterion_main, Criterion};

fn bench_storage(c: &mut Criterion) {
    c.bench_function("storage_placeholder", |b| b.iter(|| {}));
}

criterion_group!(benches, bench_storage);
criterion_main!(benches);
