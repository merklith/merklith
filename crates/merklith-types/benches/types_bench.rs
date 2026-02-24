use merklith_types::U256;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_u256_arithmetic(c: &mut Criterion) {
    let a = U256::from(u64::MAX);
    let b = U256::from(u64::MAX / 2);

    c.bench_function("u256_add", |bencher| bencher.iter(|| a.checked_add(&b)));
    c.bench_function("u256_mul", |bencher| bencher.iter(|| a.checked_mul(&b)));
    c.bench_function("u256_isqrt", |bencher| bencher.iter(|| a.isqrt()));
}

fn bench_u256_conversion(c: &mut Criterion) {
    let val = U256::from(0x1234567890abcdef_1122334455667788u128);

    c.bench_function("u256_to_be_bytes", |bencher| bencher.iter(|| val.to_be_bytes()));
    c.bench_function("u256_from_be_bytes", |bencher| {
        let bytes = val.to_be_bytes();
        bencher.iter(|| U256::from_be_bytes(bytes))
    });
}

fn bench_address(c: &mut Criterion) {
    use merklith_types::Address;

    let pubkey = [42u8; 32];
    let addr = Address::from_public_key(&pubkey);

    c.bench_function("address_from_pubkey", |bencher| {
        bencher.iter(|| Address::from_public_key(&pubkey))
    });
    c.bench_function("address_bech32m_encode", |bencher| bencher.iter(|| addr.to_string()));
    c.bench_function("address_bech32m_decode", |bencher| {
        let s = addr.to_string();
        bencher.iter(|| s.parse::<Address>())
    });
}

fn bench_hash(c: &mut Criterion) {
    use merklith_types::Hash;

    let data_32b = vec![0u8; 32];
    let data_1kb = vec![0u8; 1024];
    let data_1mb = vec![0u8; 1024 * 1024];

    c.bench_function("hash_32b", |bencher| bencher.iter(|| Hash::compute(&data_32b)));
    c.bench_function("hash_1kb", |bencher| bencher.iter(|| Hash::compute(&data_1kb)));
    c.bench_function("hash_1mb", |bencher| bencher.iter(|| Hash::compute(&data_1mb)));
}

criterion_group!(
    benches,
    bench_u256_arithmetic,
    bench_u256_conversion,
    bench_address,
    bench_hash
);
criterion_main!(benches);
