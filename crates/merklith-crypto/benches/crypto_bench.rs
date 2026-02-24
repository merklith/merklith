use criterion::{criterion_group, criterion_main, Criterion};

fn bench_ed25519(c: &mut Criterion) {
    // Placeholder benchmarks
    c.bench_function("ed25519_sign", |b| b.iter(|| {}));
    c.bench_function("ed25519_verify", |b| b.iter(|| {}));
}

criterion_group!(benches, bench_ed25519);
criterion_main!(benches);
