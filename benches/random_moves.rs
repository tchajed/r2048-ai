use criterion::{criterion_group, criterion_main, Criterion};
use r2048_ai::{ai::rand_move, StateManager};
use rand::{prelude::StdRng, SeedableRng};

fn random_game() -> u32 {
    let mut mgr = StateManager::from_rng(StdRng::seed_from_u64(0));
    let mut move_rng = StdRng::seed_from_u64(1);
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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("random game", |b| b.iter(|| random_game()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
