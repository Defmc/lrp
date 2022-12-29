use crate::{transitive, ActTable, Action, Grammar, Map, Position, Rule, Set, State, Table, Term};

#[derive(Debug, Default)]
pub struct Tabler {
    pub grammar: Grammar,
    pub first: Table,
    pub follow: Table,
    pub actions: ActTable,
    pub states: Vec<State>,
    pub kernels: Vec<(State, usize)>,
}

impl Tabler {
    #[must_use]
    pub fn new(grammar: Grammar) -> Self {
        let mut buf = Self {
            grammar,
            ..Default::default()
        };
        buf.first = dbg!(buf.gen_first());
        buf.proc_first();
        buf.follow = dbg!(buf.gen_follow());
        buf.proc_follow();
        buf
    }

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

    /// # Panics
    /// Never.
    #[must_use]
    pub fn gen_follow(&self) -> Table {
        let mut table = Table::new();
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
        table.insert(crate::INTERNAL_START_RULE, Set::from([crate::EOF]));
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
                    println!("{first}");
                    table
                        .get_mut(name)
                        .unwrap() // a
                        .extend(&input[first]);
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
    pub fn pos<'a>(
        &'a self,
        rule: Rule,
        pos: usize,
        look: Set<Term>,
    ) -> impl Iterator<Item = Position> + 'a {
        self.grammar.rules[rule]
            .prods
            .iter()
            .map(move |s| Position::new(rule, s.clone(), pos, look.clone()))
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
                    if self.grammar.is_terminal(&locus) {
                        Set::from([locus])
                    } else {
                        self.first_of(&pos.next_and_look()).clone()
                    }
                } else {
                    self.follow[top].clone()
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
        let prod = &self.grammar.rules[crate::INTERNAL_START_RULE].prods[0];
        Position::new(
            crate::INTERNAL_START_RULE,
            prod.clone(),
            0,
            Set::from([crate::EOF]),
        )
    }

    pub fn proc_closures(&mut self) {
        self.proc_closures_first_row();
        let mut idx = 0;
        while idx < self.states.len() {
            let row = self.states[idx].clone();
            for s in self.grammar.symbols() {
                println!("\ngoto({idx}, {s})");
                let (kernel, closures) = if let Some((k, c)) = self.goto(row.clone(), &s) {
                    (k, c)
                } else {
                    continue;
                };
                self.kernels.push((kernel, self.states.len()));
                self.states.push(closures);
            }
            idx += 1;
        }
    }

    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.basis_pos()]));
        self.kernels.push((State::new(), 0));
        self.states.push(start.clone());
    }

    #[must_use]
    pub fn goto(&self, kernels: State, sym: &Term) -> Option<(State, State)> {
        println!("entries for {sym}:");
        for kernel in &kernels {
            println!("\t{kernel}");
        }
        let kernels: State = kernels
            .into_iter()
            .filter(|p| p.top() == Some(&sym))
            .filter_map(|p| p.clone_next())
            .collect();
        println!("goto's:");
        for goto in &kernels {
            println!("\t{goto}");
        }
        if let Some(state) =
            self.kernels
                .iter()
                .find_map(|(k, s)| if k == &kernels { Some(s) } else { None })
        {
            println!("repeated (state {state})");
            None?;
        }
        let new = self.prop_closure(kernels.clone());
        println!("closures");
        for closure in &new {
            println!("\t{closure}");
        }
        if new.is_empty() {
            None
        } else {
            Some((kernels, new))
        }
    }

    pub fn proc_actions(&mut self) {
        let start = self.basis_pos().rule;
        for (i, row) in self.states.iter().enumerate() {
            println!("building [{i}]");
            let mut map: Map<Term, Action> = Map::new();
            for item in row {
                for (term, act) in self.decision(start, item) {
                    if map.contains_key(term) && map[term] != act {
                        println!("CONFLICT [{i}:{term}]: {:?} and {act:?}", map[term]);
                        *map.get_mut(term).unwrap() =
                            Action::Conflict(Box::new(map.get(term).unwrap().clone()), act.into());
                    } else {
                        map.insert(term, act);
                    }
                }
            }
            self.actions.push(map);
        }
    }

    #[must_use]
    pub fn decision(&self, start: Rule, pos: &Position) -> Map<Term, Action> {
        if let Some(locus) = pos.top() {
            println!("{pos}");
            let next = pos.clone_next().unwrap();
            let state = self
                .kernels
                .iter()
                .find_map(|(k, s)| if k.contains(&next) { Some(*s) } else { None })
                .expect("`kernels` is incomplete");
            if self.grammar.is_terminal(&locus) {
                println!("shift({state})");
                Map::from([(locus, Action::Shift(state))])
            } else {
                println!("goto({state})");
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
                            Action::Reduce(pos.clone())
                        },
                    )
                })
                .collect()
        }
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

    pub fn print_states(&self) {
        for (kernel, i) in &self.kernels {
            println!("state {i}: {:?}", kernel);
            for closure in &self.states[*i] {
                println!("\t{closure}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Grammar, Dfa, Item, Map, Set, StackEl, Tabler, INTERNAL_START_RULE};
    use pretty_assertions::assert_eq;

    #[test]
    pub fn dragon_book() {
        let grammar = crate::grammar! {
            "S" -> "C" "C",
            "C" -> "c" "C"
                | "d"
        };
        let grammar = Grammar::new("S", grammar, Set::from(["c", "d"]));
        let mut tabler = Tabler::new(grammar);
        assert_eq!(
            tabler.first,
            Map::from([
                ("S", Set::from(["c", "d"])),
                ("C", Set::from(["c", "d"])),
                (INTERNAL_START_RULE, Set::from(["c", "d"]))
            ])
        );
        tabler.proc_closures();
        tabler.proc_actions();

        let mut dfa = Dfa::new(["c", "d", "d"].into_iter(), tabler.actions);
        dfa.start();

        assert_eq!(
            dfa.stack,
            vec![
                StackEl::State(0),
                StackEl::Item(Item::Compound(
                    "S",
                    vec![
                        Item::Compound("C", vec![Item::Simple("d")]),
                        Item::Compound(
                            "C",
                            vec![
                                Item::Compound("C", vec![Item::Simple("d")]),
                                Item::Simple("c"),
                            ],
                        ),
                    ],
                )),
                StackEl::State(2),
            ]
        );
    }

    #[test]
    fn wikipedia() {
        let grammar = crate::grammar! {
            "S" -> "E",
            "E" -> "E" "*" "B"
                | "E" "+" "B"
                | "B",
            "B" -> "0" | "1"
        };

        let grammar = Grammar::new("S", grammar, Set::from(["0", "1", "+", "*"]));
        let mut tabler = Tabler::new(grammar);
        assert_eq!(
            tabler.first,
            Map::from([
                ("S", Set::from(["0", "1"])),
                ("E", Set::from(["0", "1"])),
                ("B", Set::from(["0", "1"])),
                (INTERNAL_START_RULE, Set::from(["0", "1"]))
            ])
        );

        tabler.proc_closures();
        tabler.proc_actions();

        let mut dfa = Dfa::new(["1", "+", "1"].into_iter(), tabler.actions);
        dfa.start();

        assert_eq!(
            dfa.stack,
            vec![
                StackEl::State(0),
                StackEl::Item(Item::Compound(
                    "S",
                    vec![Item::Compound(
                        "E",
                        vec![
                            Item::Compound("B", vec![Item::Simple("1")]),
                            Item::Simple("+"),
                            Item::Compound("E", vec![Item::Compound("B", vec![Item::Simple("1")])]),
                        ],
                    )]
                )),
                StackEl::State(5),
            ]
        );
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    fn ucalgary_uni_oth_lr1() {
        let grammar = crate::grammar! {
            "S" -> "E",
            "E" ->	"d" "D"
                |	"D"
                |	"F",
            "F" ->	"e" "C"
                |	"C",
            "D" ->	"d" "e" "B" "b"
                |	"e" "A" "c",
            "C" ->	"e" "d" "B" "c"
                |	"d" "A" "b",
            "B" ->	"a",
            "A" ->	"a"
        };

        let grammar = Grammar::new("S", grammar, Set::from(["a", "b", "c", "d", "e"]));
        let mut tabler = Tabler::new(grammar);
        assert_eq!(
            tabler.first,
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

        tabler.proc_closures();
        tabler.proc_actions();

        let mut dfa = Dfa::new(["e", "a", "c"].into_iter(), tabler.actions);
        dfa.start();
    }

    #[test]
    fn serokell() {
        let grammar = crate::grammar! {
           "Start" -> "Add",
           "Add" -> "Add" "+" "Factor"
               | "Factor",
           "Factor" -> "Factor" "*" "Term"
               | "Term",
           "Term" -> "(" "Add" ")"
               | "int"
               | "ident"
        };

        let grammar = Grammar::new(
            "Start",
            grammar,
            Set::from(["int", "ident", "(", ")", "+", "*"]),
        );
        let mut tabler = Tabler::new(grammar);

        assert_eq!(
            tabler.first,
            Map::from([
                ("Add", Set::from(["(", "ident", "int"])),
                ("Factor", Set::from(["(", "ident", "int"])),
                ("LRP'START", Set::from(["(", "ident", "int"])),
                ("Start", Set::from(["(", "ident", "int"])),
                ("Term", Set::from(["(", "ident", "int"])),
            ])
        );

        tabler.proc_closures();
        tabler.proc_actions();
        tabler.print_states();

        let _dfa = Dfa::new(
            ["int", "+", "ident", "*", "ident", "+", "int"].into_iter(),
            tabler.actions,
        );
        //dfa.start();
    }
}
