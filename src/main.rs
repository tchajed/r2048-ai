use clap::Parser;
use std::process;

use r2048_ai::{Algorithm, Config, Depth};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value = "weight")]
    algorithm: String,

    #[clap(short, long)]
    depth: Option<u32>,

    #[clap(short, long, default_value_t = 2048)]
    score: u32,

    #[clap(short, long)]
    unbounded: bool,
}

fn main() {
    let args = Args::parse();
    let depth = match args.depth {
        Option::Some(d) => Depth::Fixed(d),
        Option::None => Depth::Smart,
    };
    let algorithm = if args.algorithm == "weight" {
        Algorithm::ExpectimaxWeight(depth)
    } else if args.algorithm == "sum" {
        Algorithm::ExpectimaxSum(depth)
    } else if args.algorithm == "random" {
        Algorithm::Random
    } else {
        eprintln!("unknown algorithm {}", args.algorithm);
        process::exit(1);
    };
    let target_score = if args.unbounded {
        None
    } else {
        Some(args.score)
    };

    let config = Config {
        algorithm,
        target_score,
    };
    let win = config.run();
    if !win {
        eprintln!("failed to get to {}", config.target_score.unwrap());
        process::exit(1);
    }
}
