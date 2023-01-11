use crate::{transitive, Action, Map, Parser, Position, Rule, Set, State, Tabler, Term};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lalr {
    pub table: Tabler,
    /// Describes the "raw states" (states without lookahead symbol) in `table.kernels`, using it
    /// raw kernel as key and value being its final kernel
    pub raws: Map<State, State>,
}

impl Parser for Lalr {
    fn new(grammar: crate::Grammar) -> Self
    where
        Self: Sized,
    {
        Self::with_table(Tabler::new(grammar))
    }

    fn with_table(table: Tabler) -> Self {
        let mut parser = Self::uninit(table);
        parser.proc_actions();
        parser
    }

    fn uninit(table: Tabler) -> Self {
        Self {
            table,
            raws: Map::new(),
        }
    }

    fn tables(&self) -> &Tabler {
        &self.table
    }

    fn tables_mut(&mut self) -> &mut Tabler {
        &mut self.table
    }

    fn final_kernel<'a>(&'a self, kernel: &'a State) -> Option<&'a State> {
        let raw = Self::without_look(kernel);
        self.raws.get(&raw)
    }
}

impl Lalr {
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.raws.insert(State::new(), start.clone());
        self.table.states.push(start);
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
        let mut kers: Vec<_> = self.table.kernels.iter().collect();
        kers.sort_by_key(|(_, i)| *i);
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
        let syms: Vec<_> = self.table.grammar.symbols().collect();
        'gen: while idx < self.table.states.len() {
            let row = self.table.states[idx].clone();
            for s in &syms {
                let Some((kernel, closures)) = self.goto(&row, s) else {
                    continue;
                };
                let raw_kernel = Self::without_look(&kernel);
                if let Some(main_kernel) = self.raws.get(&raw_kernel).cloned() {
                    let Some(nk) = self.update_closures(&main_kernel, kernel, closures) else {
                        continue;
                    };
                    if self.table.kernels[&nk] == idx && nk != row {
                        continue 'gen;
                    }
                } else {
                    self.raws.insert(raw_kernel, kernel.clone());
                    self.table.states.push(closures);
                    let old = self
                        .table
                        .kernels
                        .insert(kernel, self.table.states.len() - 1);
                    debug_assert!(old.is_none());
                };
            }
            idx += 1;
        }
    }

    #[must_use]
    pub fn update_closures(&mut self, main: &State, inc: State, closures: State) -> Option<State> {
        let mut new_kernel = self.update_state(main, inc, closures)?;
        let syms: Vec<_> = self.table.grammar.symbols().collect();
        let id = self.table.kernels[&new_kernel];

        'regen: loop {
            for s in &syms {
                let Some((kernel, closures)) = self.goto(&new_kernel, s) else {
                    continue;
                };
                let raw_kernel = Self::without_look(&kernel);
                let main_kernel = self.raws.get(&raw_kernel).cloned();
                let Some(main_kernel) = main_kernel else {
                continue;
            };
                let Some(nk) = self.update_closures(&main_kernel, kernel, closures) else {
                    continue;
                };
                let updated_state = self.table.kernels[&nk];
                if updated_state == id && nk != new_kernel {
                    new_kernel = nk;
                    continue 'regen;
                }
            }
            break;
        }
        Some(new_kernel)
    }

    #[must_use]
    pub fn update_state(&mut self, main: &State, inc: State, closures: State) -> Option<State> {
        let state_id = *self.table.kernels.get(main)?;

        // By definition, there will be always more closure items than kernels (because
        // essencially, every kernel can become a reduction closure or more than one shifting).
        // So it's better to check kernel equality first.
        let both_kernels = inc.into_iter().chain(main.clone().into_iter()).collect();
        let new_kernel = Self::merged(both_kernels);

        if &new_kernel == main {
            None?;
        }

        let both_closures = self.table.states[state_id]
            .clone()
            .into_iter()
            .chain(closures.into_iter())
            .collect();

        self.table.kernels.remove(main);
        self.table.states[state_id] = Self::merged(both_closures);

        self.raws
            .insert(Self::without_look(&new_kernel), new_kernel.clone());

        let old = self.table.kernels.insert(new_kernel.clone(), state_id);
        debug_assert!(old.is_none());
        Some(new_kernel)
    }

    #[must_use]
    fn goto(&self, kernels: &State, sym: &Term) -> Option<(State, State)> {
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
        assert!(!lalr.validate(["d", "e", "a", "c"]));
        assert!(!lalr.validate(["d", "e", "a", "b"]));
        assert!(!lalr.validate(["e", "d", "a", "b"]));
        assert!(!lalr.validate(["e", "d", "a", "c"]));
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

    #[test]
    pub fn scanner() {
        let lalr = Lalr::new(grammars_tests::scanner());
        assert_eq!(0, lalr.tables().conflicts().count());

        assert!(lalr.validate([
            "l", "o", "r", "e", "m", "_", "i", "p", "s", "u", "m", "_", "d", "o", "l", "o", "r",
            "_", "s", "i", "t", "_", "a", "m", "e", "t",
        ]));
        assert!(lalr.validate([
            "i", "n", "_", "v", "i", "n", "o", "_", "v", "e", "r", "i", "t", "a", "s", "_", "b",
            "e", "f", "o", "r", "e", "_", "7", "9", "_", "a", "c",
        ]));
        assert!(lalr.validate([
            "1", "2", "_", "t", "i", "m", "e", "s", "_", "3", "_", "i", "s", "_", "e", "q", "u",
            "a", "l", "_", "t", "o", "_", "4", "0", "_", "m", "i", "n", "u", "s", "_", "4",
        ]));
        assert!(lalr.validate(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]));
        assert!(lalr.validate([
            "f", "i", "n", "a", "l", "_", "d", "e", "_", "s", "e", "m", "a", "n", "a", "_", "e",
            "l", "a", "_", "v", "a", "i", "_", "p", "r", "a", "_", "r", "u", "a",
        ]));
    }
}
