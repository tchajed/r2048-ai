//! Represent 2048 game states and transitions.
//!
//! Unpacked representation, to be used as specification for more efficient
//! packed representation (where a state is a single u64 and each cell is 4
//! bits).

// temporary, while still working
#![allow(dead_code)]

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State([Row; 4]);

impl State {
    fn get(&self, i: usize, j: usize) -> u8 {
        self.0[i].0[j]
    }

    fn transpose_in_place(&mut self) -> &mut Self {
        for i in 0..3 {
            for j in (i + 1)..4 {
                let tmp = self.get(i, j);
                self.0[i].0[j] = self.get(j, i);
                self.0[j].0[i] = tmp;
            }
        }
        self
    }

    fn transposed(&self) -> Self {
        let mut new = *self;
        new.transpose_in_place();
        new
    }
}

#[cfg(test)]
mod state_tests {

    use super::{Row, State};

    #[test]
    fn test_transpose() {
        assert_eq!(
            State([
                Row([0, 4, 8, 12]),
                Row([1, 5, 9, 13]),
                Row([2, 6, 10, 14]),
                Row([3, 7, 11, 15]),
            ]),
            State([
                Row([0, 1, 2, 3]),
                Row([4, 5, 6, 7]),
                Row([8, 9, 10, 11]),
                Row([12, 13, 14, 15]),
            ])
            .transposed()
        );
    }
}
