use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use r2048_ai::ai::{
    expectimax_sum_move, expectimax_weight_move, rand_move, sum_tiles_score, weight_score,
};
use r2048_ai::game::State;
use r2048_ai::StateManager;
use rand::{prelude::StdRng, SeedableRng};

fn random_game() -> u32 {
    let mut mgr = StateManager::from_rng(StdRng::seed_from_u64(0));
    let mut move_rng = StdRng::seed_from_u64(2);
    // run for exactly 100 iterations so timing is easy to interpret
    for _ in 0..100 {
        if let Some((_, s)) = rand_move(&mgr.state(), &mut move_rng) {
            mgr.next_state(s);
        } else {
            panic!("game went too short");
        }
    }
    return mgr.state().highest_tile();
}

fn test_state() -> State {
    let mut s = State::default();
    let values: Vec<u8> = vec![0, 0, 1, 1, 0, 1, 2, 3, 0, 1, 2, 3, 3, 6, 9, 10];
    for (i, x) in values.into_iter().enumerate() {
        s.set(i, x);
    }
    s
}

fn sparse_state() -> State {
    let mut s = State::default();
    let values: Vec<u8> = vec![0, 1, 3, 8, 0, 0, 4, 2, 0, 1, 0, 0, 0, 0, 0, 0];
    for (i, x) in values.into_iter().enumerate() {
        s.set(i, x);
    }
    s
}

fn small_criterion_benchmarks(c: &mut Criterion) {
    c.bench_function("random game", |b| b.iter(|| random_game()));

    let s = test_state();
    c.bench_function("sum tiles", |b| b.iter(|| sum_tiles_score(&black_box(s))));
    c.bench_function("weight score", |b| b.iter(|| weight_score(&black_box(s))));
}

fn expectimax_benchmarks(c: &mut Criterion) {
    let s = sparse_state();
    c.bench_function("expectimax sum-2", |b| {
        b.iter(|| expectimax_sum_move(&black_box(s), 2))
    });
    c.bench_function("expectimax weight-2", |b| {
        b.iter(|| expectimax_weight_move(&black_box(s), 2))
    });

    let s = test_state();
    c.bench_function("expectimax sum-3", |b| {
        b.iter(|| expectimax_sum_move(&black_box(s), 3))
    });
    c.bench_function("expectimax weight-3", |b| {
        b.iter(|| expectimax_weight_move(&black_box(s), 3))
    });
}

criterion_group!(microbenches, small_criterion_benchmarks);
criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(30).measurement_time(Duration::from_secs(10));
    targets = expectimax_benchmarks
);
criterion_main!(microbenches, benches);
