use crate::{transitive, Action, Map, Parser, Position, Rule, Set, State, Tabler, Term};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lalr {
    pub table: Tabler,
    /// Describes the "raw states" (states without lookahead symbol) in `table.states`, using it
    /// raw closure as key and value being its index
    pub raws: Map<State, usize>,
}

impl Parser for Lalr {
    fn new(grammar: crate::Grammar) -> Self
    where
        Self: Sized,
    {
        Self::with_table(Tabler::new(grammar))
    }

    fn with_table(table: Tabler) -> Self {
        let mut parser = Self {
            table,
            raws: Map::new(),
        };
        parser.proc_actions();
        parser
    }

    fn tables(&self) -> &Tabler {
        &self.table
    }

    fn tables_mut(&mut self) -> &mut Tabler {
        &mut self.table
    }
}

impl Lalr {
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.table.states.push(start);
    }

    #[must_use]
    pub fn kernel_like(&self, kernel: &State) -> Option<usize> {
        let eq = |k: &Position, s: &Position| {
            k.body_eq(s) && k.look.iter().all(|ak| s.look.contains(ak))
        };
        let idx = self
            .table
            .kernels
            .iter()
            .find(|(st, _)| kernel.iter().all(|k| st.iter().any(|s| eq(k, s))))?
            .1;
        Some(*idx)
    }

    #[must_use]
    pub fn without_look(state: &State) -> State {
        state.iter().map(Position::no_look).collect()
    }

    #[must_use]
    pub fn prop_closure(&self, seed: State) -> State {
        transitive(seed, |s| self.closure(s))
    }

    fn proc_actions(&mut self) {
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
    fn closure(&self, state: State) -> State {
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
                for prod in self.table.grammar.rules[top].prods() {
                    new_state.insert(Position::new(top, prod.clone(), 0, look.clone()));
                }
            }
        }
        new_state.extend(state);
        new_state
    }

    fn proc_closures(&mut self) {
        self.proc_closures_first_row();
        let mut idx = 0;
        while idx < self.table.states.len() {
            let row = self.table.states[idx].clone();
            for s in self.table.grammar.symbols() {
                let Some((kernel, closures)) = self.goto(row.clone(), &s) else {
                    continue;
                };
                let without_look = Self::without_look(&closures);
                let old = if let Some(&state) = self.raws.get(&without_look) {
                    // TODO: Build a custom merging state function
                    let both = self.table.states[state]
                        .iter()
                        .chain(closures.iter())
                        .cloned()
                        .collect();
                    self.table.states[state] = Self::merged(both);
                    self.table.kernels.insert(kernel, state)
                } else {
                    self.raws.insert(without_look, self.table.states.len());
                    self.table.states.push(closures);
                    self.table
                        .kernels
                        .insert(kernel, self.table.states.len() - 1)
                };
                debug_assert!(old.is_none());
            }
            idx += 1;
        }
    }

    #[must_use]
    fn goto(&self, kernels: State, sym: &Term) -> Option<(State, State)> {
        let kernels = Tabler::sym_filter(&kernels, sym);
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
    fn decision(&self, start: Rule, pos: &Position, row: &State) -> Map<Term, Action> {
        pos.top().map_or_else(
            || {
                pos.look
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
                let state = self.kernel_like(&filter).expect("`kernels` is incomplete");
                if self.table.grammar.is_terminal(&locus) {
                    Map::from([(locus, Action::Shift(state))])
                } else {
                    Map::from([(locus, Action::Goto(state))])
                }
            },
        )
    }

    #[must_use]
    fn merged(states: State) -> State {
        let mut new = State::new();
        'outter: for state in states {
            let keys: Vec<_> = new.iter().cloned().collect();
            for key in keys {
                if new.get(&key).unwrap().body_eq(&state) {
                    let mut state = state;
                    state.look.extend(new.get(&key).unwrap().look.clone());
                    new.remove(&key);
                    new.insert(state);
                    continue 'outter;
                }
            }
            new.insert(state);
        }
        new
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, Lalr, Parser};

    #[test]
    pub fn dragon_book() {
        let lalr = Lalr::new(grammars_tests::dragon_book());
        assert_eq!(0, lalr.tables().conflicts().count());

        assert!(lalr.validate(["d", "d"]));
        assert!(lalr.validate(["d", "c", "d"]));
        assert!(lalr.validate(["c", "d", "d"]));
        assert!(lalr.validate(["d", "c", "c", "d"]));
        assert!(lalr.validate(["c", "d", "c", "d"]));
        assert!(lalr.validate(["c", "c", "d", "d"]));
        assert!(lalr.validate(["d", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "d", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "d", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "d", "d"]));
        assert!(lalr.validate(["d", "c", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "d", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "d", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "d", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "c", "d", "d"]));
        assert!(lalr.validate(["d", "c", "c", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "d", "c", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "d", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "d", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "c", "d", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "c", "c", "d", "d"]));
        assert!(lalr.validate(["d", "c", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "d", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "d", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "d", "c", "c", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "c", "c", "c", "d", "c", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "c", "c", "c", "c", "d", "c", "d"]));
        assert!(lalr.validate(["c", "c", "c", "c", "c", "c", "c", "c", "d", "d"]));
        assert!(lalr.validate(["d", "c", "c", "c", "c", "c", "c", "c", "d"]));
        assert!(lalr.validate(["c", "d", "c", "c", "c", "c", "c", "c", "d"]));
    }

    #[test]
    fn wikipedia() {
        let lalr = Lalr::new(grammars_tests::wikipedia());
        assert_eq!(0, lalr.tables().conflicts().count());

        assert!(lalr.validate(["0"]));
        assert!(lalr.validate(["1"]));
        assert!(lalr.validate(["0", "*", "0"]));
        assert!(lalr.validate(["0", "*", "1"]));
        assert!(lalr.validate(["1", "*", "0"]));
        assert!(lalr.validate(["1", "*", "1"]));
        assert!(lalr.validate(["0", "+", "0"]));
        assert!(lalr.validate(["0", "+", "1"]));
        assert!(lalr.validate(["1", "+", "0"]));
        assert!(lalr.validate(["1", "+", "1"]));
        assert!(lalr.validate(["0", "*", "0", "*", "0"]));
        assert!(lalr.validate(["0", "*", "0", "*", "1"]));
        assert!(lalr.validate(["0", "*", "1", "*", "0"]));
        assert!(lalr.validate(["0", "*", "1", "*", "1"]));
        assert!(lalr.validate(["1", "*", "0", "*", "0"]));
        assert!(lalr.validate(["1", "*", "0", "*", "1"]));
        assert!(lalr.validate(["1", "*", "1", "*", "0"]));
        assert!(lalr.validate(["1", "*", "1", "*", "1"]));
        assert!(lalr.validate(["0", "+", "0", "*", "0"]));
        assert!(lalr.validate(["0", "+", "0", "*", "1"]));
        assert!(lalr.validate(["0", "+", "1", "*", "0"]));
        assert!(lalr.validate(["0", "+", "1", "*", "1"]));
        assert!(lalr.validate(["1", "+", "0", "*", "0"]));
        assert!(lalr.validate(["1", "+", "0", "*", "1"]));
        assert!(lalr.validate(["1", "+", "1", "*", "0"]));
        assert!(lalr.validate(["1", "+", "1", "*", "1"]));
        assert!(lalr.validate(["0", "*", "0", "+", "0"]));
        assert!(lalr.validate(["0", "*", "0", "+", "1"]));
        assert!(lalr.validate(["0", "*", "1", "+", "0"]));
        assert!(lalr.validate(["0", "*", "1", "+", "1"]));
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    fn ucalgary_uni_oth_lr1() {
        let lalr = Lalr::new(grammars_tests::ucalgary_uni_oth_lr1());
        assert_eq!(2, lalr.tables().conflicts().count());

        assert!(lalr.validate(["e", "a", "c"]));
        assert!(lalr.validate(["d", "a", "b"]));
        assert!(lalr.validate(["d", "e", "a", "c"]));
        assert!(lalr.validate(["d", "e", "a", "b"]));
        assert!(lalr.validate(["e", "d", "a", "b"]));
        assert!(lalr.validate(["e", "d", "a", "c"]));
        assert!(lalr.validate(["d", "d", "e", "a", "b"]));
        assert!(lalr.validate(["e", "e", "d", "a", "c"]));
    }

    #[test]
    fn serokell() {
        let lalr = Lalr::new(grammars_tests::serokell());
        assert_eq!(0, lalr.tables().conflicts().count());

        assert!(lalr.validate(["int"]));
        assert!(lalr.validate(["int", "*", "int"]));
        assert!(lalr.validate(["ident", "*", "int"]));
        assert!(lalr.validate(["(", "int", ")"]));
        assert!(lalr.validate(["int", "+", "int"]));
        assert!(lalr.validate(["ident", "+", "int"]));
        assert!(lalr.validate(["int", "*", "int", "*", "int"]));
        assert!(lalr.validate(["int", "*", "ident", "*", "int"]));
        assert!(lalr.validate(["ident", "*", "int", "*", "int"]));
        assert!(lalr.validate(["ident", "*", "ident", "*", "int"]));
        assert!(lalr.validate(["int", "*", "(", "int", ")"]));
        assert!(lalr.validate(["ident", "*", "(", "int", ")"]));
        assert!(lalr.validate(["int", "*", "int", "+", "int"]));
        assert!(lalr.validate(["int", "*", "(", "ident", "+", "int", ")"]));
        assert!(lalr.validate(["ident", "*", "int", "+", "int"]));
        assert!(lalr.validate([
            "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(",
            "(", "(", "(", "(", "(", "int", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
            ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
        ]));
    }

    #[test]
    pub fn puncs() {
        let lalr = Lalr::new(grammars_tests::puncs());
        assert_eq!(0, lalr.tables().conflicts().count());

        assert!(lalr.validate(["(", ")"]));
        assert!(lalr.validate(["[", "]"]));
        assert!(lalr.validate(["{", "}"]));
        assert!(lalr.validate(["(", "(", ")", ")"]));
        assert!(lalr.validate(["(", "[", "]", ")"]));
        assert!(lalr.validate(["(", "{", "}", ")"]));
        assert!(lalr.validate(["[", "(", ")", "]"]));
        assert!(lalr.validate(["[", "[", "]", "]"]));
        assert!(lalr.validate(["[", "{", "}", "]"]));
        assert!(lalr.validate(["{", "(", ")", "}"]));
        assert!(lalr.validate(["{", "[", "]", "}"]));
        assert!(lalr.validate(["{", "{", "}", "}"]));
        assert!(lalr.validate(["(", "(", "(", ")", ")", ")"]));
        assert!(lalr.validate(["(", "(", "[", "]", ")", ")"]));
        assert!(lalr.validate(["(", "(", "{", "}", ")", ")"]));
        assert!(lalr.validate(["(", "[", "(", ")", "]", ")"]));
        assert!(lalr.validate(["(", "[", "[", "]", "]", ")"]));
        assert!(lalr.validate(["(", "[", "{", "}", "]", ")"]));
        assert!(lalr.validate(["(", "{", "(", ")", "}", ")"]));
        assert!(lalr.validate(["(", "{", "[", "]", "}", ")"]));
        assert!(lalr.validate(["(", "{", "{", "}", "}", ")"]));
        assert!(lalr.validate(["[", "(", "(", ")", ")", "]"]));
        assert!(lalr.validate(["[", "(", "[", "]", ")", "]"]));
        assert!(lalr.validate(["[", "(", "{", "}", ")", "]"]));
        assert!(lalr.validate(["[", "[", "(", ")", "]", "]"]));
        assert!(lalr.validate(["[", "[", "[", "]", "]", "]"]));
        assert!(lalr.validate(["[", "[", "{", "}", "]", "]"]));
        assert!(lalr.validate(["[", "{", "(", ")", "}", "]"]));
        assert!(lalr.validate(["[", "{", "[", "]", "}", "]"]));
        assert!(lalr.validate(["[", "{", "{", "}", "}", "]"]));
    }
}
