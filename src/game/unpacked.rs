//! Represent 2048 game states and transitions.
//!
//! Unpacked representation, to be used as specification for more efficient
//! packed representation (where a state is a single u64 and each cell is 4
//! bits).

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
        new_row[4 - els.len()..4].clone_from_slice(&els);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State([Row; 4]);

impl State {
    fn get(&self, i: usize, j: usize) -> u8 {
        self.0[i].0[j]
    }

    fn transpose(&mut self) -> &mut Self {
        for i in 0..3 {
            for j in (i + 1)..4 {
                let tmp = self.get(i, j);
                self.0[i].0[j] = self.get(j, i);
                self.0[j].0[i] = tmp;
            }
        }
        self
    }
}
