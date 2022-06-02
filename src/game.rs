//! Represent 2048 game states and transitions.
//!
//! Unpacked representation, to be used as specification for more efficient
//! packed representation (where a state is a single u64 and each cell is 4
//! bits).

use std::fmt;

use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Row([u8; 4]);

impl Row {
    fn shift_left(&self) -> Self {
        let mut els: Vec<u8> = self
            .0
            .iter()
            .filter_map(|&x| if x > 0 { Some(x) } else { None })
            .collect();
        let mut i = 0;
        while i + 1 < els.len() {
            if els[i] == els[i + 1] {
                els[i] += 1;
                els.remove(i + 1);
            }
            i += 1;
        }
        let mut new_row = [0u8; 4];
        new_row[..els.len()].clone_from_slice(&els);
        Row(new_row)
    }

    fn reverse(&self) -> Self {
        let row = self.0;
        Row([row[3], row[2], row[1], row[0]])
    }

    fn shift_right(&self) -> Self {
        self.reverse().shift_left().reverse()
    }

    fn empty(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        for i in 0..4 {
            if self.0[i] == 0 {
                indices.push(i);
            }
        }
        indices
    }

    fn add(&mut self, i: usize, x: u8) {
        self.0[i] = x;
    }
}

#[cfg(test)]
mod row_tests {
    use super::Row;

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
    fn collapse() {
        assert_eq!(Row([2, 0, 0, 0]), Row([0, 1, 0, 1]).shift_left());
        assert_eq!(Row([4, 6, 0, 0]), Row([4, 5, 0, 5]).shift_left());
        assert_eq!(Row([2, 2, 0, 0]), Row([1, 1, 1, 1]).shift_left());
        // this one is subtle
        assert_eq!(Row([2, 1, 0, 0]), Row([1, 1, 1, 0]).shift_left());
        assert_eq!(Row([0, 0, 1, 2]), Row([0, 1, 1, 1]).shift_right());
        assert_eq!(Row([2, 3, 0, 0]), Row([1, 1, 2, 2]).shift_left());
    }

    #[test]
    fn empty() {
        assert_eq!(vec![0, 1, 2, 3], Row([0, 0, 0, 0]).empty());
        assert_eq!(vec![1, 2], Row([3, 0, 0, 2]).empty());
        assert_eq!(Vec::<usize>::new(), Row([1, 3, 2, 1]).empty());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State([Row; 4]);

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
            write!(
                f,
                "{:>2} {:>2} {:>2} {:>2}\n",
                row[0], row[1], row[2], row[3]
            )?;
        }
        Ok(())
    }
}

impl State {
    const FOUR_SPAWN_PROB: f64 = 0.1;

    // get by linear index
    fn get(&self, i: usize) -> u8 {
        self.0[i / 4].0[i % 4]
    }

    // set by linear index
    fn set(&mut self, i: usize, x: u8) {
        self.0[i / 4].0[i % 4] = x;
    }

    // rotate right
    //
    // internally used to implement up/down movement using only left/right
    fn rotate_right(&self) -> Self {
        let mut new = Self::default();
        // right rotation indices, computed by hand
        for (i, &idx) in [12, 8, 4, 0, 13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3]
            .iter()
            .enumerate()
        {
            new.set(i, self.get(idx));
        }
        new
    }

    // rotate left
    //
    // internally used to implement up/down movement using only left/right
    fn rotate_left(&self) -> Self {
        let mut new = Self::default();
        for (i, &idx) in [12, 8, 4, 0, 13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3]
            .iter()
            .enumerate()
        {
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

    pub fn game_over(&self) -> bool {
        self.legal_moves().is_empty()
    }

    pub fn make_move(&self, m: Move) -> Self {
        match m {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up => self.rotate_left().move_left().rotate_right(),
            Move::Down => self.rotate_right().move_left().rotate_left(),
        }
    }

    fn empty(&self) -> Vec<(usize, usize)> {
        let mut indices = Vec::new();
        self.0.iter().enumerate().for_each(|(i, &row)| {
            indices.extend(row.empty().iter().map(|&j| (i, j)));
        });
        indices
    }

    fn add(&mut self, i: usize, j: usize, x: u8) -> &mut Self {
        self.0[i].add(j, x);
        self
    }

    pub fn rand_add<R: Rng>(&mut self, rng: &mut R) -> &mut Self {
        // if nothing is empty this will fail (and we won't add anything)
        if let Some(&(i, j)) = self.empty().choose(rng) {
            let x = if rng.gen_bool(Self::FOUR_SPAWN_PROB) {
                2 // numbers are encoded by their power of 2
            } else {
                1
            };
            self.add(i, j, x);
        }
        self
    }

    pub fn score(&self) -> u8 {
        let mut best = 0;
        for i in 0..16 {
            let x = self.get(i);
            best = best.max(x);
        }
        best
    }
}

#[cfg(test)]
mod state_tests {

    use super::{Move, Row, State};

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

    #[test]
    fn empty() {
        assert_eq!(
            vec![(0, 1), (0, 2), (1, 0), (3, 3)],
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
