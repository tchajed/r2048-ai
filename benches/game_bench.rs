use criterion::{black_box, criterion_group, criterion_main, Criterion};
use r2048_ai::ai::{rand_move, sum_tiles_score, weight_score};
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
    for i in 0..16 {
        s.set(i, (i % 4) as u8);
    }
    s
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("random game", |b| b.iter(|| random_game()));

    let s = test_state();
    c.bench_function("sum tiles", |b| b.iter(|| sum_tiles_score(&black_box(s))));
    c.bench_function("weight score", |b| b.iter(|| weight_score(&black_box(s))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
