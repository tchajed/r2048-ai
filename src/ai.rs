//! AI to automatically play 2048.
use std::cmp::Ordering;

use crate::game::{Move, State};
use rand::seq::SliceRandom;
use rand::Rng;

/// Generate a random legal move from `s`, and return the next state.
pub fn rand_move<R: Rng>(s: &State, rng: &mut R) -> Option<(Move, State)> {
    let moves = s.legal_moves();
    moves.choose(rng).copied()
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

mod weight {
    pub(super) type Matrix = [f64; 16];

    const W0: Matrix = [
        0.135759, 0.121925, 0.102812, 0.099937, 0.0997992, 0.0888405, 0.076711, 0.0724143,
        0.060654, 0.0562579, 0.037116, 0.0161889, 0.0125498, 0.00992495, 0.00575871, 0.00335193,
    ];
    const W1: Matrix = rot_r(W0);
    const W2: Matrix = rot_r(W1);
    const W3: Matrix = rot_r(W2);
    pub(super) const W_MATRICES: [Matrix; 8] = [
        W0,
        W1,
        W2,
        W3,
        transpose(W0),
        transpose(W1),
        transpose(W2),
        transpose(W3),
    ];

    const RIGHT_ROTATE_IDX: [usize; 16] = [12, 8, 4, 0, 13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3];

    const fn rot_r(w: Matrix) -> Matrix {
        let mut new_w = [0f64; 16];
        let mut i = 0;
        while i < 16 {
            new_w[i] = w[RIGHT_ROTATE_IDX[i]];
            i += 1;
        }
        new_w
    }

    const fn transpose(w: Matrix) -> Matrix {
        let mut new_w = [0f64; 16];
        let mut i = 0;
        while i < 4 {
            let mut j = 0;
            while j < 4 {
                new_w[i * 4 + j] = w[j * 4 + i];
                j += 1;
            }
            i += 1;
        }
        new_w
    }
}

fn state_weight_product(s: &State, w: weight::Matrix) -> f64 {
    (0..16).map(|i| s.get(i) as f64 * w[i]).sum()
}

fn terminal_score(s: &State) -> f64 {
    weight::W_MATRICES
        .iter()
        .map(|&w_mat| state_weight_product(s, w_mat))
        .max_by(|x, y| {
            if x < y {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
        .unwrap()
}

fn expectimax_score(s: &State, search_depth: usize) -> f64 {
    if search_depth == 0 {
        return terminal_score(s);
    }
    todo!();
}

pub fn expectimax_move(s: &State, search_depth: usize) -> Option<(Move, State)> {
    let scored_moves = s
        .legal_moves()
        .into_iter()
        .map(|(m, s)| (m, s, expectimax_score(&s, search_depth)));
    scored_moves
        .max_by(|(_, _, score1), (_, _, score2)| {
            if score1 < score2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
        .map(|(m, s, _)| (m, s))
}
