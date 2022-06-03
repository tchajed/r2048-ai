use std::{io, time::Instant};

use ai::{expectimax_sum_move, expectimax_weight_move, rand_move, smart_depth};
use game::{Move, State};
use rand::{prelude::ThreadRng, Rng};
use std::io::Write;
use termcolor::{ColorChoice, StandardStream};

use crate::game::Game;

#[macro_use]
extern crate static_assertions;

pub mod ai;
pub mod game;

fn write_state(s: &State, stream: &mut StandardStream) -> io::Result<()> {
    for i in 0..4 {
        for j in 0..4 {
            let tile = s.tile(i * 4 + j);
            if tile == 1 {
                write!(stream, "     ")?;
            } else {
                write!(stream, "{:>4} ", tile)?;
            }
        }
        writeln!(stream)?;
    }
    Ok(())
}

fn print_state(s: &State) {
    write_state(s, &mut StandardStream::stdout(ColorChoice::AlwaysAnsi)).unwrap();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    Smart,
    Fixed(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    ExpectimaxSum(Depth),
    ExpectimaxWeight(Depth),
    Random,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub algorithm: Algorithm,
    pub target_score: Option<u32>,
}

impl Config {
    fn choose_depth(d: Depth, s: &State) -> u32 {
        match d {
            Depth::Smart => smart_depth(s),
            Depth::Fixed(d) => d,
        }
    }

    fn next_move(&self, s: &State) -> Option<(Move, State)> {
        match self.algorithm {
            Algorithm::ExpectimaxSum(d) => expectimax_sum_move(s, Self::choose_depth(d, s)),
            Algorithm::ExpectimaxWeight(d) => expectimax_weight_move(s, Self::choose_depth(d, s)),
            Algorithm::Random => rand_move(s, &mut ThreadRng::default()),
        }
    }

    /// Run runs the game and returns a score and whether or not this is a win.
    pub fn run(&self) -> bool {
        let mut mgr = Game::new();
        print_state(mgr.state());
        print!("{}", mgr.state());
        let start = Instant::now();
        while let Some((m, s)) = self.next_move(mgr.state()) {
            mgr.next_state(s);

            let moves = mgr.moves();
            if moves % 25 == 0 {
                let elapsed_s = start.elapsed().as_secs_f64();
                let moves_per_s = moves as f64 / elapsed_s;
                println!("  {:>4} {:?} {:0.0} moves/s", moves, m, moves_per_s);
            } else {
                println!("  {:>4} {:?}", mgr.moves(), m);
            }
            print_state(mgr.state());
            if let Some(target) = self.target_score {
                if mgr.state().highest_tile() == target {
                    break;
                }
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
        return self.won(&mgr);
    }

    pub fn won<R: Rng>(&self, g: &Game<R>) -> bool {
        if let Some(target) = self.target_score {
            g.state().highest_tile() >= target
        } else {
            true
        }
    }
}
