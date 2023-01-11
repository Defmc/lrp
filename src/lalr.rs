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
}

impl Lalr {
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.raws.insert(State::new(), start.clone());
        self.table.states.push(start);
    }

    #[must_use]
    pub fn kernel_like(&self, kernel: &State) -> Option<usize> {
        let raw = Self::without_look(kernel);
        let kernel = self.raws.get(&raw)?;
        self.table.kernels.get(kernel).copied()
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
        for (k, &i) in kers {
            println!("\nstate: {i}");
            println!("kernel: {:?}", k);
            println!("raw kernel: {:?}", Self::without_look(k));
            for state in &self.table.states[i] {
                println!("\t{state:?}");
            }
        }
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
            println!("new {row:#?}");
            for s in &syms {
                let Some((kernel, closures)) = self.goto(&row, &s) else {
                    continue;
                };
                let raw_kernel = Self::without_look(&kernel);
                let main_kernel = self.raws.get(&raw_kernel).cloned();
                let old = if let Some(main_kernel) = main_kernel {
                    let Some((old, nk)) = self.update_closures(main_kernel, kernel, closures) else {
                        continue;
                    };
                    let updated_state = self.table.kernels[&nk];
                    if updated_state == idx && nk != row {
                        println!("{updated_state} == {idx}\n\t{nk:#?} != {row:#?}");
                        continue 'gen;
                    }
                    old
                } else {
                    self.raws.insert(raw_kernel, kernel.clone());
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
    pub fn update_closures(
        &mut self,
        main: State,
        inc: State,
        closures: State,
    ) -> Option<(Option<usize>, State)> {
        print!("updating {main:#?} with {inc:#?}");
        let syms: Vec<_> = self.table.grammar.symbols().collect();
        let (old, mut new_kernel) = self.update_state(main, inc, closures)?;
        let id = self.table.kernels[&new_kernel];
        println!(" turned into {new_kernel:#?}");

        'regen: loop {
            for s in &syms {
                let Some((kernel, closures)) = self.goto(&new_kernel, &s) else {
                    continue;
                };
                let raw_kernel = Self::without_look(&kernel);
                let main_kernel = self.raws.get(&raw_kernel).cloned();
                let Some(main_kernel) = main_kernel else {
                continue;
            };
                let Some((old, nk)) = self.update_closures(main_kernel, kernel, closures) else {
                    continue;
                };
                let updated_state = self.table.kernels[&nk];
                if updated_state == id && nk != new_kernel {
                    println!("{updated_state} IS EQUAL {id}\n\t{nk:#?} IS DIFF {new_kernel:#?}");
                    new_kernel = nk;
                    continue 'regen;
                }
                debug_assert!(old.is_none());
            }
            break;
        }
        Some((old, new_kernel))
    }

    #[must_use]
    pub fn update_state(
        &mut self,
        main: State,
        inc: State,
        closures: State,
    ) -> Option<(Option<usize>, State)> {
        // TODO: Build a custom merging state function
        let state_id = self.table.kernels[&main];
        let both_kernels = inc
            .clone()
            .into_iter()
            .chain(main.clone().into_iter())
            .collect();
        let both_closures = self.table.states[state_id]
            .clone()
            .into_iter()
            .chain(closures.into_iter())
            .collect();
        let (new_kernel, new_closures) = (Self::merged(both_kernels), Self::merged(both_closures));
        println!("before: {:?}", self.table.states[state_id]);
        if self.table.states[state_id] == new_closures {
            None?
        }
        self.table.kernels.remove(&main);
        self.table.states[state_id] = new_closures;
        println!("after: {:?}", self.table.states[state_id]);

        let raw_kernel = Self::without_look(&new_kernel);
        self.raws.insert(raw_kernel, new_kernel.clone());

        let old = self.table.kernels.insert(new_kernel.clone(), state_id);
        Some((old, new_kernel))
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
        let mut looks: Map<Position, Set<Term>> = Map::new();
        for state in states {
            let no_look = state.no_look();
            if let Some(look) = looks.get_mut(&no_look) {
                look.extend(state.look);
            } else {
                new.insert(no_look.clone());
                looks.insert(no_look, state.look);
            }
        }
        let merged = new
            .into_iter()
            .map(|s| {
                let look = looks[&s].clone();
                s.with_look(look)
            })
            .collect();
        merged
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
}
