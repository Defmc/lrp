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
    pub kernels: Map<State, usize>,
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
        println!("{:?}", self.follow);
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
                if !self.is_terminal(top) {
                    let look = if let Some(locus) = pos.locus() {
                        if self.is_terminal(locus) {
                            Set::from([locus])
                        } else {
                            self.first[locus].clone()
                        }
                    } else {
                        if pos.point == 0 {
                            self.follow[&pos.rule].clone()
                        } else {
                            pos.look.clone()
                        }
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
                self.kernels.insert(kernel, self.states.len());
                self.states.push(closures);
            }
            idx += 1;
        }
    }

    pub fn proc_closures_first_row(&mut self, start: Position) {
        let start = self.prop_closure(State::from([start]));
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
        if let Some(state) = self.kernels.get(&kernels) {
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
        for row in &self.states {
            let mut map: Map<Term, Action> = Map::new();
            for item in row {
                for (term, act) in self.decision(start, item) {
                    if map.contains_key(term) {
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
            let next = pos.clone_next().unwrap();
            let state = self
                .kernels
                .iter()
                .find_map(|(k, &s)| if k.contains(&next) { Some(s) } else { None })
                .expect("`kernels` is incomplete");
            if self.is_terminal(locus) {
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
                            Action::Reduce(pos.clone())
                        },
                    )
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Map, Set, Tabler};
    use pretty_assertions::assert_eq;

    #[test]
    pub fn firsts() {
        let grammar = crate::grammar! {
            "S" -> "C" "C",
            "C" -> "c" "C"
                | "d"
        };
        let tabler = Tabler::new(grammar, Set::from(["c", "d", "$"]));
        assert_eq!(
            tabler.first,
            Map::from([("S", Set::from(["c", "d"])), ("C", Set::from(["c", "d"]))])
        );

        let grammar = crate::grammar! {};
    }
}
