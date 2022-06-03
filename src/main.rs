use r2048_ai::ai::{expectimax_weight_move, smart_depth};
use r2048_ai::StateManager;

fn main() {
    let mut mgr = StateManager::new();
    print!("{}", mgr.state());
    while let Some((m, s)) = expectimax_weight_move(mgr.state(), smart_depth(mgr.state())) {
        println!("{:?}", m);
        mgr.next_state(s);
        print!("{}", mgr.state());
    }
    println!(
        "score: {score}  moves: {moves}",
        score = mgr.state().highest_tile(),
        moves = mgr.moves()
    );
}
