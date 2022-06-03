//! Represent 2048 game states and transitions.
//!
//! Unpacked representation, to be used as specification for more efficient
//! packed representation (where a state is a single u64 and each cell is 4
//! bits).

mod row;

use std::fmt;

use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

pub use row::{ArrayRow, CachedRow, Row};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State<R: Row>([R; 4]);

assert_eq_size!([u8; 16], State<ArrayRow>);
assert_eq_size!([u8; 8], State<CachedRow>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

impl Move {
    pub const ALL: [Move; 4] = [Move::Left, Move::Right, Move::Up, Move::Down];
}

impl<R: Row + fmt::Display> fmt::Display for State<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.0.iter() {
            writeln!(f, "{row}")?;
        }
        Ok(())
    }
}

pub const FOUR_SPAWN_PROB: f64 = 0.1;
pub const TWO_SPAWN_PROB: f64 = 1.0 - FOUR_SPAWN_PROB;

impl State<ArrayRow> {
    #[cfg(test)]
    fn new(els: [[u8; 4]; 4]) -> Self {
        State([
            ArrayRow::from_arr(els[0]),
            ArrayRow::from_arr(els[1]),
            ArrayRow::from_arr(els[2]),
            ArrayRow::from_arr(els[3]),
        ])
    }
}

impl<R: Row> State<R> {
    /// Get a cell by linear index (in 0..16).
    fn get(&self, i: usize) -> u8 {
        self.0[i / 4].get(i % 4)
    }

    /// Get a tile's value by linear index.
    ///
    /// This will be the power-of-two seen in the game.
    #[inline]
    pub fn tile(&self, i: usize) -> u32 {
        2u32.pow(self.get(i).into())
    }

    /// Add a tile by linear index.
    ///
    /// Should only be used when the tile is empty.
    pub fn add(&mut self, i: usize, x: u8) {
        self.0[i / 4].add(i % 4, x);
    }

    const RIGHT_ROTATE_IDX: [usize; 16] = [12, 8, 4, 0, 13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3];

    /// rotate right
    ///
    /// internally used to implement up/down movement using only left/right
    fn rotate_right(&self) -> Self {
        let mut new = Self::default();
        // right rotation indices, computed by hand
        for (i, &idx) in Self::RIGHT_ROTATE_IDX.iter().enumerate() {
            new.add(i, self.get(idx));
        }
        new
    }

    /// rotate left
    ///
    /// internally used to implement up/down movement using only left/right
    fn rotate_left(&self) -> Self {
        let mut new = Self::default();
        for (i, &idx) in Self::RIGHT_ROTATE_IDX.iter().enumerate() {
            new.add(idx, self.get(i));
        }
        new
    }

    fn move_left(&self) -> Self {
        let [r0, r1, r2, r3] = self.0;
        Self([
            r0.shift_left(),
            r1.shift_left(),
            r2.shift_left(),
            r3.shift_left(),
        ])
    }

    fn move_right(&self) -> Self {
        let [r0, r1, r2, r3] = self.0;
        Self([
            r0.shift_right(),
            r1.shift_right(),
            r2.shift_right(),
            r3.shift_right(),
        ])
    }

    fn make_move(&self, m: Move) -> Self {
        match m {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up => self.rotate_left().move_left().rotate_right(),
            Move::Down => self.rotate_right().move_left().rotate_left(),
        }
    }

    /// Generate legal moves and immediate next states.
    ///
    /// Only moves that change the state are legal.
    pub fn legal_moves(&self) -> Vec<(Move, Self)> {
        Move::ALL
            .iter()
            .filter_map(|&m| {
                let s = self.make_move(m);
                if s != *self {
                    Some((m, s))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the linear indices of empty positions.
    pub fn empty(&self) -> Vec<u8> {
        let mut indices = Vec::new();
        self.0.iter().enumerate().for_each(|(i, &row)| {
            let i = i as u8;
            indices.extend(row.empty().iter().map(|&j| i * 4 + j));
        });
        indices
    }

    /// Add a random tile to the board.
    pub fn rand_add<Rn: Rng>(&mut self, rng: &mut Rn) -> &mut Self {
        if let Some(&i) = self.empty().choose(rng) {
            let x = if rng.gen_bool(TWO_SPAWN_PROB) {
                1 // numbers are encoded by their power of 2
            } else {
                2
            };
            self.add(i as usize, x);
        } else {
            // no move should leave the board this full
            panic!("attempt to add to a full board");
        }
        self
    }

    /// Return the highest tile, converted to the usual power of two.
    pub fn highest_tile(&self) -> u32 {
        let exp = (0..16).map(|i| self.get(i)).max().unwrap();
        (2_u32).pow(exp.into())
    }
}

#[cfg(test)]
mod tests {

    use super::{ArrayRow, Move, State};
    use quickcheck::{quickcheck, Arbitrary, Gen};

    impl Arbitrary for State<ArrayRow> {
        fn arbitrary(g: &mut Gen) -> Self {
            dbg!(State([
                ArrayRow::arbitrary(g),
                ArrayRow::arbitrary(g),
                ArrayRow::arbitrary(g),
                ArrayRow::arbitrary(g),
            ]))
        }
    }

    #[test]
    fn rotate() {
        let s = State::new([[0, 1, 2, 3], [4, 0, 0, 0], [8, 0, 0, 0], [12, 0, 0, 0]]);
        assert_eq!(
            State::new([[12, 8, 4, 0], [0, 0, 0, 1], [0, 0, 0, 2], [0, 0, 0, 3],]),
            s.rotate_right()
        );
        assert_eq!(
            State::new([[3, 0, 0, 0], [2, 0, 0, 0], [1, 0, 0, 0], [0, 4, 8, 12],]),
            s.rotate_left()
        )
    }

    // test is broken due to a panic in quickcheck
    #[test]
    #[ignore]
    fn prop_rotate_left3_is_right() {
        fn prop(s: State<ArrayRow>) -> bool {
            s.rotate_left().rotate_left().rotate_left() == s
        }
        quickcheck(prop as fn(State<ArrayRow>) -> bool);
    }

    fn index(i: usize, j: usize) -> u8 {
        (i * 4 + j) as u8
    }

    #[test]
    fn empty() {
        assert_eq!(
            // these are linear indices, but compute them here for readability
            vec![index(0, 1), index(0, 2), index(1, 0), index(3, 3)],
            State::new([[1, 0, 0, 2], [0, 2, 1, 3], [3, 4, 2, 5], [1, 2, 1, 0],]).empty()
        )
    }

    #[test]
    fn test_moves() {
        let s = State::new([[0, 0, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        assert_eq!(
            State::new([[0, 0, 0, 0], [1, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0],]),
            s.make_move(Move::Left),
        );
        assert_eq!(
            State::new([[0, 0, 0, 0], [0, 0, 0, 1], [0, 0, 0, 0], [0, 0, 0, 0],]),
            s.make_move(Move::Right),
        );
        assert_eq!(
            State::new([[0, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0],]),
            s.make_move(Move::Up),
        );
        assert_eq!(
            State::new([[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 1, 0, 0],]),
            s.make_move(Move::Down),
        );
    }

    #[test]
    fn printing() {
        assert_eq!(
            " 0  1  2  3
 4  5  6  7
 8  9 10 11
12 13 14 15
",
            format!(
                "{}",
                State::new([[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15],])
            )
        )
    }
}

type GameState = State<ArrayRow>;

pub struct Game<R: Rng> {
    rng: R,
    s: GameState,
    moves: u32,
}

impl Game<ThreadRng> {
    pub fn new() -> Self {
        Self::from_rng(ThreadRng::default())
    }
}

impl<R: Rng> Game<R> {
    pub fn from_rng(rng: R) -> Self {
        let mut rng = rng;
        let mut s = State::default();
        // game starts with two tiles
        s.rand_add(&mut rng);
        s.rand_add(&mut rng);
        Self { rng, s, moves: 0 }
    }

    pub fn state(&self) -> &GameState {
        &self.s
    }

    pub fn next_state(&mut self, s: GameState) {
        self.s = s;
        self.s.rand_add(&mut self.rng);
        self.moves += 1;
    }

    /// Get the number of moves made so far.
    pub fn moves(&self) -> u32 {
        self.moves
    }
}
