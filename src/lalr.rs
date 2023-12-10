use crate::{transitive, Action, Map, Parser, Position, Set, State, Tabler};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lalr<T>
where
    T: PartialEq + Ord + Clone + Debug,
{
    pub table: Tabler<T>,
    /// Describes the "raw states" (states without lookahead symbol) in `table.kernels`, using it
    /// raw kernel as key and value being its final kernel
    pub raws: Map<State<T>, State<T>>,
}

impl<T> Parser<T> for Lalr<T>
where
    T: PartialEq + Ord + Clone + Debug,
{
    fn new(grammar: crate::Grammar<T>) -> Self
    where
        Self: Sized,
    {
        Self::with_table(Tabler::new(grammar))
    }

    fn with_table(table: Tabler<T>) -> Self {
        let mut parser = Self::uninit(table);
        parser.proc_actions();
        parser
    }

    fn uninit(table: Tabler<T>) -> Self {
        Self {
            table,
            raws: Map::new(),
        }
    }

    fn tables(&self) -> &Tabler<T> {
        &self.table
    }

    fn tables_mut(&mut self) -> &mut Tabler<T> {
        &mut self.table
    }

    fn final_kernel<'a>(&'a self, kernel: &'a State<T>) -> Option<&'a State<T>> {
        let raw = Self::without_look(kernel);
        self.raws.get(&raw)
    }
}

impl<T> Lalr<T>
where
    T: PartialEq + Ord + Clone + Debug,
{
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.raws.insert(State::new(), start.clone());
        self.table.states.push(start);
    }

    #[must_use]
    pub fn without_look(state: &State<T>) -> State<T> {
        state.iter().map(Position::no_look).collect()
    }

    #[must_use]
    pub fn prop_closure(&self, seed: State<T>) -> State<T> {
        transitive(seed, |s| self.closure(s))
    }

    fn proc_actions(&mut self) {
        self.proc_closures();
        // TODO: Check impl
        // let mut kers: Vec<_> = self.table.kernels.iter().collect();
        // kers.sort_by_key(|(_, i)| *i);
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
    fn closure(&self, state: State<T>) -> State<T> {
        let mut new_state = State::new();
        for pos in &state {
            if let Some(top) = pos.top() {
                if self.table.grammar.is_terminal(&top) {
                    continue;
                }
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
    pub fn update_closures(
        &mut self,
        main: &State<T>,
        inc: State<T>,
        closures: State<T>,
    ) -> Option<State<T>> {
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
    pub fn update_state(
        &mut self,
        main: &State<T>,
        inc: State<T>,
        closures: State<T>,
    ) -> Option<State<T>> {
        let state_id = *self.table.kernels.get(main)?;

        // By definition, there will be always more closure items than kernels (because
        // essencially, every kernel can become a reduction closure or more than one shifting).
        // So it's better to check kernel equality first.
        let both_kernels = inc.into_iter().chain(main.clone()).collect();
        let new_kernel = Self::merged(both_kernels);

        if &new_kernel == main {
            None?;
        }

        let both_closures = self.table.states[state_id]
            .clone()
            .into_iter()
            .chain(closures)
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
    fn goto(&self, kernels: &State<T>, sym: &T) -> Option<(State<T>, State<T>)> {
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
    fn decision(&self, start: &T, pos: &Position<T>, row: &State<T>) -> Map<T, Action<T>> {
        pos.top().map_or_else(
            || {
                pos.look
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
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, to_tokens, Lalr, Parser};

    #[test]
    pub fn dragon_book() {
        let lalr = Lalr::new(grammars_tests::dragon_book());
        assert_eq!(0, lalr.tables().conflicts().count());

        for input in grammars_tests::DRAGON_BOOK_INPUTS {
            assert!(lalr.validate(to_tokens(input.iter().cloned())));
        }
    }

    #[test]
    fn wikipedia() {
        let lalr = Lalr::new(grammars_tests::wikipedia());
        assert_eq!(0, lalr.tables().conflicts().count());

        for input in grammars_tests::WIKIPEDIA_INPUTS {
            assert!(lalr.validate(to_tokens(input.iter().cloned())));
        }
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    fn ucalgary_uni_oth_lr1() {
        let lalr = Lalr::new(grammars_tests::ucalgary_uni_oth_lr1());
        assert_eq!(2, lalr.tables().conflicts().count());

        for input in grammars_tests::UCALGARY_UNI_OTH_LR1_INPUTS {
            assert_eq!(
                lalr.validate(to_tokens(input.iter().cloned())),
                !grammars_tests::NON_LALR_UCALGARY_UNI_OTH_LR1_INPUTS.contains(input)
            );
        }
    }

    #[test]
    fn serokell() {
        let lalr = Lalr::new(grammars_tests::serokell());
        assert_eq!(0, lalr.tables().conflicts().count());

        for input in grammars_tests::SEROKELL_INPUTS {
            assert!(lalr.validate(to_tokens(input.iter().cloned())));
        }
    }

    #[test]
    pub fn puncs() {
        let lalr = Lalr::new(grammars_tests::puncs());
        assert_eq!(0, lalr.tables().conflicts().count());

        for input in grammars_tests::PUNCS_INPUTS {
            assert!(lalr.validate(to_tokens(input.iter().cloned())));
        }
    }

    #[test]
    pub fn scanner() {
        let lalr = Lalr::new(grammars_tests::scanner());
        assert_eq!(0, lalr.tables().conflicts().count());

        for input in grammars_tests::SCANNER_INPUTS {
            assert!(lalr.validate(to_tokens(input.iter().cloned())));
        }
    }
}
