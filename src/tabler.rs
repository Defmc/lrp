use crate::{
    transitive, ActTable, Action, Grammar, Map, Position, Rule, Set, State, Table, Term, TermSet,
};

#[derive(Debug, Default)]
pub struct Tabler {
    pub grammar: Grammar,
    pub terminals: TermSet,
    pub first: Table,
    pub follow: Table,
    pub actions: ActTable,
    pub syms: TermSet,
    pub states: Vec<State>,
    pub kernels: Vec<(State, usize)>,
}

impl Tabler {
    #[must_use]
    pub fn new(grammar: Grammar, terminals: TermSet) -> Self {
        let syms = grammar
            .keys()
            .chain(terminals.clone().iter().filter(|&&c| c != "$"))
            .copied()
            .collect();

        let mut buf = Self {
            grammar,
            terminals,
            syms,
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
        for (name, rules) in &self.grammar {
            table.insert(name, Set::new());
            for rule in rules.iter().filter(|r| &r[0] != name) {
                table.get_mut(name).unwrap().insert(rule[0]);
            }
        }
        table
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn gen_follow(&self) -> Table {
        let mut table = Table::new();
        for (name, rules) in &self.grammar {
            for rule in rules {
                for term_idx in 0..rule.len() - 1 {
                    // A = . . . A a -> {A: FIRST(A)} -> {A: A} -> {}
                    if !self.is_terminal(rule[term_idx]) {
                        let entry = table.entry(rule[term_idx]).or_insert_with(Set::new);
                        if self.is_terminal(rule[term_idx + 1]) {
                            // A = . . . T a -> {T: a}
                            entry.insert(rule[term_idx + 1]);
                        } else {
                            // A = . . . T B -> {T: FIRST(B)}
                            entry.extend(self.first[rule[term_idx + 1]].clone())
                        }
                    }
                }
                let last = rule.last().unwrap();
                // A = . . . . T -> {T: FOLLOW(A)}
                // But if A = . . . . A -> {A: FOLLOW(A)} -> {A: A} -> {}
                if !self.is_terminal(last) && last != name {
                    table.entry(last).or_insert_with(Set::new).insert(name);
                }
            }
        }
        table.insert("S", Set::from(["$"]));
        table
    }

    pub fn proc_first(&mut self) {
        self.first = transitive(self.first.clone(), |t| self.first_step(&t));
        // FIRST must be a subset of TERMINALS
        debug_assert!(self.first.values().flatten().all(|t| self.is_terminal(t)));
    }

    pub fn proc_follow(&mut self) {
        self.follow = transitive(self.follow.clone(), |t| self.follow_step(&t));
        // FOLLOW must be a subset of TERMINALS
        debug_assert!(self.follow.values().flatten().all(|t| self.is_terminal(t)));
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn first_step(&self, input: &Table) -> Table {
        let mut table = Table::new();
        for (name, firsts) in input {
            table.insert(name, Set::new());
            for first in firsts {
                if self.is_terminal(first) {
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
                if self.is_terminal(term) {
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
    pub fn is_terminal(&self, term: &str) -> bool {
        self.terminals.contains(term)
    }

    #[must_use]
    pub fn pos<'a>(
        &'a self,
        rule: Rule,
        pos: usize,
        look: Set<Term>,
    ) -> impl Iterator<Item = Position> + 'a {
        self.grammar[rule]
            .iter()
            .map(move |s| Position::new(rule, s.clone(), pos, look.clone()))
    }

    #[must_use]
    pub fn closure(&self, state: State) -> State {
        let mut new_state = State::new();
        for pos in &state {
            if let Some(top) = pos.top() {
                if self.is_terminal(top) {
                    continue;
                }
                let look = if let Some(locus) = pos.locus() {
                    if self.is_terminal(locus) {
                        Set::from([locus])
                    } else {
                        self.first_of(&pos.next_and_look()).clone()
                    }
                } else {
                    self.follow[top].clone()
                };
                for prod in &self.grammar[top] {
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

    pub fn proc_closures(&mut self, start: Position) {
        self.proc_closures_first_row(start.clone());
        let mut idx = 0;
        while idx < self.states.len() {
            let row = self.states[idx].clone();
            for s in &self.syms {
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

    pub fn proc_closures_first_row(&mut self, start: Position) {
        let start = self.prop_closure(State::from([start]));
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

    pub fn proc_actions(&mut self, start: Rule) {
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
            if self.is_terminal(locus) {
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
    use crate::{Dfa, Item, Map, Position, Set, StackEl, Tabler};
    use pretty_assertions::assert_eq;

    #[test]
    pub fn dragon_book() {
        let grammar = crate::grammar! {
            "S" -> "C" "C",
            "C" -> "c" "C"
                | "d"
        };
        let mut tabler = Tabler::new(grammar, Set::from(["c", "d", "$"]));
        assert_eq!(
            tabler.first,
            Map::from([("S", Set::from(["c", "d"])), ("C", Set::from(["c", "d"]))])
        );
        tabler.proc_closures(Position::new("S", vec!["C", "C"], 0, Set::from(["$"])));
        tabler.proc_actions("S");

        let mut dfa = Dfa::new(["c", "d", "d", "$"].into_iter(), tabler.actions);
        dfa.start();

        assert_eq!(
            dfa.stack,
            vec![
                StackEl::State(0),
                StackEl::Item(Item::Compound(
                    "C",
                    vec![
                        Item::Compound("C", vec![Item::Simple("d")]),
                        Item::Simple("c"),
                    ]
                )),
                StackEl::State(1),
                StackEl::Item(Item::Compound("C", vec![Item::Simple("d")])),
                StackEl::State(4),
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

        let mut tabler = Tabler::new(grammar, Set::from(["0", "1", "+", "*", "$"]));
        assert_eq!(
            tabler.first,
            Map::from([
                ("S", Set::from(["0", "1"])),
                ("E", Set::from(["0", "1"])),
                ("B", Set::from(["0", "1"]))
            ])
        );

        tabler.proc_closures(Position::new("S", vec!["E"], 0, Set::from(["$"])));
        tabler.proc_actions("S");

        let mut dfa = Dfa::new(["1", "+", "1", "$"].into_iter(), tabler.actions);
        dfa.start();

        assert_eq!(
            dfa.stack,
            vec![
                StackEl::State(0),
                StackEl::Item(Item::Compound(
                    "E",
                    vec![
                        Item::Compound("B", vec![Item::Simple("1")]),
                        Item::Simple("+"),
                        Item::Compound("E", vec![Item::Compound("B", vec![Item::Simple("1")])]),
                    ]
                )),
                StackEl::State(4,),
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

        let mut tabler = Tabler::new(grammar, Set::from(["a", "b", "c", "d", "e", "$"]));
        assert_eq!(
            tabler.first,
            Map::from([
                ("A", Set::from(["a",])),
                ("B", Set::from(["a",])),
                ("C", Set::from(["d", "e",])),
                ("D", Set::from(["d", "e",])),
                ("E", Set::from(["d", "e",])),
                ("F", Set::from(["d", "e",])),
                ("S", Set::from(["d", "e",])),
            ])
        );

        tabler.proc_closures(Position::new("S", vec!["E"], 0, Set::from(["$"])));
        tabler.proc_actions("S");

        let mut dfa = Dfa::new(["e", "a", "c", "$"].into_iter(), tabler.actions);
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

        let mut tabler = Tabler::new(
            "Start",
            grammar,
            Set::from(["int", "ident", "(", ")", "+", "*"]),
        );
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

        let mut dfa = Dfa::new(
            ["int", "+", "ident", "*", "ident", "+", "int"].into_iter(),
            tabler.actions,
        );
        //dfa.start();
    }
}
