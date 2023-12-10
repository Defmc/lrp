use crate::{transitive, Action, Map, Parser, Position, Set, State, Tabler};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Clr<T>
where
    T: PartialEq + PartialOrd + Ord + Clone + Debug,
{
    pub table: Tabler<T>,
}

impl<T> Parser<T> for Clr<T>
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
    T: PartialEq + Ord + Clone + Debug,
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
    pub fn closure(&self, state: State<T>) -> State<T> {
        let mut new_state = State::new();
        for pos in &state {
            if let Some(top) = pos.top() {
                if self.table.grammar.is_terminal(&top) {
                    continue;
                }
                // TODO: Remove `clone`
                let look = pos.next().map_or_else(
                    || pos.look.clone(),
                    |locus| self.table.first_of(&Set::from([locus])),
                );
                for prod in self.table.grammar.rules[&top].prods() {
                    new_state.insert(Position::new(top.clone(), prod.clone(), 0, look.clone()));
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
            for s in self.table.grammar.symbols() {
                let Some((kernel, closures)) = self.goto(&self.table.states[idx], &s) else {
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

    /// Maps the decisions for the passed position
    /// # Panics
    /// Never.
    #[must_use]
    pub fn decision(
        &self,
        entry_point: &T,
        pos: &Position<T>,
        row: &State<T>,
    ) -> Map<T, Action<T>> {
        pos.top().map_or_else(
            || {
                pos.look
                    .iter()
                    .map(|l| {
                        (
                            l.clone(),
                            if &pos.rule == entry_point {
                                Action::Acc
                            } else {
                                Action::Reduce(pos.rule.clone(), pos.seq.clone())
                            },
                        )
                    })
                    .collect()
            },
            |top| {
                let filter = Tabler::sym_filter(row, &top);
                let state = self
                    .state_from_kernel(&filter)
                    .expect("`kernels` is incomplete");
                if self.table.grammar.is_terminal(&top) {
                    Map::from([(top, Action::Shift(state))])
                } else {
                    Map::from([(top, Action::Goto(state))])
                }
            },
        )
    }

    /// Propagates a closure until there's no more states to generate from it
    /// Following the transitive definition: prop_closure(prop_closure(S)) = prop_closure(S), but
    /// S != prop_closure(S)
    #[must_use]
    pub fn prop_closure(&self, seed: State<T>) -> State<T> {
        transitive(seed, |s| self.closure(s))
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, to_tokens, Clr, Parser};

    #[test]
    pub fn dragon_book() {
        let clr = Clr::new(grammars_tests::dragon_book());
        assert_eq!(0, clr.tables().conflicts().count());

        for input in grammars_tests::DRAGON_BOOK_INPUTS {
            assert!(clr.validate(to_tokens(input.iter().cloned())));
        }
    }

    #[test]
    pub fn wikipedia() {
        let clr = Clr::new(grammars_tests::wikipedia());
        assert_eq!(0, clr.tables().conflicts().count());

        for input in grammars_tests::WIKIPEDIA_INPUTS {
            assert!(clr.validate(to_tokens(input.iter().cloned())));
        }
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    pub fn ucalgary_uni_oth_lr1() {
        let clr = Clr::new(grammars_tests::ucalgary_uni_oth_lr1());
        assert_eq!(0, clr.tables().conflicts().count());

        for input in grammars_tests::UCALGARY_UNI_OTH_LR1_INPUTS {
            assert!(clr.validate(to_tokens(input.iter().cloned())));
        }
    }

    #[test]
    pub fn serokell() {
        let clr = Clr::new(grammars_tests::serokell());
        assert_eq!(0, clr.tables().conflicts().count());

        for input in grammars_tests::SEROKELL_INPUTS {
            assert!(clr.validate(to_tokens(input.iter().cloned())));
        }
    }

    #[test]
    pub fn puncs() {
        let clr = Clr::new(grammars_tests::puncs());
        assert_eq!(0, clr.tables().conflicts().count());

        for input in grammars_tests::PUNCS_INPUTS {
            assert!(clr.validate(to_tokens(input.iter().cloned())));
        }
    }

    #[test]
    pub fn scanner() {
        let clr = Clr::new(grammars_tests::scanner());
        assert_eq!(0, clr.tables().conflicts().count());

        for input in grammars_tests::SCANNER_INPUTS {
            assert!(clr.validate(to_tokens(input.iter().cloned())));
        }
    }
}
