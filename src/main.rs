use std::process;

use r2048_ai::{Algorithm, Config, Depth};

fn main() {
    let config = Config {
        algorithm: Algorithm::ExpectimaxWeight(Depth::Smart),
        target_score: Some(2048),
    };
    let win = config.run();
    if !win {
        eprintln!("failed to get to {}", config.target_score.unwrap());
        process::exit(1);
    }
}
