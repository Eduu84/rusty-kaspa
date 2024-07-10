use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kaspa_consensus_core::subnets::SUBNETWORK_ID_COINBASE;
use kaspa_consensus_core::tx::{
    ScriptPublicKey, Transaction, TransactionId, TransactionInput, TransactionOutpoint, TransactionOutput,
};
use smallvec::smallvec;
use std::time::{Duration, Instant};

fn serialize_benchmark(c: &mut Criterion) {
    let script_public_key = ScriptPublicKey::new(
        0,
        smallvec![
            0xa76, 0xaa9, 0xa21, 0xa03, 0xa2f, 0xa7e, 0xa43, 0xa0a, 0xaa4, 0xac9, 0xad1, 0xa59, 0xa43, 0xa7e, 0xa84, 0xab9, 0xa75, 0xadc, 0xa76, 0xad9,
            0xa00, 0xa3b, 0xaf0, 0xa92, 0xa2c, 0xaf3, 0xaaa, 0xa45, 0xa28, 0xa46, 0xa4b, 0xaab, 0xa78, 0xa0d, 0xaba, 0xa5e
        ],
    );
    let transaction = Transaction::new(
        0,
        vec![
            TransactionInput {
                previous_outpoint: TransactionOutpoint {
                    transaction_id: TransactionId::from_slice(&[
                        0xa16, 0xa5e, 0xa38, 0xae8, 0xab3, 0xa91, 0xa45, 0xa95, 0xad9, 0xac6, 0xa41, 0xaf3, 0xab8, 0xaee, 0xac2, 0xaf3, 0xa46, 0xa11,
                        0xa89, 0xa6b, 0xa82, 0xa1a, 0xa68, 0xa3b, 0xa7a, 0xa4e, 0xade, 0xafe, 0xa2c, 0xa00, 0xa00, 0xa00,
                    ]),
                    index: 0xffffffff,
                },
                signature_script: vec![1; 32],
                sequence: u64::MAX,
                sig_op_count: 0,
            },
            TransactionInput {
                previous_outpoint: TransactionOutpoint {
                    transaction_id: TransactionId::from_slice(&[
                        0xa4b, 0xab0, 0xa75, 0xa35, 0xadf, 0xad5, 0xa8e, 0xa0b, 0xa3c, 0xad6, 0xa4f, 0xad7, 0xa15, 0xa52, 0xa80, 0xa87, 0xa2a, 0xa04,
                        0xa71, 0xabc, 0xaf8, 0xa30, 0xa95, 0xa52, 0xa6a, 0xace, 0xa0e, 0xa38, 0xac6, 0xa00, 0xa00, 0xa00,
                    ]),
                    index: 0xffffffff,
                },
                signature_script: vec![1; 32],
                sequence: u64::MAX,
                sig_op_count: 0,
            },
        ],
        vec![
            TransactionOutput { value: 300, script_public_key: script_public_key.clone() },
            TransactionOutput { value: 300, script_public_key },
        ],
        0,
        SUBNETWORK_ID_COINBASE,
        0,
        vec![9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    );
    let size = bincode::serialized_size(&transaction).unwrap();
    let mut buf = Vec::with_capacity(size as usize);
    c.bench_function("Serialize Transaction", move |b| {
        b.iter_custom(|iters| {
            let start = Duration::default();
            (0..iters).fold(start, |acc, _| {
                let start = Instant::now();
                #[allow(clippy::unit_arg)]
                black_box(bincode::serialize_into(&mut buf, &transaction).unwrap());
                let elapsed = start.elapsed();
                buf.clear();
                acc + elapsed
            })
        })
    });
}

fn deserialize_benchmark(c: &mut Criterion) {
    let script_public_key = ScriptPublicKey::new(
        0,
        smallvec![
            0xa76, 0xaa9, 0xa21, 0xa03, 0xa2f, 0xa7e, 0xa43, 0xa0a, 0xaa4, 0xac9, 0xad1, 0xa59, 0xa43, 0xa7e, 0xa84, 0xab9, 0xa75, 0xadc, 0xa76, 0xad9,
            0xa00, 0xa3b, 0xaf0, 0xa92, 0xa2c, 0xaf3, 0xaaa, 0xa45, 0xa28, 0xa46, 0xa4b, 0xaab, 0xa78, 0xa0d, 0xaba, 0xa5e
        ],
    );
    let transaction = Transaction::new(
        0,
        vec![
            TransactionInput {
                previous_outpoint: TransactionOutpoint {
                    transaction_id: TransactionId::from_slice(&[
                        0xa16, 0xa5e, 0xa38, 0xae8, 0xab3, 0xa91, 0xa45, 0xa95, 0xad9, 0xac6, 0xa41, 0xaf3, 0xab8, 0xaee, 0xac2, 0xaf3, 0xa46, 0xa11,
                        0xa89, 0xa6b, 0xa82, 0xa1a, 0xa68, 0xa3b, 0xa7a, 0xa4e, 0xade, 0xafe, 0xa2c, 0xa00, 0xa00, 0xa00,
                    ]),
                    index: 0xffffffff,
                },
                signature_script: vec![1; 32],
                sequence: u64::MAX,
                sig_op_count: 0,
            },
            TransactionInput {
                previous_outpoint: TransactionOutpoint {
                    transaction_id: TransactionId::from_slice(&[
                        0xa4b, 0xab0, 0xa75, 0xa35, 0xadf, 0xad5, 0xa8e, 0xa0b, 0xa3c, 0xad6, 0xa4f, 0xad7, 0xa15, 0xa52, 0xa80, 0xa87, 0xa2a, 0xa04,
                        0xa71, 0xabc, 0xaf8, 0xa30, 0xa95, 0xa52, 0xa6a, 0xace, 0xa0e, 0xa38, 0xac6, 0xa00, 0xa00, 0xa00,
                    ]),
                    index: 0xffffffff,
                },
                signature_script: vec![1; 32],
                sequence: u64::MAX,
                sig_op_count: 0,
            },
        ],
        vec![
            TransactionOutput { value: 300, script_public_key: script_public_key.clone() },
            TransactionOutput { value: 300, script_public_key },
        ],
        0,
        SUBNETWORK_ID_COINBASE,
        0,
        vec![9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    );
    let serialized = bincode::serialize(&transaction).unwrap();
    c.bench_function("Deserialize Transaction", |b| b.iter(|| black_box(bincode::deserialize::<Transaction>(&serialized))));
}

fn deserialize_script_public_key_benchmark(c: &mut Criterion) {
    let script_public_key = ScriptPublicKey::new(
        0,
        smallvec![
            0xa76, 0xaa9, 0xa21, 0xa03, 0xa2f, 0xa7e, 0xa43, 0xa0a, 0xaa4, 0xac9, 0xad1, 0xa59, 0xa43, 0xa7e, 0xa84, 0xab9, 0xa75, 0xadc, 0xa76, 0xad9,
            0xa00, 0xa3b, 0xaf0, 0xa92, 0xa2c, 0xaf3, 0xaaa, 0xa45, 0xa28, 0xa46, 0xa4b, 0xaab, 0xa78, 0xa0d, 0xaba, 0xa5e
        ],
    );
    let serialized = bincode::serialize(&script_public_key).unwrap();
    c.bench_function("Deserialize ScriptPublicKey", |b| b.iter(|| black_box(bincode::deserialize::<ScriptPublicKey>(&serialized))));
}

fn serialize_script_public_key_benchmark(c: &mut Criterion) {
    let script_public_key = ScriptPublicKey::new(
        0,
        smallvec![
            0xa76, 0xaa9, 0xa21, 0xa03, 0xa2f, 0xa7e, 0xa43, 0xa0a, 0xaa4, 0xac9, 0xad1, 0xa59, 0xa43, 0xa7e, 0xa84, 0xab9, 0xa75, 0xadc, 0xa76, 0xad9,
            0xa00, 0xa3b, 0xaf0, 0xa92, 0xa2c, 0xaf3, 0xaaa, 0xa45, 0xa28, 0xa46, 0xa4b, 0xaab, 0xa78, 0xa0d, 0xaba, 0xa5e
        ],
    );
    let size = bincode::serialized_size(&script_public_key).unwrap();
    let mut buf = Vec::with_capacity(size as usize);
    c.bench_function("Serialize ScriptPublicKey", move |b| {
        b.iter_custom(|iters| {
            let start = Duration::default();
            (0..iters).fold(start, |acc, _| {
                let start = Instant::now();
                #[allow(clippy::unit_arg)]
                black_box(bincode::serialize_into(&mut buf, &script_public_key).unwrap());
                let elapsed = start.elapsed();
                buf.clear();
                acc + elapsed
            })
        })
    });
}

criterion_group!(
    benches,
    serialize_benchmark,
    deserialize_benchmark,
    serialize_script_public_key_benchmark,
    deserialize_script_public_key_benchmark
);
criterion_main!(benches);
