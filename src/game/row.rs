use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Row([u8; 4]);

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let row = self.0;
        write!(f, "{:>2} {:>2} {:>2} {:>2}", row[0], row[1], row[2], row[3])?;
        Ok(())
    }
}

impl Row {
    #[cfg(test)]
    pub fn from_arr(xs: [u8; 4]) -> Row {
        Self(xs)
    }

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
    pub fn shift_left(&self) -> Self {
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

    pub fn shift_right(&self) -> Self {
        self.reverse().shift_left().reverse()
    }

    pub fn empty(&self) -> Vec<u8> {
        let mut indices = Vec::new();
        for i in 0..4 {
            if self.0[i] == 0 {
                indices.push(i as u8);
            }
        }
        indices
    }

    pub fn get(&self, i: usize) -> u8 {
        self.0[i]
    }

    /// Add a tile
    ///
    /// Should only be used to add tiles to empty cells.
    pub fn add(&mut self, i: usize, x: u8) {
        assert_eq!(0, self.0[i]);
        self.0[i] = x;
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{quickcheck, Arbitrary, Gen};

    use super::Row;

    impl Arbitrary for Row {
        fn arbitrary(g: &mut Gen) -> Row {
            Row([
                u8::arbitrary(g) / 2,
                u8::arbitrary(g) / 2,
                u8::arbitrary(g) / 2,
                u8::arbitrary(g) / 2,
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
