#[macro_use]
extern crate static_assertions;

use rand::prelude::ThreadRng;
use rand::thread_rng;
use rand::Rng;

pub mod ai;
pub mod game;

use game::State;

pub struct StateManager<R: Rng> {
    rng: R,
    s: State,
    moves: u32,
}

impl StateManager<ThreadRng> {
    pub fn new() -> Self {
        Self::from_rng(thread_rng())
    }
}

impl<R: Rng> StateManager<R> {
    pub fn from_rng(rng: R) -> Self {
        let mut rng = rng;
        let mut s = State::default();
        // game starts with two tiles
        s.rand_add(&mut rng);
        s.rand_add(&mut rng);
        Self { rng, s, moves: 0 }
    }

    pub fn state(&self) -> &State {
        &self.s
    }

    pub fn next_state(&mut self, s: State) {
        self.s = s;
        self.s.rand_add(&mut self.rng);
        self.moves += 1;
    }

    /// Get the number of moves made so far.
    pub fn moves(&self) -> u32 {
        self.moves
    }
}
