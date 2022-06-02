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
    (0..16).map(|i| s.tile(i) as f64 * w[i]).sum()
}

fn float_cmp(x: f64, y: f64) -> std::cmp::Ordering {
    if x < y {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// Score a terminal state using a weight matrix that encourages tiles to be in
/// one corner.
pub fn weight_score(s: &State) -> f64 {
    weight::W_MATRICES
        .iter()
        .map(|&w_mat| state_weight_product(s, w_mat))
        .max_by(|&x, &y| float_cmp(x, y))
        .unwrap()
}

/// Score a state just using the total value of all tiles, without regard to placement.
pub fn sum_tiles_score(s: &State) -> f64 {
    // weight everything equally
    state_weight_product(s, [1f64; 16])
}

fn state_place(s: &State, i: u8, x: u8) -> State {
    let mut next_s = *s;
    next_s.set(i as usize, x);
    next_s
}

fn expectimax_score<ScoreF>(s: &State, search_depth: usize, terminal_score: ScoreF) -> f64
where
    ScoreF: Fn(&State) -> f64,
{
    if search_depth == 0 {
        return terminal_score(s);
    }
    // we want to the expected value of the expectimax score over all the random
    // placements that could happen in this state
    let mut weighted_sum: f64 = 0.0;
    let poss = s.empty();
    let total_weight = poss.len() as f64;
    for i in poss.into_iter() {
        for (p, x) in [(State::TWO_SPAWN_PROB, 1), (State::FOUR_SPAWN_PROB, 2)] {
            let next_s = state_place(s, i, x);
            weighted_sum += p * expectimax_best(&next_s, search_depth - 1, &terminal_score)
                .map(|(_, _, s)| s)
                .unwrap_or_else(|| terminal_score(s));
        }
    }
    return weighted_sum / total_weight;
}

fn expectimax_best<ScoreF>(
    s: &State,
    search_depth: usize,
    terminal_score: ScoreF,
) -> Option<(Move, State, f64)>
where
    ScoreF: Fn(&State) -> f64,
{
    let scored_moves = s
        .legal_moves()
        .into_iter()
        .map(|(m, s)| (m, s, expectimax_score(&s, search_depth, &terminal_score)));
    scored_moves.max_by(|&(_, _, score1), &(_, _, score2)| float_cmp(score1, score2))
}

pub fn expectimax_move<ScoreF>(
    s: &State,
    search_depth: usize,
    terminal_score: ScoreF,
) -> Option<(Move, State)>
where
    ScoreF: Fn(&State) -> f64,
{
    expectimax_best(s, search_depth, terminal_score).map(|(m, s, _)| (m, s))
}
