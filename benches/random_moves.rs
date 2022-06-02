use criterion::{criterion_group, criterion_main, Criterion};
use r2048_ai::{ai::rand_move, StateManager};
use rand::{prelude::StdRng, SeedableRng};

fn random_game() -> u8 {
    let rng = StdRng::seed_from_u64(0);
    let mut mgr = StateManager::from_rng(rng);
    // run for exactly 100 iterations so timing is easy to interpret
    for _ in 0..100 {
        if let Some((_, s)) = rand_move(&mgr.state(), mgr.rng()) {
            mgr.next_state(s);
        } else {
            panic!("game went too short");
        }
    }
    return mgr.state().score();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("random game", |b| b.iter(|| random_game()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
