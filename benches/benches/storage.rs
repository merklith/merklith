use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use merklith_crypto::MerkleTree;
use merklith_storage::{state_db::StateDB, Database};
use merklith_types::{Address, Hash, U256};
use tempfile::TempDir;

fn bench_database(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_db");

    group.bench_function("put_1k_entries", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let db = Database::new(temp_dir.path()).unwrap();
                (db, (0..1000).map(|i| (format!("k{i}").into_bytes(), format!("v{i}").into_bytes())).collect::<Vec<_>>())
            },
            |(db, data)| {
                for (k, v) in data {
                    db.put("default", &k, &v).unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_state_db(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_state_db");

    group.bench_function("transfer", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let state = StateDB::new(temp_dir.path()).unwrap();
                let from = Address::from_bytes([1u8; 20]);
                let to = Address::from_bytes([2u8; 20]);
                state.set_balance(&from, U256::from(1_000_000u64)).unwrap();
                (state, from, to)
            },
            |(state, from, to)| {
                black_box(state.transfer(&from, &to, U256::from(1u64)).unwrap());
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_merkle(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_merkle");
    let leaves: Vec<Hash> = (0..512)
        .map(|i| Hash::compute(format!("leaf_{i}").as_bytes()))
        .collect();

    group.bench_function("build_tree_512", |b| {
        b.iter(|| {
            let tree = MerkleTree::from_leaves(&leaves);
            black_box(tree.root())
        })
    });

    let tree = MerkleTree::from_leaves(&leaves);
    group.bench_function("proof_256", |b| {
        b.iter(|| black_box(tree.proof(256).unwrap()))
    });

    group.finish();
}

criterion_group!(benches, bench_database, bench_state_db, bench_merkle);
criterion_main!(benches);
