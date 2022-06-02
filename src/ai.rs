//! AI to automatically play 2048.
use crate::game::{Move, State};
use rand::seq::SliceRandom;
use rand::Rng;

pub fn rand_move<R: Rng>(s: &State, rng: &mut R) -> Option<(Move, State)> {
    let moves = s.legal_moves();
    moves.choose(rng).map(|&m_s| m_s)
}
