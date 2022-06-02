use r2048_ai::ai::expectimax_weight_move;
use r2048_ai::StateManager;

fn main() {
    let mut mgr = StateManager::new();
    print!("{}", mgr.state());
    while let Some((m, s)) = expectimax_weight_move(&mgr.state(), 2) {
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
