//! AI to automatically play 2048.
use crate::game::{Move, State};
use rand::seq::SliceRandom;
use rand::Rng;

/// Generate a random legal move from `s`, and return the next state.
pub fn rand_move<R: Rng>(s: &State, rng: &mut R) -> Option<(Move, State)> {
    let moves = s.legal_moves();
    moves.choose(rng).map(|&m_s| m_s)
}

// compute the possible tile placements from a state, and their probabilities
// (which will sum to 1)
fn next_placements(s: &State) -> Vec<(f64, State)> {
    let poss = s.empty();
    let mut states = Vec::with_capacity(2 * poss.len());
    let base_prob = 1.0 / (poss.len() as f64);
    for &i in poss.iter() {
        // states where we add a 2 (= 2^1)
        let mut next_s = *s;
        next_s.set(i as usize, 1);
        let p = base_prob * State::TWO_SPAWN_PROB;
        states.push((p, next_s));
    }
    for &i in poss.iter() {
        // states where we add a 4 (= 2^2)
        let mut next_s = *s;
        next_s.set(i as usize, 2);
        let p = base_prob * State::FOUR_SPAWN_PROB;
        states.push((p, next_s));
    }
    states
}
