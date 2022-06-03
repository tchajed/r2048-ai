#![allow(clippy::needless_return)]
use std::{io, time::Instant};

use ai::{expectimax_sum_move, expectimax_weight_move, rand_move, smart_depth};
use game::{ArrayRow, Move, Row};
use rand::{prelude::ThreadRng, Rng};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::game::Game;

#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate lazy_static;

pub mod ai;
pub mod game;

type State = game::State<ArrayRow>;

fn gray_write<S: AsRef<str>>(stream: &mut StandardStream, s: S) -> io::Result<()> {
    _ = stream.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(200, 200, 200))));
    write!(stream, "{}", s.as_ref())?;
    stream.reset()?;
    Ok(())
}

// convenience that cleans up the code
fn gray_writeln<S: AsRef<str>>(stream: &mut StandardStream, s: S) -> io::Result<()> {
    gray_write(stream, s)?;
    writeln!(stream)?;
    Ok(())
}

fn write_state(s: &State, stream: &mut StandardStream) -> io::Result<()> {
    let sep = format!("+{bar}+{bar}+{bar}+{bar}+", bar = "------");
    gray_writeln(stream, &sep)?;
    for i in 0..4 {
        gray_write(stream, "|")?;
        for j in 0..4 {
            let tile = s.tile(i * 4 + j);
            if tile == 1 {
                write!(stream, "      ")?;
                stream.reset()?;
            } else {
                write!(stream, "{:>5} ", tile)?;
            }
            gray_write(stream, "|")?;
        }
        writeln!(stream)?;
        gray_writeln(stream, &sep)?;
    }
    Ok(())
}

fn print_state(s: &State) {
    write_state(s, &mut StandardStream::stdout(ColorChoice::Always))
        .expect("could not print colored state");
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
        let start = Instant::now();
        // current estimate
        let mut moves_per_s = 0.0;
        while let Some((_, s)) = self.next_move(mgr.state()) {
            mgr.next_state(s);

            _ = clearscreen::clear();
            let moves = mgr.moves();
            // generate an estimate early on, and then periodically
            if moves == 10 || moves % 50 == 0 {
                let elapsed_s = start.elapsed().as_secs_f64();
                moves_per_s = moves as f64 / elapsed_s;
            }
            println!("  {:>4} {:0.0} moves/s", moves, moves_per_s);
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

    pub fn won<R: Row, Rn: Rng>(&self, g: &Game<R, Rn>) -> bool {
        match self.target_score {
            Some(target) => g.state().highest_tile() >= target,
            None => true,
        }
    }
}
