use crate::{transitive, Action, Map, Parser, Position, Rule, Set, State, Tabler, Term};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slr {
    pub table: Tabler,
}

impl Parser for Slr {
    fn with_table(table: Tabler) -> Self {
        let mut parser = Self::uninit(table);
        parser.proc_actions();
        parser
    }

    fn uninit(table: Tabler) -> Self {
        Self { table }
    }

    fn tables(&self) -> &Tabler {
        &self.table
    }

    fn tables_mut(&mut self) -> &mut Tabler {
        &mut self.table
    }
}

impl Slr {
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.table.states.push(start);
    }

    #[must_use]
    pub fn closure(&self, state: State) -> State {
        let mut new_state = State::new();
        for pos in &state {
            if let Some(top) = pos.top() {
                if self.table.grammar.is_terminal(&top) {
                    continue;
                }
                for prod in self.table.grammar.rules[top].prods() {
                    new_state.insert(Position::new(top, prod.clone(), 0, Set::new()));
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
    pub fn decision(&self, start: Rule, pos: &Position, row: &State) -> Map<Term, Action> {
        pos.top().map_or_else(
            || {
                self.table.follow[pos.rule]
                    .iter()
                    .map(|l| {
                        (
                            <&str>::clone(l),
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
                    .table
                    .kernels
                    .get(&filter)
                    .expect("`kernels` is incomplete");
                if self.table.grammar.is_terminal(&locus) {
                    Map::from([(locus, Action::Shift(*state))])
                } else {
                    Map::from([(locus, Action::Goto(*state))])
                }
            },
        )
    }

    #[must_use]
    pub fn goto(&self, kernels: &State, sym: &Term) -> Option<(State, State)> {
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

    /// # Panics
    /// Never.
    pub fn proc_actions(&mut self) {
        self.proc_closures();
        let start = self.table.basis_pos().rule;
        for row in &self.table.states {
            let mut map: Map<Term, Action> = Map::new();
            for item in row {
                for (term, act) in self.decision(start, item, row) {
                    if map.contains_key(term) && map[term] != act {
                        *map.get_mut(term).unwrap() =
                            Action::Conflict(Box::new(map.get(term).unwrap().clone()), act.into());
                    } else {
                        map.insert(term, act);
                    }
                }
            }
            self.table.actions.push(map);
        }
    }

    #[must_use]
    pub fn prop_closure(&self, seed: State) -> State {
        transitive(seed, |s| self.closure(s))
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, Parser, Slr};

    #[test]
    pub fn dragon_book() {
        let slr = Slr::new(grammars_tests::dragon_book());
        assert_eq!(0, slr.tables().conflicts().count());

        assert!(slr.validate(["d", "d"]));
        assert!(slr.validate(["d", "c", "d"]));
        assert!(slr.validate(["c", "d", "d"]));
        assert!(slr.validate(["d", "c", "c", "d"]));
        assert!(slr.validate(["c", "d", "c", "d"]));
        assert!(slr.validate(["c", "c", "d", "d"]));
        assert!(slr.validate(["d", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "d", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "d", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "d", "d"]));
        assert!(slr.validate(["d", "c", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "d", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "d", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "d", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "c", "d", "d"]));
        assert!(slr.validate(["d", "c", "c", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "d", "c", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "d", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "d", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "c", "d", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "c", "c", "d", "d"]));
        assert!(slr.validate(["d", "c", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "d", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "d", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "d", "c", "c", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "c", "c", "c", "d", "c", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "c", "c", "c", "c", "d", "c", "d"]));
        assert!(slr.validate(["c", "c", "c", "c", "c", "c", "c", "c", "d", "d"]));
        assert!(slr.validate(["d", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(slr.validate(["c", "d", "c", "c", "c", "c", "c", "c", "d"]));
    }

    #[test]
    fn wikipedia() {
        let slr = Slr::new(grammars_tests::wikipedia());
        assert_eq!(0, slr.tables().conflicts().count());

        assert!(slr.validate(["0"]));
        assert!(slr.validate(["1"]));
        assert!(slr.validate(["0", "*", "0"]));
        assert!(slr.validate(["0", "*", "1"]));
        assert!(slr.validate(["1", "*", "0"]));
        assert!(slr.validate(["1", "*", "1"]));
        assert!(slr.validate(["0", "+", "0"]));
        assert!(slr.validate(["0", "+", "1"]));
        assert!(slr.validate(["1", "+", "0"]));
        assert!(slr.validate(["1", "+", "1"]));
        assert!(slr.validate(["0", "*", "0", "*", "0"]));
        assert!(slr.validate(["0", "*", "0", "*", "1"]));
        assert!(slr.validate(["0", "*", "1", "*", "0"]));
        assert!(slr.validate(["0", "*", "1", "*", "1"]));
        assert!(slr.validate(["1", "*", "0", "*", "0"]));
        assert!(slr.validate(["1", "*", "0", "*", "1"]));
        assert!(slr.validate(["1", "*", "1", "*", "0"]));
        assert!(slr.validate(["1", "*", "1", "*", "1"]));
        assert!(slr.validate(["0", "+", "0", "*", "0"]));
        assert!(slr.validate(["0", "+", "0", "*", "1"]));
        assert!(slr.validate(["0", "+", "1", "*", "0"]));
        assert!(slr.validate(["0", "+", "1", "*", "1"]));
        assert!(slr.validate(["1", "+", "0", "*", "0"]));
        assert!(slr.validate(["1", "+", "0", "*", "1"]));
        assert!(slr.validate(["1", "+", "1", "*", "0"]));
        assert!(slr.validate(["1", "+", "1", "*", "1"]));
        assert!(slr.validate(["0", "*", "0", "+", "0"]));
        assert!(slr.validate(["0", "*", "0", "+", "1"]));
        assert!(slr.validate(["0", "*", "1", "+", "0"]));
        assert!(slr.validate(["0", "*", "1", "+", "1"]));
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    fn ucalgary_uni_oth_lr1() {
        let slr = Slr::new(grammars_tests::ucalgary_uni_oth_lr1());
        assert_eq!(2, slr.tables().conflicts().count());

        assert!(slr.validate(["e", "a", "c"]));
        assert!(slr.validate(["d", "a", "b"]));
        assert!(!slr.validate(["d", "e", "a", "c"]));
        assert!(!slr.validate(["d", "e", "a", "b"]));
        assert!(!slr.validate(["e", "d", "a", "b"]));
        assert!(!slr.validate(["e", "d", "a", "c"]));
        assert!(slr.validate(["d", "d", "e", "a", "b"]));
        assert!(slr.validate(["e", "e", "d", "a", "c"]));
    }

    #[test]
    fn serokell() {
        let slr = Slr::new(grammars_tests::serokell());
        assert_eq!(0, slr.tables().conflicts().count());

        assert!(slr.validate(["int"]));
        assert!(slr.validate(["int", "*", "int"]));
        assert!(slr.validate(["ident", "*", "int"]));
        assert!(slr.validate(["(", "int", ")"]));
        assert!(slr.validate(["int", "+", "int"]));
        assert!(slr.validate(["ident", "+", "int"]));
        assert!(slr.validate(["int", "*", "int", "*", "int"]));
        assert!(slr.validate(["int", "*", "ident", "*", "int"]));
        assert!(slr.validate(["ident", "*", "int", "*", "int"]));
        assert!(slr.validate(["ident", "*", "ident", "*", "int"]));
        assert!(slr.validate(["int", "*", "(", "int", ")"]));
        assert!(slr.validate(["ident", "*", "(", "int", ")"]));
        assert!(slr.validate(["int", "*", "int", "+", "int"]));
        assert!(slr.validate(["int", "*", "(", "ident", "+", "int", ")"]));
        assert!(slr.validate(["ident", "*", "int", "+", "int"]));
        assert!(slr.validate([
            "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(",
            "(", "(", "(", "(", "(", "int", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
            ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
        ]));
    }

    #[test]
    pub fn puncs() {
        let slr = Slr::new(grammars_tests::puncs());
        assert_eq!(0, slr.tables().conflicts().count());

        assert!(slr.validate(["(", ")"]));
        assert!(slr.validate(["[", "]"]));
        assert!(slr.validate(["{", "}"]));
        assert!(slr.validate(["(", "(", ")", ")"]));
        assert!(slr.validate(["(", "[", "]", ")"]));
        assert!(slr.validate(["(", "{", "}", ")"]));
        assert!(slr.validate(["[", "(", ")", "]"]));
        assert!(slr.validate(["[", "[", "]", "]"]));
        assert!(slr.validate(["[", "{", "}", "]"]));
        assert!(slr.validate(["{", "(", ")", "}"]));
        assert!(slr.validate(["{", "[", "]", "}"]));
        assert!(slr.validate(["{", "{", "}", "}"]));
        assert!(slr.validate(["(", "(", "(", ")", ")", ")"]));
        assert!(slr.validate(["(", "(", "[", "]", ")", ")"]));
        assert!(slr.validate(["(", "(", "{", "}", ")", ")"]));
        assert!(slr.validate(["(", "[", "(", ")", "]", ")"]));
        assert!(slr.validate(["(", "[", "[", "]", "]", ")"]));
        assert!(slr.validate(["(", "[", "{", "}", "]", ")"]));
        assert!(slr.validate(["(", "{", "(", ")", "}", ")"]));
        assert!(slr.validate(["(", "{", "[", "]", "}", ")"]));
        assert!(slr.validate(["(", "{", "{", "}", "}", ")"]));
        assert!(slr.validate(["[", "(", "(", ")", ")", "]"]));
        assert!(slr.validate(["[", "(", "[", "]", ")", "]"]));
        assert!(slr.validate(["[", "(", "{", "}", ")", "]"]));
        assert!(slr.validate(["[", "[", "(", ")", "]", "]"]));
        assert!(slr.validate(["[", "[", "[", "]", "]", "]"]));
        assert!(slr.validate(["[", "[", "{", "}", "]", "]"]));
        assert!(slr.validate(["[", "{", "(", ")", "}", "]"]));
        assert!(slr.validate(["[", "{", "[", "]", "}", "]"]));
        assert!(slr.validate(["[", "{", "{", "}", "}", "]"]));
    }
}
