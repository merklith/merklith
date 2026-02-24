use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use merklith_consensus::{Attestation, AttestationPool, ContributionTracker, ValidatorSet};
use merklith_types::Address;

fn bench_contribution_tracker(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_contributions");

    group.bench_function("record_1k_contributions", |b| {
        b.iter_batched(
            ContributionTracker::new,
            |mut tracker| {
                for i in 0..1000 {
                    let addr = Address::from_bytes([(i % 255) as u8; 20]);
                    tracker.record_block_production(addr, i as u64);
                }
                black_box(tracker.total_contributions());
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_validator_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_validator_set");

    group.bench_function("select_proposer_poc", |b| {
        b.iter_batched(
            || {
                let mut set = ValidatorSet::new();
                for i in 0..200 {
                    let addr = Address::from_bytes([(i % 255) as u8; 20]);
                    set.add_validator(addr, 1_000_000);
                    set.contribution_tracker_mut().record_block_production(addr, i as u64);
                }
                set
            },
            |set| black_box(set.select_proposer_poc(42)),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_attestation_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_attestations");

    group.bench_function("add_and_finalize", |b| {
        b.iter_batched(
            || {
                let pool = AttestationPool::new().with_threshold(16);
                let block_hash = [9u8; 32];
                (pool, block_hash)
            },
            |(mut pool, block_hash)| {
                for i in 0..16 {
                    let addr = Address::from_bytes([(i + 1) as u8; 20]);
                    let att = Attestation::new(100, block_hash, addr, vec![1, 2, 3]);
                    pool.add_attestation(att);
                }
                black_box(pool.check_finality(100, block_hash));
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_contribution_tracker, bench_validator_selection, bench_attestation_pool);
criterion_main!(benches);
