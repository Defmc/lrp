use crate::{Action, Grammar, Map, Parser, Position, Rule, Set, State, Tabler, Term};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Clr {
    pub table: Tabler,
}

impl Parser for Clr {
    fn new(grammar: Grammar) -> Self {
        let mut parser = Self {
            table: Tabler::new(grammar),
        };
        parser.proc_actions();
        parser
    }

    #[must_use]
    fn tables(&self) -> &Tabler {
        &self.table
    }

    #[must_use]
    fn tables_mut(&mut self) -> &mut Tabler {
        &mut self.table
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
                debug_assert!(self
                    .table
                    .kernels
                    .insert(kernel, self.table.states.len())
                    .is_none());
                self.table.states.push(closures);
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

impl Clr {
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.table.states.push(start);
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
    fn wikipedia() {
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
    fn ucalgary_uni_oth_lr1() {
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
    fn serokell() {
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
}
