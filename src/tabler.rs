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
    pub pos_states: Map<Position, usize>,
    pub syms: TermSet,
    pub closures: Map<usize, State>,
}

impl Tabler {
    #[must_use]
    pub fn new(grammar: Grammar, terminals: TermSet) -> Self {
        let syms = grammar
            .keys()
            .chain(terminals.clone().iter())
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
                    if &rule[term_idx] == name {
                        continue;
                    }
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
        table
    }

    pub fn proc_first(&mut self) {
        self.first = transitive(self.first.clone(), |t| self.first_step(&t));
        // FIRST must be a subset of TERMINALS
        debug_assert!(self.first.values().flatten().all(|t| self.is_terminal(t)));
    }

    pub fn proc_follow(&mut self) {
        self.follow = transitive(self.follow.clone(), |t| self.follow_step(&t));
        self.follow.insert("S", Set::from(["$"]));
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
                    table.get_mut(name).unwrap().extend(&input[first]);
                }
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
                if !self.is_terminal(top) {
                    let look = if let Some(locus) = pos.locus() {
                        if self.is_terminal(locus) {
                            Set::from([locus])
                        } else {
                            self.first[locus].clone()
                        }
                    } else {
                        self.follow[&pos.rule].clone()
                    };
                    for prod in &self.grammar[top] {
                        new_state.insert(Position::new(top, prod.clone(), 0, look.clone()));
                    }
                }
            }
        }
        new_state.extend(state);
        new_state
    }

    #[must_use]
    pub fn prop_closure(&self, state: State) -> State {
        transitive(state, |s| self.closure(s))
    }

    pub fn proc_closures(&mut self, start: Position) {
        let syms: Vec<_> = self
            .syms
            .iter()
            .filter(|&&s| s != start.rule && s != "$")
            .collect();
        let start = self.prop_closure(State::from([start]));
        self.closures.insert(0, start.clone());
        start.into_iter().for_each(|c| {
            self.pos_states.insert(c, 0);
        });
        let mut idx = 0;
        let mut state_idx = 0;
        loop {
            if self.closures.get(&idx).is_some() {
                let closures = self.closures.get(&idx).unwrap();
                let gotos: Vec<State> = syms
                    .iter()
                    .map(|s| self.goto(closures.clone(), s))
                    .flatten()
                    .collect();
                for goto in gotos {
                    if goto.len() == 1 && self.pos_states.contains_key(goto.first().unwrap()) {
                        continue;
                    }
                    println!("out state {state_idx}");
                    state_idx += 1;
                    println!("in state {state_idx}");
                    self.closures.insert(state_idx, goto.clone());
                    for closure in goto {
                        self.pos_states.insert(closure, state_idx);
                    }
                }
            }
            if &idx == self.pos_states.last_key_value().unwrap().1 {
                println!("table builded. You did it, bro.");
                return;
            } else {
                idx += 1;
            }
        }
    }

    #[must_use]
    pub fn goto(&self, kernels: State, sym: &Term) -> Option<State> {
        println!("entries for {sym}:");
        for kernel in &kernels {
            println!("\t{kernel}");
        }
        let kernels = kernels
            .into_iter()
            .filter(|p| p.top() == Some(&sym))
            .filter_map(|p| p.clone_next())
            .collect();
        println!("goto:");
        for kernel in &kernels {
            println!("\t{kernel}");
        }
        let new = self.prop_closure(kernels);
        println!("closures:");
        for closure in &new {
            println!("\t{closure}");
        }
        if new.is_empty() {
            None
        } else {
            Some(new)
        }
    }

    #[must_use]
    pub fn decision(&self, start: Rule, pos: Position) -> Map<Term, Action> {
        if pos.can_adv() {
            todo!()
        } else {
            pos.look
                .iter()
                .map(|l| {
                    (
                        l.clone(),
                        if *l == "$" && pos.rule == start {
                            Action::Acc
                        } else {
                            Action::Reduce(pos.clone())
                        },
                    )
                })
                .collect()
        }
    }
}
