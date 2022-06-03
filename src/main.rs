use std::time::Instant;

use r2048_ai::{
    ai::{expectimax_weight_move, smart_depth},
    game::Game,
};

fn main() {
    let mut mgr = Game::new();
    print!("{}", mgr.state());
    let start = Instant::now();
    while let Some((m, s)) = expectimax_weight_move(mgr.state(), smart_depth(mgr.state())) {
        mgr.next_state(s);

        let moves = mgr.moves();
        if moves % 25 == 0 {
            let elapsed_s = start.elapsed().as_secs_f64();
            let moves_per_s = moves as f64 / elapsed_s;
            println!("  {:>4} {:?} {:0.0} moves/s", moves, m, moves_per_s);
        } else {
            println!("  {:>4} {:?}", mgr.moves(), m);
        }
        print!("{}", mgr.state());
        if mgr.state().highest_tile() == 2048 {
            break;
        }
    }

    let elapsed_s = start.elapsed().as_secs_f64();
    let moves_per_s = mgr.moves() as f64 / elapsed_s;
    println!(
        "score: {score}  moves: {moves}  {moves_per_s:0.0} moves/s",
        score = mgr.state().highest_tile(),
        moves = mgr.moves(),
        moves_per_s = moves_per_s,
    );
}
