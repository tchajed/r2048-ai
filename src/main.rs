use r2048_ai::ai::{expectimax_move, weight_score};
use r2048_ai::game::State;
use r2048_ai::StateManager;

fn main() {
    let mut mgr = StateManager::new();
    let mut i = 0;
    print!("{}", mgr.state());
    while let Some((m, s)) = expectimax_move(&mgr.state(), 2, |s: &State| weight_score(&s)) {
        println!("{:?}", m);
        mgr.next_state(s);
        print!("{}", mgr.state());
        i += 1;
    }
    println!("score: {}  moves: {i}", mgr.state().highest_tile());
}
