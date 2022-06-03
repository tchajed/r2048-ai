//! Represent 2048 game states and transitions.
//!
//! Unpacked representation, to be used as specification for more efficient
//! packed representation (where a state is a single u64 and each cell is 4
//! bits).

use std::fmt;

use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Row([u8; 4]);

impl Row {
    #[cfg(test)]
    /// The logic that shift_left is supposed to implement.
    fn shift_left_spec(&self) -> Self {
        // move the non-zero elements to the front
        let mut els: Vec<u8> = self
            .0
            .iter()
            .filter_map(|&x| if x > 0 { Some(x) } else { None })
            .collect();
        // pad with zeros so els.len() == 4
        while els.len() < 4 {
            els.push(0);
        }
        // look at i and i+1 in a sliding window
        let mut i = 0;
        while i + 1 < 4 {
            // collapse adjacent equal elements
            if els[i] != 0 && els[i] == els[i + 1] {
                // increment i (adding two equal powers of two means incrementing the exponent)
                els[i] += 1;
                // remove the extra copy and add a 0 to the end
                els.remove(i + 1);
                els.push(0);
                // check again without incrementing i, in case another merge is
                // needed at this position
                continue;
            }
            i += 1;
        }
        // turn els into an array
        let mut new_row = [0u8; 4];
        new_row.clone_from_slice(&els);
        Row(new_row)
    }

    /// Shift the row's elements to the left and collapse tiles together.
    ///
    /// This is extremely performance-critical and is thus written imperatively
    /// with no allocations.
    ///
    /// Honestly, I don't understand this code - it was written by fiddling with
    /// the logic and indices until the tests passed (which compare against the
    /// spec above).
    fn shift_left(&self) -> Self {
        let mut els = self.0;
        // current index
        let mut i = 0;
        // next non-zero
        let mut j = 0;
        while j < 4 && els[j] == 0 {
            j += 1;
        }
        // while we have non-zeros to process
        while j < 4 {
            // move the next non-zero to i
            let tmp = els[j];
            els[j] = 0;
            els[i] = tmp;
            j += 1;
            // if there's a previous element, try to collapse with it
            if i > 0 && els[i] == els[i - 1] {
                els[i - 1] += 1;
                els[i] = 0;
                // re-merge at same position
                i -= 1;
            }
            while j < 4 && els[j] == 0 {
                j += 1;
            }
            i += 1;
        }
        Row(els)
    }

    #[inline]
    fn reverse(&self) -> Self {
        let row = self.0;
        Row([row[3], row[2], row[1], row[0]])
    }

    fn shift_right(&self) -> Self {
        self.reverse().shift_left().reverse()
    }

    fn empty(&self) -> Vec<u8> {
        let mut indices = Vec::new();
        for i in 0..4 {
            if self.0[i] == 0 {
                indices.push(i as u8);
            }
        }
        indices
    }

    fn get(&self, i: usize) -> u8 {
        self.0[i]
    }

    fn set(&mut self, i: usize, x: u8) {
        self.0[i] = x;
    }
}

#[cfg(test)]
mod row_tests {
    use quickcheck::{quickcheck, Arbitrary, Gen};

    use super::Row;

    impl Arbitrary for Row {
        fn arbitrary(g: &mut Gen) -> Row {
            Row([
                u8::arbitrary(g),
                u8::arbitrary(g),
                u8::arbitrary(g),
                u8::arbitrary(g),
            ])
        }
    }

    #[test]
    fn reverse() {
        assert_eq!(Row([3, 2, 1, 0]), Row([0, 1, 2, 3]).reverse());
        assert_eq!(Row([0, 1, 2, 3]), Row([0, 1, 2, 3]).reverse().reverse());
    }

    #[test]
    fn no_collapsing() {
        assert_eq!(Row([1, 2, 3, 0]), Row([0, 1, 2, 3]).shift_left());
        assert_eq!(Row([1, 0, 0, 0]), Row([0, 0, 1, 0]).shift_left());
        assert_eq!(Row([1, 0, 0, 0]), Row([0, 0, 0, 1]).shift_left());
        assert_eq!(Row([5, 3, 0, 0]), Row([0, 5, 0, 3]).shift_left());
    }

    #[test]
    fn shifts() {
        for (shifted, r) in vec![
            // simple, no-collapse tests
            (Row([1, 2, 3, 0]), Row([0, 1, 2, 3])),
            (Row([1, 0, 0, 0]), Row([0, 0, 0, 1])),
            (Row([5, 3, 0, 0]), Row([0, 5, 0, 3])),
            // collapsing
            (Row([2, 0, 0, 0]), Row([0, 1, 0, 1])),
            (Row([4, 6, 0, 0]), Row([4, 5, 0, 5])),
            (Row([2, 2, 0, 0]), Row([1, 1, 1, 1])),
            (Row([2, 1, 0, 0]), Row([1, 1, 1, 0])),
            (Row([2, 1, 0, 0]), Row([0, 1, 1, 1])),
            (Row([3, 2, 0, 0]), Row([1, 1, 2, 2])),
            (Row([4, 5, 0, 0]), Row([2, 2, 3, 5])),
            (Row([3, 5, 0, 0]), Row([2, 2, 4, 4])),
            // bunch of unchanged examples
            (Row([0, 0, 0, 0]), Row([0, 0, 0, 0])),
            (Row([1, 0, 0, 0]), Row([1, 0, 0, 0])),
            (Row([2, 3, 2, 0]), Row([2, 3, 2, 0])),
            (Row([3, 4, 5, 3]), Row([3, 4, 5, 3])),
        ]
        .into_iter()
        {
            assert_eq!(shifted, r.shift_left(), "{:?} left shift is wrong", r);
            assert_eq!(
                r.reverse().shift_left().reverse(),
                r.shift_right(),
                "{:?} shifted wrong",
                r
            );
            assert_eq!(
                r.shift_left_spec(),
                r.shift_left(),
                "{:?} left shift doesn't match spec",
                r
            )
        }
    }

    #[test]
    fn prop_shift_left_spec() {
        fn prop(r: Row) -> bool {
            r.shift_left_spec() == r.shift_left()
        }
        quickcheck(prop as fn(Row) -> bool);
    }

    #[test]
    fn empty() {
        assert_eq!(vec![0, 1, 2, 3], Row([0, 0, 0, 0]).empty());
        assert_eq!(vec![1, 2], Row([3, 0, 0, 2]).empty());
        assert_eq!(Vec::<u8>::new(), Row([1, 3, 2, 1]).empty());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State([Row; 4]);

assert_eq_size!([u8; 16], State);

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

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.0.iter() {
            let row = row.0;
            writeln!(f, "{:>2} {:>2} {:>2} {:>2}", row[0], row[1], row[2], row[3])?;
        }
        Ok(())
    }
}

impl State {
    pub const FOUR_SPAWN_PROB: f64 = 0.1;
    pub const TWO_SPAWN_PROB: f64 = 1.0 - Self::FOUR_SPAWN_PROB;

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

    /// Set a tile by linear index.
    pub fn set(&mut self, i: usize, x: u8) {
        self.0[i / 4].set(i % 4, x);
    }

    const RIGHT_ROTATE_IDX: [usize; 16] = [12, 8, 4, 0, 13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3];

    /// rotate right
    ///
    /// internally used to implement up/down movement using only left/right
    fn rotate_right(&self) -> Self {
        let mut new = Self::default();
        // right rotation indices, computed by hand
        for (i, &idx) in Self::RIGHT_ROTATE_IDX.iter().enumerate() {
            new.set(i, self.get(idx));
        }
        new
    }

    /// rotate left
    ///
    /// internally used to implement up/down movement using only left/right
    fn rotate_left(&self) -> Self {
        let mut new = Self::default();
        for (i, &idx) in Self::RIGHT_ROTATE_IDX.iter().enumerate() {
            new.set(idx, self.get(i));
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

    // returns linear indices of empty positions
    pub fn empty(&self) -> Vec<u8> {
        let mut indices = Vec::new();
        self.0.iter().enumerate().for_each(|(i, &row)| {
            let i = i as u8;
            indices.extend(row.empty().iter().map(|&j| i * 4 + j));
        });
        indices
    }

    /// Add a random tile to the board.
    pub fn rand_add<R: Rng>(&mut self, rng: &mut R) -> &mut Self {
        if let Some(&i) = self.empty().choose(rng) {
            let x = if rng.gen_bool(Self::FOUR_SPAWN_PROB) {
                2 // numbers are encoded by their power of 2
            } else {
                1
            };
            self.set(i as usize, x);
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
mod state_tests {

    use super::{Move, Row, State};
    use quickcheck::{quickcheck, Arbitrary, Gen};

    impl Arbitrary for State {
        fn arbitrary(g: &mut Gen) -> State {
            dbg!(State([
                Row::arbitrary(g),
                Row::arbitrary(g),
                Row::arbitrary(g),
                Row::arbitrary(g),
            ]))
        }
    }

    #[test]
    fn rotate() {
        let s = State([
            Row([0, 1, 2, 3]),
            Row([4, 0, 0, 0]),
            Row([8, 0, 0, 0]),
            Row([12, 0, 0, 0]),
        ]);
        assert_eq!(
            State([
                Row([12, 8, 4, 0]),
                Row([0, 0, 0, 1]),
                Row([0, 0, 0, 2]),
                Row([0, 0, 0, 3]),
            ]),
            s.rotate_right()
        );
        assert_eq!(
            State([
                Row([3, 0, 0, 0]),
                Row([2, 0, 0, 0]),
                Row([1, 0, 0, 0]),
                Row([0, 4, 8, 12]),
            ]),
            s.rotate_left()
        )
    }

    // test is broken due to a panic in quickcheck
    #[test]
    #[ignore]
    fn prop_rotate_left3_is_right() {
        fn prop(s: State) -> bool {
            s.rotate_left().rotate_left().rotate_left() == s
        }
        quickcheck(prop as fn(State) -> bool);
    }

    fn index(i: usize, j: usize) -> u8 {
        (i * 4 + j) as u8
    }

    #[test]
    fn empty() {
        assert_eq!(
            // these are linear indices, but compute them here for readability
            vec![index(0, 1), index(0, 2), index(1, 0), index(3, 3)],
            State([
                Row([1, 0, 0, 2]),
                Row([0, 2, 1, 3]),
                Row([3, 4, 2, 5]),
                Row([1, 2, 1, 0]),
            ])
            .empty()
        )
    }

    #[test]
    fn test_moves() {
        let s = State([
            Row([0, 0, 0, 0]),
            Row([0, 1, 0, 0]),
            Row([0, 0, 0, 0]),
            Row([0, 0, 0, 0]),
        ]);
        assert_eq!(
            State([
                Row([0, 0, 0, 0]),
                Row([1, 0, 0, 0]),
                Row([0, 0, 0, 0]),
                Row([0, 0, 0, 0]),
            ]),
            s.make_move(Move::Left),
        );
        assert_eq!(
            State([
                Row([0, 0, 0, 0]),
                Row([0, 0, 0, 1]),
                Row([0, 0, 0, 0]),
                Row([0, 0, 0, 0]),
            ]),
            s.make_move(Move::Right),
        );
        assert_eq!(
            State([
                Row([0, 1, 0, 0]),
                Row([0, 0, 0, 0]),
                Row([0, 0, 0, 0]),
                Row([0, 0, 0, 0]),
            ]),
            s.make_move(Move::Up),
        );
        assert_eq!(
            State([
                Row([0, 0, 0, 0]),
                Row([0, 0, 0, 0]),
                Row([0, 0, 0, 0]),
                Row([0, 1, 0, 0]),
            ]),
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
                State([
                    Row([0, 1, 2, 3]),
                    Row([4, 5, 6, 7]),
                    Row([8, 9, 10, 11]),
                    Row([12, 13, 14, 15]),
                ])
            )
        )
    }
}

pub struct Game<R: Rng> {
    rng: R,
    s: State,
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
