//! AI to automatically play 2048.
//!
//! Uses the expectimax algorithm with a depth limit. The high-level algorithm
//! is nicely described in [these lecture notes by Anca Dragan and Sergey
//! Levine](https://inst.eecs.berkeley.edu/~cs188/fa19/assets/slides/archive/SP18-CS188%20Lecture%207%20--%20Expectimax%20Search%20and%20Utilities.pdf),
//! while the 2048-specific terminal state scoring is taken from [this blog
//! post](https://codemyroad.wordpress.com/2014/05/14/2048-ai-the-intelligent-bot/),
//! which did some sort of hyperparameter search to come up with a weight
//! matrix.
use std::cmp::Ordering;

use crate::game::{self, Move, State};
use rand::seq::SliceRandom;
use rand::Rng;

/// Generate a random legal move from `s`, and return the next state.
pub fn rand_move<R: Rng>(s: &State, rng: &mut R) -> Option<(Move, State)> {
    let moves = s.legal_moves();
    moves.choose(rng).copied()
}

mod weight {
    pub(super) type Matrix = [f64; 16];

    // This magical weight matrix is taken from
    // https://codemyroad.wordpress.com/2014/05/14/2048-ai-the-intelligent-bot/.
    //
    // It tries to encourage putting a big number in the top-left corner (then
    // all rotations/transposes of this matrix are tried to explore the
    // symmetries).
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
        // TODO: would be nice to initialize this in a better way (using a macro
        // probably), we have an initialization expression in terms of the
        // index...
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

    pub(super) fn dot(w1: Matrix, w2: Matrix) -> f64 {
        // NOTE: this is actually much more confusing than the imperative version
        w1.iter().zip(w2.iter()).map(|(x1, x2)| x1 * x2).sum()
    }
}

fn float_cmp(x: f64, y: f64) -> std::cmp::Ordering {
    if x < y {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

fn state_tiles(s: &State) -> weight::Matrix {
    let mut tiles = [0f64; 16];
    for (i, tile) in tiles.iter_mut().enumerate() {
        *tile = s.tile(i) as f64;
    }
    tiles
}

/// Score a terminal state using a weight matrix that encourages tiles to be in
/// one corner.
pub fn weight_score(s: &State) -> f64 {
    let tiles: [f64; 16] = state_tiles(s);
    weight::W_MATRICES
        .iter()
        .map(|&w_mat| weight::dot(tiles, w_mat))
        .max_by(|&x, &y| float_cmp(x, y))
        .unwrap()
}

/// Score a state just using the total value of all tiles, without regard to placement.
pub fn sum_tiles_score(s: &State) -> f64 {
    (0..16).map(|i| s.tile(i) as f64).sum()
}

fn expectimax_score(s: &State, search_depth: u32, terminal_score: &impl Fn(&State) -> f64) -> f64 {
    if search_depth == 0 {
        return terminal_score(s);
    }

    fn state_place(s: &State, i: u8, x: u8) -> State {
        let mut next_s = *s;
        next_s.add(i as usize, x);
        next_s
    }

    // we want to the expected value of the expectimax score over all the random
    // placements that could happen in this state
    let mut weighted_sum: f64 = 0.0;
    let poss = s.empty();
    let total_weight = poss.len() as f64;
    for i in poss.into_iter() {
        for (p, x) in [(game::TWO_SPAWN_PROB, 1), (game::FOUR_SPAWN_PROB, 2)] {
            let next_s = state_place(s, i, x);
            weighted_sum += p * expectimax_best(&next_s, search_depth - 1, terminal_score)
                .map(|(_, _, s)| s)
                .unwrap_or_else(|| terminal_score(s));
        }
    }
    #[allow(clippy::needless_return)]
    return weighted_sum / total_weight;
}

fn expectimax_best(
    s: &State,
    search_depth: u32,
    terminal_score: &impl Fn(&State) -> f64,
) -> Option<(Move, State, f64)> {
    let scored_moves = s
        .legal_moves()
        .into_iter()
        .map(|(m, s)| (m, s, expectimax_score(&s, search_depth, terminal_score)));
    scored_moves.max_by(|&(_, _, score1), &(_, _, score2)| float_cmp(score1, score2))
}

fn expectimax_move(
    s: &State,
    search_depth: u32,
    terminal_score: &impl Fn(&State) -> f64,
) -> Option<(Move, State)> {
    expectimax_best(s, search_depth, terminal_score).map(|(m, s, _)| (m, s))
}

pub fn smart_depth(s: &State) -> u32 {
    #[allow(clippy::let_and_return)]
    let depth = if s.empty().len() < 5 { 3 } else { 2 };
    depth
}

pub fn expectimax_weight_move(s: &State, search_depth: u32) -> Option<(Move, State)> {
    expectimax_move(s, search_depth, &weight_score)
}

pub fn expectimax_sum_move(s: &State, search_depth: u32) -> Option<(Move, State)> {
    expectimax_move(s, search_depth, &sum_tiles_score)
}
