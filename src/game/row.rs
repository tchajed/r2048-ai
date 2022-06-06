//! Representation of single rows and implementation of the core shifting
//! functionality.
//!
//! This module [`ArrayRow`] type uses an array of `u8`'s, which represent the
//! log2 of each tile. It is only used to bootstrap [`CachedRow`], which is
//! further compacted to a single u16 with four bits per cell (note that this
//! limits us to tiles worth 2^15 = 32,768) and then the left and right shift
//! operations are fully pre-computed for all 2^16 possible rows.
use std::fmt;

pub trait Row: Copy + Clone + PartialEq + Eq + Default {
    /// Shift the row's elements to the left and collapse tiles together.
    fn shift_left(&self) -> Self;

    /// Shift the row's elements to the right and collapse tiles together.
    fn shift_right(&self) -> Self;

    /// Return the indices (in 0..4) of the empty positions in this row.
    fn empty(&self) -> Vec<u8>;

    /// Get a value by index.
    fn get(&self, i: usize) -> u8;

    /// Add a tile
    ///
    /// Should only be used to add tiles to empty cells.
    fn add(&mut self, i: usize, x: u8);
}

/// ArrowRow implements rows with a fixed-size array of bytes and a fairly
/// efficient shift algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ArrayRow([u8; 4]);

impl fmt::Display for ArrayRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let row = self.0;
        write!(f, "{:>2} {:>2} {:>2} {:>2}", row[0], row[1], row[2], row[3])?;
        Ok(())
    }
}

impl ArrayRow {
    #[inline]
    // helper for implementing shift_right in terms of shift_left
    fn reverse(&self) -> Self {
        let row = self.0;
        ArrayRow([row[3], row[2], row[1], row[0]])
    }
}

impl Row for ArrayRow {
    fn shift_left(&self) -> Self {
        // This is extremely performance-critical and is thus written imperatively
        // with no allocations.
        //
        // Honestly, I don't understand this code - it was written by fiddling with
        // the logic and indices until the tests passed (which compare against the
        // spec above).
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
        ArrayRow(els)
    }

    fn shift_right(&self) -> Self {
        self.reverse().shift_left().reverse()
    }

    #[inline]
    fn empty(&self) -> Vec<u8> {
        let mut indices = Vec::new();
        for i in 0..4 {
            if self.0[i] == 0 {
                indices.push(i as u8);
            }
        }
        indices
    }

    #[inline]
    fn get(&self, i: usize) -> u8 {
        self.0[i]
    }

    fn add(&mut self, i: usize, x: u8) {
        debug_assert_eq!(0, self.0[i]);
        self.0[i] = x;
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{quickcheck, Arbitrary, Gen};

    use super::{ArrayRow, Row};

    impl Arbitrary for ArrayRow {
        fn arbitrary(g: &mut Gen) -> ArrayRow {
            ArrayRow([
                u8::arbitrary(g) & 0xf,
                u8::arbitrary(g) & 0xf,
                u8::arbitrary(g) & 0xf,
                u8::arbitrary(g) & 0xf,
            ])
        }
    }

    impl ArrayRow {
        pub fn from_arr(xs: [u8; 4]) -> ArrayRow {
            Self(xs)
        }

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
            ArrayRow(new_row)
        }
    }

    #[test]
    fn reverse() {
        assert_eq!(ArrayRow([3, 2, 1, 0]), ArrayRow([0, 1, 2, 3]).reverse());
        assert_eq!(
            ArrayRow([0, 1, 2, 3]),
            ArrayRow([0, 1, 2, 3]).reverse().reverse()
        );
    }

    #[test]
    fn no_collapsing() {
        assert_eq!(ArrayRow([1, 2, 3, 0]), ArrayRow([0, 1, 2, 3]).shift_left());
        assert_eq!(ArrayRow([1, 0, 0, 0]), ArrayRow([0, 0, 1, 0]).shift_left());
        assert_eq!(ArrayRow([1, 0, 0, 0]), ArrayRow([0, 0, 0, 1]).shift_left());
        assert_eq!(ArrayRow([5, 3, 0, 0]), ArrayRow([0, 5, 0, 3]).shift_left());
    }

    #[test]
    fn shifts() {
        for (shifted, r) in vec![
            // simple, no-collapse tests
            (ArrayRow([1, 2, 3, 0]), ArrayRow([0, 1, 2, 3])),
            (ArrayRow([1, 0, 0, 0]), ArrayRow([0, 0, 0, 1])),
            (ArrayRow([5, 3, 0, 0]), ArrayRow([0, 5, 0, 3])),
            // collapsing
            (ArrayRow([2, 0, 0, 0]), ArrayRow([0, 1, 0, 1])),
            (ArrayRow([4, 6, 0, 0]), ArrayRow([4, 5, 0, 5])),
            (ArrayRow([2, 2, 0, 0]), ArrayRow([1, 1, 1, 1])),
            (ArrayRow([2, 1, 0, 0]), ArrayRow([1, 1, 1, 0])),
            (ArrayRow([2, 1, 0, 0]), ArrayRow([0, 1, 1, 1])),
            (ArrayRow([3, 2, 0, 0]), ArrayRow([1, 1, 2, 2])),
            (ArrayRow([4, 5, 0, 0]), ArrayRow([2, 2, 3, 5])),
            (ArrayRow([3, 5, 0, 0]), ArrayRow([2, 2, 4, 4])),
            // bunch of unchanged examples
            (ArrayRow([0, 0, 0, 0]), ArrayRow([0, 0, 0, 0])),
            (ArrayRow([1, 0, 0, 0]), ArrayRow([1, 0, 0, 0])),
            (ArrayRow([2, 3, 2, 0]), ArrayRow([2, 3, 2, 0])),
            (ArrayRow([3, 4, 5, 3]), ArrayRow([3, 4, 5, 3])),
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
        fn prop(r: ArrayRow) -> bool {
            r.shift_left_spec() == r.shift_left()
        }
        quickcheck(prop as fn(ArrayRow) -> bool);
    }

    #[test]
    fn empty() {
        assert_eq!(vec![0, 1, 2, 3], ArrayRow([0, 0, 0, 0]).empty());
        assert_eq!(vec![1, 2], ArrayRow([3, 0, 0, 2]).empty());
        assert_eq!(Vec::<u8>::new(), ArrayRow([1, 3, 2, 1]).empty());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CachedRow {
    num: u16,
}

// document (in an unchecked way) when values are supposed to be half-bytes
type U4 = u8;

impl CachedRow {
    #[inline]
    fn geti(&self, i: usize) -> U4 {
        ((self.num >> (i * 4)) & 0xf) as U4
    }

    fn to_array(self) -> ArrayRow {
        ArrayRow([self.geti(0), self.geti(1), self.geti(2), self.geti(3)])
    }

    #[allow(clippy::identity_op)]
    fn from_array(r: ArrayRow) -> Self {
        let r = r.0.map(|n| (n & 0xf) as u16);
        let num = (r[3] << 12) | (r[2] << 8) | (r[1] << 4) | (r[0] << 0);
        Self { num }
    }

    #[cfg(test)]
    pub fn from_arr(xs: [u8; 4]) -> Self {
        Self::from_array(ArrayRow::from_arr(xs))
    }
}

impl fmt::Display for CachedRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // defer to the ArrayRow implementation
        self.to_array().fmt(f)
    }
}

type CacheTable = Box<[CachedRow; 65536]>;

struct CachedRowTable {
    shift_left: CacheTable,
    shift_right: CacheTable,
}

lazy_static! {
    static ref CACHED_ROWS: CachedRowTable = CachedRowTable::new();
}

impl CachedRowTable {
    fn vec_to_table(v: Vec<CachedRow>) -> CacheTable {
        assert_eq!(65536, v.len(), "vector is not of cache length");
        v.into_boxed_slice().try_into().unwrap()
    }
    fn new() -> Self {
        let mut shift_left = vec![CachedRow::default(); 65536];
        let mut shift_right = vec![CachedRow::default(); 65536];
        for i in 0..65536 {
            let r = CachedRow { num: i as u16 }.to_array();
            shift_left[i] = CachedRow::from_array(r.shift_left());
            shift_right[i] = CachedRow::from_array(r.shift_right());
        }
        let shift_left = Self::vec_to_table(shift_left);
        let shift_right = Self::vec_to_table(shift_right);
        Self {
            shift_left,
            shift_right,
        }
    }
}

impl Row for CachedRow {
    fn shift_left(&self) -> Self {
        CACHED_ROWS.shift_left[self.num as usize]
    }

    fn shift_right(&self) -> Self {
        CACHED_ROWS.shift_right[self.num as usize]
    }

    #[inline]
    fn empty(&self) -> Vec<u8> {
        let mut idxs = Vec::new();
        for i in 0..4 {
            if self.geti(i) == 0 {
                idxs.push(i as u8);
            }
        }
        idxs
    }

    fn get(&self, i: usize) -> u8 {
        self.geti(i)
    }

    fn add(&mut self, i: usize, x: u8) {
        debug_assert!(x < 16, "{} will not fit in a CachedRow", x);
        debug_assert!(i < 4, "add is out-of-bounds {}", i);
        self.num |= (x as u16) << (4 * i);
    }
}

#[cfg(test)]
mod cached_tests {
    use super::{ArrayRow, CachedRow, Row};
    use quickcheck::{quickcheck, Arbitrary, Gen};

    impl Arbitrary for CachedRow {
        fn arbitrary(g: &mut Gen) -> Self {
            CachedRow {
                num: u16::arbitrary(g),
            }
        }
    }

    #[test]
    fn prop_to_from_cached() {
        fn prop(r: ArrayRow) -> bool {
            CachedRow::from_array(r).to_array() == r
        }
        quickcheck(prop as fn(ArrayRow) -> bool);
    }

    #[test]
    fn test_all_roundtrip() {
        for i in 0..65526 {
            let r = CachedRow { num: i as u16 };
            assert_eq!(
                r,
                CachedRow::from_array(r.to_array()),
                "row {:?} handled incorrectly",
                r.to_array()
            );
        }
    }

    #[test]
    fn test_add_get() {
        let mut r = CachedRow::default();
        r.add(2, 5);
        r.add(3, 8);
        assert_eq!(0, r.get(0));
        assert_eq!(0, r.get(1));
        assert_eq!(5, r.get(2));
        assert_eq!(8, r.get(3));
    }

    #[test]
    fn compare_to_array_row() {
        let rs: Vec<ArrayRow> = vec![
            [3, 4, 10, 0],
            [4, 3, 0, 0],
            [0, 0, 1, 0],
            [3, 3, 4, 5],
            [5, 2, 3, 3],
        ]
        .iter()
        .map(|&r| ArrayRow(r))
        .collect();
        for &row in rs.iter() {
            let r = CachedRow::from_array(row);
            assert_eq!(row, r.to_array(), "row does not even roundtrip");
        }
        for &row in rs.iter() {
            let row = row.shift_left();
            let r = CachedRow::from_array(row);
            assert_eq!(row, r.to_array(), "row does not even roundtrip");
        }
        for &row in rs.iter() {
            let r = CachedRow::from_array(row);
            assert_eq!(
                row.shift_left(),
                r.shift_left().to_array(),
                "shift left is wrong for {:?}",
                row
            );
            assert_eq!(
                row.shift_right(),
                r.shift_right().to_array(),
                "shift right is wrong for {:?}",
                row
            );
            assert_eq!(row.empty(), r.empty(), "empty is wrong for {}", row);
        }
    }
}
