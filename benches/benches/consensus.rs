use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use merklith_consensus::{calculate_poc_score, select_committee};
use merklith_types::{Address, U256};
use merklith_crypto::{generate_keypair, vrf_prove};
use std::collections::HashMap;

fn bench_poc_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_poc");
    
    // Create test validators
    let validators: Vec<(Address, u64, u64, u64)> = (0..100)
        .map(|i| {
            let keypair = generate_keypair();
            let address = keypair.address();
            let stake = 10000u64 + (i * 1000) as u64;
            let reliability = 95u64 + (i % 5) as u64;
            let governance = 50u64 + (i % 50) as u64;
            (address, stake, reliability, governance)
        })
        .collect();
    
    group.bench_function("calculate_score_100_validators", |b| {
        b.iter(|| {
            for (address, stake, reliability, governance) in &validators {
                black_box(calculate_poc_score(*stake, *reliability, *governance));
            }
        })
    });
    
    group.bench_function("calculate_score_1000_validators", |b| {
        // Scale up to 1000 validators
        let large_validators: Vec<(Address, u64, u64, u64)> = (0..1000)
            .map(|i| {
                let keypair = generate_keypair();
                let address = keypair.address();
                let stake = 10000u64 + (i * 100) as u64;
                let reliability = 95u64 + (i % 5) as u64;
                let governance = 50u64 + (i % 50) as u64;
                (address, stake, reliability, governance)
            })
            .collect();
        
        b.iter(|| {
            for (address, stake, reliability, governance) in &large_validators {
                black_box(calculate_poc_score(*stake, *reliability, *governance));
            }
        })
    });
    
    group.finish();
}

fn bench_committee_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_committee");
    
    group.bench_function("select_10_from_100", |b| {
        b.iter_batched(
            || {
                let validators: HashMap<Address, u64> = (0..100)
                    .map(|i| {
                        let keypair = generate_keypair();
                        (keypair.address(), 10000u64 + (i * 1000) as u64)
                    })
                    .collect();
                let seed = [0u8; 32];
                (validators, seed)
            },
            |(validators, seed)| {
                black_box(select_committee(&validators, 10, &seed));
            },
            BatchSize::SmallInput,
        )
    });
    
    group.bench_function("select_32_from_1000", |b| {
        b.iter_batched(
            || {
                let validators: HashMap<Address, u64> = (0..1000)
                    .map(|i| {
                        let keypair = generate_keypair();
                        (keypair.address(), 10000u64 + (i * 100) as u64)
                    })
                    .collect();
                let seed = [0u8; 32];
                (validators, seed)
            },
            |(validators, seed)| {
                black_box(select_committee(&validators, 32, &seed));
            },
            BatchSize::SmallInput,
        )
    });
    
    group.finish();
}

fn bench_vrf_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_vrf");
    
    group.bench_function("prove", |b| {
        b.iter_batched(
            || {
                let keypair = generate_keypair();
                let message = b"test message for vrf";
                (keypair, message)
            },
            |(keypair, message)| {
                black_box(vrf_prove(&keypair.secret_key(), message));
            },
            BatchSize::SmallInput,
        )
    });
    
    group.bench_function("prove_and_verify", |b| {
        b.iter_batched(
            || {
                let keypair = generate_keypair();
                let message = b"test message for vrf";
                (keypair, message)
            },
            |(keypair, message)| {
                let (output, proof) = vrf_prove(&keypair.secret_key(), message).unwrap();
                black_box(merklith_crypto::vrf_verify(
                    &keypair.public_key(),
                    message,
                    &output,
                    &proof,
                ));
            },
            BatchSize::SmallInput,
        )
    });
    
    group.finish();
}

fn bench_signature_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_bls");
    
    use merklith_crypto::{bls_generate_keypair, bls_sign, bls_aggregate_signatures};
    
    group.bench_function("aggregate_10_signatures", |b| {
        b.iter_batched(
            || {
                let message = b"block_hash_12345";
                let signatures: Vec<_> = (0..10)
                    .map(|_| {
                        let keypair = bls_generate_keypair();
                        bls_sign(&keypair, message).unwrap()
                    })
                    .collect();
                signatures
            },
            |signatures| {
                black_box(bls_aggregate_signatures(&signatures).unwrap());
            },
            BatchSize::SmallInput,
        )
    });
    
    group.bench_function("aggregate_100_signatures", |b| {
        b.iter_batched(
            || {
                let message = b"block_hash_12345";
                let signatures: Vec<_> = (0..100)
                    .map(|_| {
                        let keypair = bls_generate_keypair();
                        bls_sign(&keypair, message).unwrap()
                    })
                    .collect();
                signatures
            },
            |signatures| {
                black_box(bls_aggregate_signatures(&signatures).unwrap());
            },
            BatchSize::SmallInput,
        )
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_poc_scoring,
    bench_committee_selection,
    bench_vrf_operations,
    bench_signature_aggregation
);
criterion_main!(benches);
