use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use merklith_types::{Address, U256, Transaction, BlockHeader};
use merklith_crypto::{generate_keypair, sign_transaction};
use blake3;

fn bench_address_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("address");
    
    // Generate test addresses
    let keypair = generate_keypair();
    let address = keypair.address();
    
    group.bench_function("from_public_key", |b| {
        b.iter(|| {
            black_box(Address::from_public_key(&keypair.public_key()));
        })
    });
    
    group.bench_function("bech32m_encode", |b| {
        b.iter(|| {
            black_box(address.to_string());
        })
    });
    
    group.finish();
}

fn bench_u256_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("u256");
    
    let a = U256::from(12345678901234567890u64);
    let b = U256::from(9876543210987654321u64);
    
    group.bench_function("add", |b| {
        b.iter(|| {
            black_box(a.checked_add(b));
        })
    });
    
    group.bench_function("mul", |b| {
        b.iter(|| {
            black_box(a.checked_mul(b));
        })
    });
    
    group.bench_function("div", |b| {
        b.iter(|| {
            black_box(a.checked_div(b));
        })
    });
    
    group.bench_function("isqrt", |b| {
        b.iter(|| {
            black_box(a.isqrt());
        })
    });
    
    group.finish();
}

fn bench_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashing");
    
    let data_32b = vec![0u8; 32];
    let data_1kb = vec![0u8; 1024];
    let data_1mb = vec![0u8; 1024 * 1024];
    
    group.throughput(Throughput::Bytes(32));
    group.bench_function("blake3_32b", |b| {
        b.iter(|| {
            black_box(blake3::hash(&data_32b));
        })
    });
    
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("blake3_1kb", |b| {
        b.iter(|| {
            black_box(blake3::hash(&data_1kb));
        })
    });
    
    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("blake3_1mb", |b| {
        b.iter(|| {
            black_box(blake3::hash(&data_1mb));
        })
    });
    
    group.finish();
}

fn bench_transaction_signing(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction");
    
    let keypair = generate_keypair();
    let tx = Transaction {
        nonce: 0,
        gas_price: U256::from(20000000000u64),
        gas_limit: 21000,
        to: Some(Address::from([0u8; 20])),
        value: U256::from(1000000000000000000u64),
        data: vec![].into(),
        chain_id: 17001,
        access_list: vec![],
    };
    
    group.bench_function("sign", |b| {
        b.iter(|| {
            black_box(sign_transaction(&tx, &keypair));
        })
    });
    
    group.finish();
}

fn bench_block_header_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("block");
    
    let header = BlockHeader {
        parent_hash: [0u8; 32].into(),
        state_root: [0u8; 32].into(),
        transactions_root: [0u8; 32].into(),
        receipts_root: [0u8; 32].into(),
        logs_bloom: [0u8; 256],
        number: 1000000,
        timestamp: 1700000000,
        gas_used: 15000000,
        gas_limit: 30000000,
        extra_data: vec![].into(),
    };
    
    group.bench_function("compute_hash", |b| {
        b.iter(|| {
            black_box(header.compute_hash());
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_address_operations,
    bench_u256_operations,
    bench_hashing,
    bench_transaction_signing,
    bench_block_header_hash
);
criterion_main!(benches);
