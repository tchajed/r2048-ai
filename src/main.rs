use r2048_ai::ai::rand_move;
use r2048_ai::StateManager;

fn main() {
    let mut mgr = StateManager::new();
    let mut i = 0;
    print!("{}", mgr.state());
    loop {
        if let Some((m, s)) = rand_move(&mgr.state(), mgr.rng()) {
            println!("{:?}", m);
            mgr.next_state(s);
            print!("{}", mgr.state());
            i += 1;
        } else {
            break;
        }
    }
    println!("score: {}  moves: {i}", mgr.state().score());
}
