use crate::{
    transitive, ActTable, Action, Grammar, Map, Position, Rule, Set, State, Table, Term, EOF,
    INTERNAL_START_RULE,
};

#[derive(Debug, Default)]
pub struct Tabler {
    pub grammar: Grammar,
    pub first: Table,
    pub follow: Table,
    pub actions: ActTable,
    pub states: Vec<State>,
    pub kernels: Map<State, usize>,
}

impl Tabler {
    #[must_use]
    pub fn new(grammar: Grammar) -> Self {
        let mut buf = Self {
            grammar,
            ..Default::default()
        };
        buf.first = buf.gen_first();
        buf.proc_first();
        buf.follow = buf.gen_follow();
        buf.proc_follow();
        buf
    }

    /// Generates the first FIRST set iteration for the given grammar.
    /// # Panics
    /// Never.
    #[must_use]
    pub fn gen_first(&self) -> Table {
        let mut table = Table::new();
        for rule in self.grammar.rules() {
            table.insert(rule.name, Set::new());
            for prod in rule.prods.iter().filter(|r| r[0] != rule.name) {
                table.get_mut(rule.name).unwrap().insert(prod[0]);
            }
        }
        table
    }

    /// Generates the first FOLLOW set iteration for the given grammar.
    /// # Panics
    /// Never.
    #[must_use]
    pub fn gen_follow(&self) -> Table {
        let mut table = Table::new();
        table.insert(INTERNAL_START_RULE, Set::from([EOF]));
        for rule in self.grammar.rules() {
            for prod in rule.prods() {
                for term_idx in 0..prod.len() - 1 {
                    // A = . . . A a -> {A: FIRST(A)} -> {A: A} -> {}
                    if !self.grammar.is_terminal(&prod[term_idx]) {
                        let entry = table.entry(prod[term_idx]).or_insert_with(Set::new);
                        if self.grammar.is_terminal(&prod[term_idx + 1]) {
                            // A = . . . T a -> {T: a}
                            entry.insert(prod[term_idx + 1]);
                        } else {
                            // A = . . . T B -> {T: FIRST(B)}
                            entry.extend(self.first[prod[term_idx + 1]].clone())
                        }
                    }
                }
                let last = prod.last().unwrap();
                // A = . . . . T -> {T: FOLLOW(A)}
                // But if A = . . . . A -> {A: FOLLOW(A)} -> {A: A} -> {}
                if !self.grammar.is_terminal(last) && last != &rule.name {
                    table.entry(last).or_insert_with(Set::new).insert(rule.name);
                }
            }
        }
        table
    }

    pub fn proc_first(&mut self) {
        self.first = transitive(self.first.clone(), |t| self.first_step(&t));
        // FIRST must be a subset of TERMINALS
        debug_assert!(self
            .first
            .values()
            .flatten()
            .all(|t| self.grammar.is_terminal(t)));
    }

    pub fn proc_follow(&mut self) {
        self.follow = transitive(self.follow.clone(), |t| self.follow_step(&t));
        // FOLLOW must be a subset of TERMINALS
        debug_assert!(self
            .follow
            .values()
            .flatten()
            .all(|t| self.grammar.is_terminal(t)));
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn first_step(&self, input: &Table) -> Table {
        let mut table = Table::new();
        for (name, firsts) in input {
            table.insert(name, Set::new());
            for first in firsts {
                if self.grammar.is_terminal(first) {
                    table.get_mut(name).unwrap().insert(first);
                } else {
                    table.get_mut(name).unwrap().extend(&input[first]);
                }
            }
            if table[name].contains(name) {
                table.get_mut(name).unwrap().remove(name);
            }
        }
        table
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn follow_step(&self, input: &Table) -> Table {
        let mut table = Table::new();
        for (noterm, terms) in input {
            table.insert(noterm, Set::new());
            for term in terms {
                if self.grammar.is_terminal(term) {
                    table.get_mut(noterm).unwrap().insert(term);
                } else if let Some(entry) = input.get(term) {
                    table.get_mut(noterm).unwrap().extend(entry);
                }
            }
            if table[noterm].contains(noterm) {
                table.get_mut(noterm).unwrap().remove(noterm);
            }
        }
        table
    }

    #[must_use]
    pub fn closure(&self, state: State) -> State {
        let mut new_state = State::new();
        for pos in &state {
            if let Some(top) = pos.top() {
                if self.grammar.is_terminal(&top) {
                    continue;
                }
                let look = if let Some(locus) = pos.locus() {
                    self.first_of(&Set::from([locus])).clone()
                } else {
                    pos.look.clone()
                };
                for prod in self.grammar.rules[top].prods() {
                    new_state.insert(Position::new(top, prod.clone(), 0, look.clone()));
                }
            }
        }
        new_state.extend(state);
        new_state
    }

    #[must_use]
    pub fn prop_closure(&self, state: State) -> State {
        Self::merged(transitive(state, |s| self.closure(s)))
    }

    #[must_use]
    pub fn basis_pos(&self) -> Position {
        let prod = &self.grammar.rules[INTERNAL_START_RULE].prods[0];
        Position::new(INTERNAL_START_RULE, prod.clone(), 0, Set::from([EOF]))
    }

    pub fn proc_closures(&mut self) {
        self.proc_closures_first_row();
        let mut idx = 0;
        while idx < self.states.len() {
            let row = self.states[idx].clone();
            for s in self.grammar.symbols() {
                let (kernel, closures) = if let Some((k, c)) = self.goto(row.clone(), &s) {
                    (k, c)
                } else {
                    continue;
                };
                self.kernels.insert(kernel, self.states.len());
                self.states.push(closures);
            }
            idx += 1;
        }
    }

    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.basis_pos()]));
        self.kernels.insert(State::new(), 0);
        self.states.push(start.clone());
    }

    #[must_use]
    pub fn goto(&self, kernels: State, sym: &Term) -> Option<(State, State)> {
        let kernels = Self::sym_filter(&kernels, sym);
        if self.kernels.contains_key(&kernels) {
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
    pub fn decision(&self, start: Rule, pos: &Position, row: &State) -> Map<Term, Action> {
        if let Some(locus) = pos.top() {
            let filter = Self::sym_filter(row, &locus);
            let state = self
                .kernels
                .iter()
                .find_map(|(k, s)| if k == &filter { Some(*s) } else { None })
                .expect("`kernels` is incomplete");
            if self.grammar.is_terminal(&locus) {
                Map::from([(locus, Action::Shift(state))])
            } else {
                Map::from([(locus, Action::Goto(state))])
            }
        } else {
            pos.look
                .iter()
                .map(|l| {
                    (
                        l.clone(),
                        if pos.rule == start {
                            Action::Acc
                        } else {
                            Action::Reduce(pos.rule, pos.seq.clone())
                        },
                    )
                })
                .collect()
        }
    }

    #[must_use]
    pub fn sym_filter(state: &State, sym: &Term) -> State {
        state
            .into_iter()
            .filter(|p| p.top() == Some(&sym))
            .filter_map(|p| p.clone_next())
            .collect()
    }

    #[must_use]
    pub fn merged(states: State) -> State {
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

    #[must_use]
    pub fn first_of(&self, items: &Set<Term>) -> Set<Term> {
        let mut firsts = Set::new();
        for item in items {
            if let Some(first) = self.first.get(item) {
                firsts.extend(first.clone());
            } else {
                firsts.insert(*item);
            };
        }
        firsts
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, Map, Set, Tabler, INTERNAL_START_RULE};
    use pretty_assertions::assert_eq;

    #[test]
    pub fn dragon_book() {
        let table = Tabler::new(grammars_tests::dragon_book());

        assert_eq!(
            table.first,
            Map::from([
                ("S", Set::from(["c", "d"])),
                ("C", Set::from(["c", "d"])),
                (INTERNAL_START_RULE, Set::from(["c", "d"]))
            ])
        );

        assert_eq!(
            table.follow,
            Map::from([
                ("C", Set::from(["\u{3}", "c", "d"])),
                ("S", Set::from(["\u{3}"])),
                (INTERNAL_START_RULE, Set::from(["\u{3}"])),
            ])
        );
    }

    #[test]
    fn wikipedia() {
        let table = Tabler::new(grammars_tests::wikipedia());

        assert_eq!(
            table.first,
            Map::from([
                ("S", Set::from(["0", "1"])),
                ("E", Set::from(["0", "1"])),
                ("B", Set::from(["0", "1"])),
                (INTERNAL_START_RULE, Set::from(["0", "1"]))
            ])
        );

        assert_eq!(
            table.follow,
            Map::from([
                ("B", Set::from(["\u{3}", "*", "+"])),
                ("E", Set::from(["\u{3}", "*", "+"])),
                ("S", Set::from(["\u{3}"])),
                (INTERNAL_START_RULE, Set::from(["\u{3}"])),
            ])
        );
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    fn ucalgary_uni_oth_lr1() {
        let table = Tabler::new(grammars_tests::ucalgary_uni_oth_lr1());

        assert_eq!(
            table.first,
            Map::from([
                ("A", Set::from(["a"])),
                ("B", Set::from(["a"])),
                ("C", Set::from(["d", "e"])),
                ("D", Set::from(["d", "e"])),
                ("E", Set::from(["d", "e"])),
                ("F", Set::from(["d", "e"])),
                ("S", Set::from(["d", "e"])),
                (INTERNAL_START_RULE, Set::from(["d", "e"]))
            ])
        );

        assert_eq!(
            table.follow,
            Map::from([
                ("A", Set::from(["b", "c"])),
                ("B", Set::from(["b", "c"])),
                ("C", Set::from(["\u{3}"])),
                ("D", Set::from(["\u{3}"])),
                ("E", Set::from(["\u{3}"])),
                ("F", Set::from(["\u{3}"])),
                ("S", Set::from(["\u{3}"])),
                (INTERNAL_START_RULE, Set::from(["\u{3}"])),
            ])
        );
    }

    #[test]
    fn serokell() {
        let table = Tabler::new(grammars_tests::serokell());

        assert_eq!(
            table.first,
            Map::from([
                ("Add", Set::from(["(", "ident", "int"])),
                ("Factor", Set::from(["(", "ident", "int"])),
                ("Start", Set::from(["(", "ident", "int"])),
                ("Term", Set::from(["(", "ident", "int"])),
                (INTERNAL_START_RULE, Set::from(["(", "ident", "int"])),
            ])
        );

        assert_eq!(
            table.follow,
            Map::from([
                ("Add", Set::from(["\u{3}", ")", "+"])),
                ("Factor", Set::from(["\u{3}", ")", "*", "+"])),
                ("Term", Set::from(["\u{3}", ")", "*", "+"])),
                ("Start", Set::from(["\u{3}"])),
                (INTERNAL_START_RULE, Set::from(["\u{3}"])),
            ])
        );
    }

    #[test]
    pub fn puncs() {
        let table = Tabler::new(grammars_tests::puncs());

        assert_eq!(
            table.first,
            Map::from([
                ("LRP'START", Set::from(["(", "[", "{",])),
                ("S", Set::from(["(", "[", "{",])),
            ])
        );

        assert_eq!(
            table.follow,
            Map::from([
                ("LRP'START", Set::from(["\u{3}",])),
                ("S", Set::from(["\u{3}", ")", "]", "}",])),
            ])
        );
    }
}
