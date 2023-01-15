use crate::{transitive, Action, Map, Parser, Position, Set, State, Tabler};
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Clr<T>
where
    T: PartialEq + PartialOrd + Ord + Clone + Display + Debug,
{
    pub table: Tabler<T>,
}

impl<T> Parser<T> for Clr<T>
where
    T: PartialEq + Ord + Clone + Display + Debug,
{
    fn with_table(table: Tabler<T>) -> Self {
        let mut parser = Self::uninit(table);
        parser.proc_actions();
        parser
    }

    fn uninit(table: Tabler<T>) -> Self {
        Self { table }
    }

    #[must_use]
    fn tables(&self) -> &Tabler<T> {
        &self.table
    }

    #[must_use]
    fn tables_mut(&mut self) -> &mut Tabler<T> {
        &mut self.table
    }
}

impl<T> Clr<T>
where
    T: PartialEq + Ord + Clone + Display + Debug,
{
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.table.states.push(start);
    }

    /// # Panics
    /// Never.
    pub fn proc_actions(&mut self) {
        self.proc_closures();
        let start = self.table.basis_pos().rule;
        for row in &self.table.states {
            let mut map: Map<T, Action<T>> = Map::new();
            for item in row {
                for (term, act) in self.decision(start, item, row) {
                    if map.contains_key(&term) && map[&term] != act {
                        *map.get_mut(&term).unwrap() =
                            Action::Conflict(Box::new(map.get(&term).unwrap().clone()), act.into());
                    } else {
                        map.insert(term, act);
                    }
                }
            }
            self.table.actions.push(map);
        }
    }

    #[must_use]
    pub fn closure(&self, state: State<T>) -> State<T> {
        let mut new_state = State::new();
        for pos in &state {
            if let Some(top) = pos.top() {
                if self.table.grammar.is_terminal(&top) {
                    continue;
                }
                let look = pos.locus().map_or_else(
                    || pos.look.clone(),
                    |locus| self.table.first_of(&Set::from([locus])),
                );
                for prod in self.table.grammar.rules[&top].prods() {
                    new_state.insert(Position::new(top, prod.clone(), 0, look.clone()));
                }
            }
        }
        new_state.extend(state);
        new_state
    }

    pub fn proc_closures(&mut self) {
        self.proc_closures_first_row();
        let mut idx = 0;
        while idx < self.table.states.len() {
            let row = self.table.states[idx].clone();
            for s in self.table.grammar.symbols() {
                let Some((kernel, closures)) = self.goto(&row, &s) else {
                    continue;
                };
                let old_val = self.table.kernels.insert(kernel, self.table.states.len());
                debug_assert!(old_val.is_none());
                self.table.states.push(closures);
            }
            idx += 1;
        }
    }

    #[must_use]
    pub fn goto(&self, kernels: &State<T>, sym: &T) -> Option<(State<T>, State<T>)> {
        let kernels = Tabler::sym_filter(kernels, sym);
        if self.table.kernels.contains_key(&kernels) {
            None?;
        }
        let new = self.prop_closure(kernels.clone());
        if new.is_empty() {
            None
        } else {
            Some((kernels, new))
        }
    }

    #[must_use]
    pub fn decision(&self, start: T, pos: &Position<T>, row: &State<T>) -> Map<T, Action<T>> {
        pos.top().map_or_else(
            || {
                pos.look
                    .iter()
                    .map(|l| {
                        (
                            <T>::clone(l),
                            if pos.rule == start {
                                Action::Acc
                            } else {
                                Action::Reduce(pos.rule, pos.seq.clone())
                            },
                        )
                    })
                    .collect()
            },
            |locus| {
                let filter = Tabler::sym_filter(row, &locus);
                let state = self
                    .state_from_kernel(&filter)
                    .expect("`kernels` is incomplete");
                if self.table.grammar.is_terminal(&locus) {
                    Map::from([(locus, Action::Shift(state))])
                } else {
                    Map::from([(locus, Action::Goto(state))])
                }
            },
        )
    }

    #[must_use]
    pub fn prop_closure(&self, seed: State<T>) -> State<T> {
        transitive(seed, |s| self.closure(s))
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, Clr, Parser};

    #[test]
    pub fn dragon_book() {
        let clr = Clr::new(grammars_tests::dragon_book());
        assert_eq!(0, clr.tables().conflicts().count());

        assert!(clr.validate(["d", "d"]));
        assert!(clr.validate(["d", "c", "d"]));
        assert!(clr.validate(["c", "d", "d"]));
        assert!(clr.validate(["d", "c", "c", "d"]));
        assert!(clr.validate(["c", "d", "c", "d"]));
        assert!(clr.validate(["c", "c", "d", "d"]));
        assert!(clr.validate(["d", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "d", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "d", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "d", "d"]));
        assert!(clr.validate(["d", "c", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "d", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "d", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "d", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "c", "d", "d"]));
        assert!(clr.validate(["d", "c", "c", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "d", "c", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "d", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "d", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "c", "d", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "c", "c", "d", "d"]));
        assert!(clr.validate(["d", "c", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "d", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "d", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "d", "c", "c", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "c", "c", "c", "d", "c", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "c", "c", "c", "c", "d", "c", "d"]));
        assert!(clr.validate(["c", "c", "c", "c", "c", "c", "c", "c", "d", "d"]));
        assert!(clr.validate(["d", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(clr.validate(["c", "d", "c", "c", "c", "c", "c", "c", "d"]));
    }

    #[test]
    pub fn wikipedia() {
        let clr = Clr::new(grammars_tests::wikipedia());
        assert_eq!(0, clr.tables().conflicts().count());

        assert!(clr.validate(["0"]));
        assert!(clr.validate(["1"]));
        assert!(clr.validate(["0", "*", "0"]));
        assert!(clr.validate(["0", "*", "1"]));
        assert!(clr.validate(["1", "*", "0"]));
        assert!(clr.validate(["1", "*", "1"]));
        assert!(clr.validate(["0", "+", "0"]));
        assert!(clr.validate(["0", "+", "1"]));
        assert!(clr.validate(["1", "+", "0"]));
        assert!(clr.validate(["1", "+", "1"]));
        assert!(clr.validate(["0", "*", "0", "*", "0"]));
        assert!(clr.validate(["0", "*", "0", "*", "1"]));
        assert!(clr.validate(["0", "*", "1", "*", "0"]));
        assert!(clr.validate(["0", "*", "1", "*", "1"]));
        assert!(clr.validate(["1", "*", "0", "*", "0"]));
        assert!(clr.validate(["1", "*", "0", "*", "1"]));
        assert!(clr.validate(["1", "*", "1", "*", "0"]));
        assert!(clr.validate(["1", "*", "1", "*", "1"]));
        assert!(clr.validate(["0", "+", "0", "*", "0"]));
        assert!(clr.validate(["0", "+", "0", "*", "1"]));
        assert!(clr.validate(["0", "+", "1", "*", "0"]));
        assert!(clr.validate(["0", "+", "1", "*", "1"]));
        assert!(clr.validate(["1", "+", "0", "*", "0"]));
        assert!(clr.validate(["1", "+", "0", "*", "1"]));
        assert!(clr.validate(["1", "+", "1", "*", "0"]));
        assert!(clr.validate(["1", "+", "1", "*", "1"]));
        assert!(clr.validate(["0", "*", "0", "+", "0"]));
        assert!(clr.validate(["0", "*", "0", "+", "1"]));
        assert!(clr.validate(["0", "*", "1", "+", "0"]));
        assert!(clr.validate(["0", "*", "1", "+", "1"]));
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    pub fn ucalgary_uni_oth_lr1() {
        let clr = Clr::new(grammars_tests::ucalgary_uni_oth_lr1());
        assert_eq!(0, clr.tables().conflicts().count());

        assert!(clr.validate(["e", "a", "c"]));
        assert!(clr.validate(["d", "a", "b"]));
        assert!(clr.validate(["d", "e", "a", "c"]));
        assert!(clr.validate(["d", "e", "a", "b"]));
        assert!(clr.validate(["e", "d", "a", "b"]));
        assert!(clr.validate(["e", "d", "a", "c"]));
        assert!(clr.validate(["d", "d", "e", "a", "b"]));
        assert!(clr.validate(["e", "e", "d", "a", "c"]));
    }

    #[test]
    pub fn serokell() {
        let clr = Clr::new(grammars_tests::serokell());
        assert_eq!(0, clr.tables().conflicts().count());

        assert!(clr.validate(["int"]));
        assert!(clr.validate(["int", "*", "int"]));
        assert!(clr.validate(["ident", "*", "int"]));
        assert!(clr.validate(["(", "int", ")"]));
        assert!(clr.validate(["int", "+", "int"]));
        assert!(clr.validate(["ident", "+", "int"]));
        assert!(clr.validate(["int", "*", "int", "*", "int"]));
        assert!(clr.validate(["int", "*", "ident", "*", "int"]));
        assert!(clr.validate(["ident", "*", "int", "*", "int"]));
        assert!(clr.validate(["ident", "*", "ident", "*", "int"]));
        assert!(clr.validate(["int", "*", "(", "int", ")"]));
        assert!(clr.validate(["ident", "*", "(", "int", ")"]));
        assert!(clr.validate(["int", "*", "int", "+", "int"]));
        assert!(clr.validate(["int", "*", "(", "ident", "+", "int", ")"]));
        assert!(clr.validate(["ident", "*", "int", "+", "int"]));
        assert!(clr.validate([
            "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(",
            "(", "(", "(", "(", "(", "int", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
            ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
        ]));
    }

    #[test]
    pub fn puncs() {
        let clr = Clr::new(grammars_tests::puncs());
        assert_eq!(0, clr.tables().conflicts().count());

        assert!(clr.validate(["(", ")"]));
        assert!(clr.validate(["[", "]"]));
        assert!(clr.validate(["{", "}"]));
        assert!(clr.validate(["(", "(", ")", ")"]));
        assert!(clr.validate(["(", "[", "]", ")"]));
        assert!(clr.validate(["(", "{", "}", ")"]));
        assert!(clr.validate(["[", "(", ")", "]"]));
        assert!(clr.validate(["[", "[", "]", "]"]));
        assert!(clr.validate(["[", "{", "}", "]"]));
        assert!(clr.validate(["{", "(", ")", "}"]));
        assert!(clr.validate(["{", "[", "]", "}"]));
        assert!(clr.validate(["{", "{", "}", "}"]));
        assert!(clr.validate(["(", "(", "(", ")", ")", ")"]));
        assert!(clr.validate(["(", "(", "[", "]", ")", ")"]));
        assert!(clr.validate(["(", "(", "{", "}", ")", ")"]));
        assert!(clr.validate(["(", "[", "(", ")", "]", ")"]));
        assert!(clr.validate(["(", "[", "[", "]", "]", ")"]));
        assert!(clr.validate(["(", "[", "{", "}", "]", ")"]));
        assert!(clr.validate(["(", "{", "(", ")", "}", ")"]));
        assert!(clr.validate(["(", "{", "[", "]", "}", ")"]));
        assert!(clr.validate(["(", "{", "{", "}", "}", ")"]));
        assert!(clr.validate(["[", "(", "(", ")", ")", "]"]));
        assert!(clr.validate(["[", "(", "[", "]", ")", "]"]));
        assert!(clr.validate(["[", "(", "{", "}", ")", "]"]));
        assert!(clr.validate(["[", "[", "(", ")", "]", "]"]));
        assert!(clr.validate(["[", "[", "[", "]", "]", "]"]));
        assert!(clr.validate(["[", "[", "{", "}", "]", "]"]));
        assert!(clr.validate(["[", "{", "(", ")", "}", "]"]));
        assert!(clr.validate(["[", "{", "[", "]", "}", "]"]));
        assert!(clr.validate(["[", "{", "{", "}", "}", "]"]));
    }

    #[test]
    pub fn scanner() {
        let clr = Clr::new(grammars_tests::scanner());
        assert_eq!(0, clr.tables().conflicts().count());

        assert!(clr.validate([
            "l", "o", "r", "e", "m", "_", "i", "p", "s", "u", "m", "_", "d", "o", "l", "o", "r",
            "_", "s", "i", "t", "_", "a", "m", "e", "t",
        ]));
        assert!(clr.validate([
            "i", "n", "_", "v", "i", "n", "o", "_", "v", "e", "r", "i", "t", "a", "s", "_", "b",
            "e", "f", "o", "r", "e", "_", "7", "9", "_", "a", "c",
        ]));
        assert!(clr.validate([
            "1", "2", "_", "t", "i", "m", "e", "s", "_", "3", "_", "i", "s", "_", "e", "q", "u",
            "a", "l", "_", "t", "o", "_", "4", "0", "_", "m", "i", "n", "u", "s", "_", "4",
        ]));
        assert!(clr.validate(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]));
        assert!(clr.validate([
            "f", "i", "n", "a", "l", "_", "d", "e", "_", "s", "e", "m", "a", "n", "a", "_", "e",
            "l", "a", "_", "v", "a", "i", "_", "p", "r", "a", "_", "r", "u", "a",
        ]));
    }
}
