use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use merklith_crypto::{ed25519_verify, hash::hash, Keypair};
use merklith_types::{Address, Transaction, U256};

fn bench_address_to_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("address");
    let addr = Address::from_bytes([7u8; 20]);
    group.bench_function("to_string", |b| b.iter(|| black_box(addr.to_string())));
    group.finish();
}

fn bench_u256_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("u256");
    let a = U256::from(123456789u64);
    let b = U256::from(987654321u64);

    group.bench_function("checked_add", |bencher| {
        bencher.iter(|| black_box(a.checked_add(&b)))
    });
    group.bench_function("checked_mul", |bencher| {
        bencher.iter(|| black_box(a.checked_mul(&b)))
    });
    group.finish();
}

fn bench_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashing");
    let data = vec![0u8; 1024];
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("blake3_1kb", |b| b.iter(|| black_box(hash(&data))));
    group.finish();
}

fn bench_sign_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature");
    let keypair = Keypair::generate();
    let tx = Transaction::new(
        17001,
        1,
        Some(Address::from_bytes([1u8; 20])),
        U256::from(42u64),
        21_000,
        U256::from(1_000_000_000u64),
        U256::from(1_000_000_000u64),
    );
    let msg = tx.signing_hash();
    let sig = keypair.sign(msg.as_bytes());
    let pk = keypair.public_key();

    group.bench_function("sign", |b| b.iter(|| black_box(keypair.sign(msg.as_bytes()))));
    group.bench_function("verify", |b| {
        b.iter(|| black_box(ed25519_verify(&pk, msg.as_bytes(), &sig).is_ok()))
    });
    group.finish();
}

criterion_group!(benches, bench_address_to_string, bench_u256_ops, bench_hashing, bench_sign_verify);
criterion_main!(benches);
