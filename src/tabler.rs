use crate::{
    transitive, ActTable, Action, Grammar, Map, Position, Set, State, Table, Term, EOF,
    INTERNAL_START_RULE,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
                            entry.extend(self.first[prod[term_idx + 1]].clone());
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
                    table
                        .get_mut(name)
                        .unwrap()
                        .extend(input.get(first).unwrap_or_else(|| {
                            panic!("{name} was not listed as terminal or non-terminal")
                        }));
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
    pub fn basis_pos(&self) -> Position {
        let prod = &self.grammar.rules[INTERNAL_START_RULE].prods[0];
        Position::new(INTERNAL_START_RULE, prod.clone(), 0, Set::from([EOF]))
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

    #[must_use]
    pub fn sym_filter(state: &State, sym: &Term) -> State {
        state
            .iter()
            .filter(|p| p.top() == Some(sym))
            .filter_map(Position::clone_next)
            .collect()
    }

    pub fn conflicts(&self) -> impl Iterator<Item = &Action> + '_ {
        self.actions
            .iter()
            .flat_map(Map::values)
            .filter(|a| matches!(a, Action::Conflict(..)))
    }

    pub fn reduce_equals(&mut self) {
        let (travel, new_actions) = self.reduced_actions();
        self.actions = new_actions;

        self.actions
            .iter_mut()
            .flat_map(Map::iter_mut)
            .for_each(|(_, e)| *e = Self::update_entry(e, &travel));
    }

    /// Updates an action by re-indexing states from `travel`.
    #[must_use]
    pub fn update_entry(entry: &Action, travel: &Map<usize, usize>) -> Action {
        match entry {
            Action::Acc | Action::Reduce(..) => entry.clone(),
            Action::Goto(n) => Action::Goto(travel[n]),
            Action::Shift(n) => Action::Shift(travel[n]),
            // TODO: Reuse old allocations from `a` and `b`
            Action::Conflict(a, b) => Action::Conflict(
                Self::update_entry(a, travel).into(),
                Self::update_entry(b, travel).into(),
            ),
        }
    }

    /// Generates a map containing the update references (old state idx - new state idx) and a
    /// reduced action table where equal states was merged.
    /// Warranted to be O(n) and `actions.len() <= self.states`
    #[must_use]
    pub fn reduced_actions(&self) -> (Map<usize, usize>, ActTable) {
        let mut actions_map = Map::new();
        let mut travel_idx = Map::new();
        let mut actions = Vec::new();
        for (i, old_action) in self.actions.iter().enumerate() {
            if !actions_map.contains_key(old_action) {
                actions_map.insert(old_action.clone(), actions.len());
                actions.push(old_action.clone());
            }
            travel_idx.insert(i, actions_map[old_action]);
        }
        debug_assert!(actions.len() <= self.states.len());
        (travel_idx, actions)
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, Map, Set, Tabler, INTERNAL_START_RULE};

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

    #[test]
    pub fn scanner() {
        let table = Tabler::new(grammars_tests::scanner());
        assert_eq!(
            table.first,
            Map::from([
                (
                    "Alpha",
                    Set::from([
                        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
                        "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"
                    ]),
                ),
                (
                    "Digit",
                    Set::from(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]),
                ),
                (
                    "Item",
                    Set::from([
                        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e",
                        "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t",
                        "u", "v", "w", "x", "y", "z"
                    ]),
                ),
                (
                    "LRP'START",
                    Set::from([
                        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e",
                        "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t",
                        "u", "v", "w", "x", "y", "z"
                    ]),
                ),
                (
                    "Num",
                    Set::from(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]),
                ),
                (
                    "Phrase",
                    Set::from([
                        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e",
                        "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t",
                        "u", "v", "w", "x", "y", "z"
                    ]),
                ),
                ("Space", Set::from(["_"])),
                (
                    "Word",
                    Set::from([
                        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
                        "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"
                    ])
                )
            ])
        );

        assert_eq!(
            table.follow,
            Map::from([
                (
                    "Alpha",
                    Set::from([
                        "\u{3}", "_", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l",
                        "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"
                    ])
                ),
                (
                    "Digit",
                    Set::from(["\u{3}", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "_"])
                ),
                ("Item", Set::from(["\u{3}", "_"])),
                ("LRP'START", Set::from(["\u{3}"])),
                ("Num", Set::from(["\u{3}", "_"])),
                ("Phrase", Set::from(["\u{3}"])),
                (
                    "Space",
                    Set::from([
                        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e",
                        "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t",
                        "u", "v", "w", "x", "y", "z"
                    ])
                ),
                ("Word", Set::from(["\u{3}", "_"]))
            ])
        );
    }
}
