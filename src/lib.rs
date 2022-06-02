use rand::prelude::ThreadRng;
use rand::thread_rng;

pub mod ai;
pub mod game;

use game::State;

pub struct StateManager {
    rng: ThreadRng,
    s: State,
}

impl StateManager {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut s = State::default();
        s.rand_add(&mut rng);
        Self { rng, s }
    }

    pub fn state(&self) -> State {
        self.s
    }

    pub fn rng(&mut self) -> &mut ThreadRng {
        &mut self.rng
    }

    pub fn next_state(&mut self, s: State) {
        self.s = s;
        self.s.rand_add(&mut self.rng);
    }
}
