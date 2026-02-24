use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use merklith_storage::{Database, StateDB};
use merklith_types::{Address, Account, U256};
use tempfile::TempDir;

fn bench_database_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("database");
    
    group.bench_function("put_1k_entries", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let db = Database::open(temp_dir.path()).unwrap();
                let data: Vec<(Vec<u8>, Vec<u8>)> = (0..1000)
                    .map(|i| (format!("key_{}", i).into_bytes(), format!("value_{}", i).into_bytes()))
                    .collect();
                (db, data)
            },
            |(db, data)| {
                for (key, value) in data {
                    db.put(b"default", &key, &value).unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });
    
    group.bench_function("get_1k_entries", |b| {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        let keys: Vec<Vec<u8>> = (0..1000)
            .map(|i| format!("key_{}", i).into_bytes())
            .collect();
        
        // Pre-populate
        for key in &keys {
            db.put(b"default", key, b"value").unwrap();
        }
        
        b.iter(|| {
            for key in &keys {
                black_box(db.get(b"default", key).unwrap());
            }
        })
    });
    
    group.finish();
}

fn bench_state_db_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_db");
    
    group.bench_function("create_account", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let db = Database::open(temp_dir.path()).unwrap();
                let state_db = StateDB::new(db);
                let address = Address::from([0u8; 20]);
                let account = Account::new(U256::from(1000u64), 0);
                (state_db, address, account)
            },
            |(mut state_db, address, account)| {
                state_db.create_account(address, account).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
    
    group.bench_function("get_account", |b| {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        let mut state_db = StateDB::new(db);
        let address = Address::from([0u8; 20]);
        let account = Account::new(U256::from(1000u64), 0);
        state_db.create_account(address, account).unwrap();
        
        b.iter(|| {
            black_box(state_db.get_account(address).unwrap());
        })
    });
    
    group.bench_function("update_balance", |b| {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        let mut state_db = StateDB::new(db);
        let address = Address::from([0u8; 20]);
        let account = Account::new(U256::from(1000u64), 0);
        state_db.create_account(address, account).unwrap();
        
        b.iter(|| {
            let mut account = state_db.get_account(address).unwrap().unwrap();
            account.balance = account.balance + U256::from(1u64);
            state_db.update_account(address, account).unwrap();
        })
    });
    
    group.finish();
}

fn bench_trie_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("trie");
    
    group.bench_function("insert_100_leaves", |b| {
        b.iter_batched(
            || {
                use merklith_storage::MerkleTree;
                let leaves: Vec<[u8; 32]> = (0..100)
                    .map(|i| blake3::hash(format!("leaf_{}", i).as_bytes()).into())
                    .collect();
                leaves
            },
            |leaves| {
                let tree = MerkleTree::from_leaves(&leaves);
                black_box(tree.root());
            },
            BatchSize::SmallInput,
        )
    });
    
    group.bench_function("proof_100_leaves", |b| {
        let leaves: Vec<[u8; 32]> = (0..100)
            .map(|i| blake3::hash(format!("leaf_{}", i).as_bytes()).into())
            .collect();
        let tree = MerkleTree::from_leaves(&leaves);
        
        b.iter(|| {
            black_box(tree.proof(50).unwrap());
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_database_operations,
    bench_state_db_operations,
    bench_trie_operations
);
criterion_main!(benches);
