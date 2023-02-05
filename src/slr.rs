use crate::{transitive, Action, Map, Parser, Position, Set, State, Tabler};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slr<T>
where
    T: PartialEq + Ord + Clone + Debug,
{
    pub table: Tabler<T>,
}

impl<T> Parser<T> for Slr<T>
where
    T: PartialEq + Ord + Clone + Debug,
{
    fn with_table(table: Tabler<T>) -> Self {
        let mut parser = Self::uninit(table);
        parser.proc_actions();
        parser
    }

    fn uninit(table: Tabler<T>) -> Self {
        Self { table }
    }

    fn tables(&self) -> &Tabler<T> {
        &self.table
    }

    fn tables_mut(&mut self) -> &mut Tabler<T> {
        &mut self.table
    }
}

impl<T> Slr<T>
where
    T: PartialEq + Ord + Clone + Debug,
{
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.table.states.push(start);
    }

    #[must_use]
    pub fn closure(&self, state: State<T>) -> State<T> {
        let mut new_state = State::new();
        for pos in &state {
            if let Some(top) = pos.top() {
                if self.table.grammar.is_terminal(&top) {
                    continue;
                }
                for prod in self.table.grammar.rules[&top].prods() {
                    new_state.insert(Position::new(top.clone(), prod.clone(), 0, Set::new()));
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
    pub fn decision(&self, start: &T, pos: &Position<T>, row: &State<T>) -> Map<T, Action<T>> {
        pos.top().map_or_else(
            || {
                self.table.follow[&pos.rule]
                    .iter()
                    .map(|l| {
                        (
                            <T>::clone(l),
                            if &pos.rule == start {
                                Action::Acc
                            } else {
                                Action::Reduce(pos.rule.clone(), pos.seq.clone())
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

    /// # Panics
    /// Never.
    pub fn proc_actions(&mut self) {
        self.proc_closures();
        let start = self.table.basis_pos().rule;
        for row in &self.table.states {
            let mut map: Map<T, Action<T>> = Map::new();
            for item in row {
                for (term, act) in self.decision(&start, item, row) {
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
    pub fn prop_closure(&self, seed: State<T>) -> State<T> {
        transitive(seed, |s| self.closure(s))
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, to_tokens, Parser, Slr};

    #[test]
    pub fn dragon_book() {
        let slr = Slr::new(grammars_tests::dragon_book());
        assert_eq!(0, slr.tables().conflicts().count());

        for input in grammars_tests::DRAGON_BOOK_INPUTS {
            assert!(slr.validate(to_tokens(input.into_iter().cloned())));
        }
    }

    #[test]
    fn wikipedia() {
        let slr = Slr::new(grammars_tests::wikipedia());
        assert_eq!(0, slr.tables().conflicts().count());

        for input in grammars_tests::WIKIPEDIA_INPUTS {
            assert!(slr.validate(to_tokens(input.into_iter().cloned())));
        }
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    fn ucalgary_uni_oth_lr1() {
        let slr = Slr::new(grammars_tests::ucalgary_uni_oth_lr1());
        assert_eq!(2, slr.tables().conflicts().count());

        for input in grammars_tests::UCALGARY_UNI_OTH_LR1_INPUTS {
            assert_eq!(
                slr.validate(to_tokens(input.into_iter().cloned())),
                !grammars_tests::NON_LALR_UCALGARY_UNI_OTH_LR1_INPUTS.contains(input)
            );
        }
    }

    #[test]
    fn serokell() {
        let slr = Slr::new(grammars_tests::serokell());
        assert_eq!(0, slr.tables().conflicts().count());

        for input in grammars_tests::SEROKELL_INPUTS {
            assert!(slr.validate(to_tokens(input.into_iter().cloned())));
        }
    }

    #[test]
    pub fn puncs() {
        let slr = Slr::new(grammars_tests::puncs());
        assert_eq!(0, slr.tables().conflicts().count());

        for input in grammars_tests::PUNCS_INPUTS {
            assert!(slr.validate(to_tokens(input.into_iter().cloned())));
        }
    }

    #[test]
    pub fn scanner() {
        let slr = Slr::new(grammars_tests::scanner());
        assert_eq!(0, slr.tables().conflicts().count());

        for input in grammars_tests::SCANNER_INPUTS {
            assert!(slr.validate(to_tokens(input.into_iter().cloned())));
        }
    }
}
