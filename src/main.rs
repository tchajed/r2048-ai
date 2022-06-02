use r2048_ai::{rand_move, State};
use rand::thread_rng;

fn main() {
    let mut rng = thread_rng();
    let mut s = State::default();
    s.rand_add(&mut rng);
    print!("{s}");
    loop {
        if let Some((m, next_s)) = rand_move(&s, &mut rng) {
            println!("{:?}", m);
            s = next_s;
            s.rand_add(&mut rng);
            print!("{s}");
        } else {
            break;
        }
    }
    println!("score: {}", s.score());
}
