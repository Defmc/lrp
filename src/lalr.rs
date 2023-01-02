use crate::{Action, Grammar, Map, Parser, Position, Rule, Set, State, Tabler, Term};

pub struct Lalr {
    pub table: Tabler,
}

impl Parser for Lalr {
    fn new(grammar: Grammar) -> Self {
        let mut parser = Self {
            table: Tabler::new(grammar),
        };
        parser.proc_actions();
        parser
    }

    fn tables(&self) -> &Tabler {
        &self.table
    }

    fn tables_mut(&mut self) -> &mut Tabler {
        &mut self.table
    }

    fn closure(&self, state: State) -> State {
        let mut new_state = State::new();
        for pos in &state {
            if let Some(top) = pos.top() {
                if self.table.grammar.is_terminal(&top) {
                    continue;
                }
                for prod in self.table.grammar.rules[top].prods() {
                    new_state.insert(Position::new(top, prod.clone(), 0, Set::new()));
                }
            }
        }
        new_state.extend(state);
        new_state
    }

    fn proc_closures(&mut self) {
        self.proc_closures_first_row();
        let mut idx = 0;
        while idx < self.table.states.len() {
            let row = self.table.states[idx].clone();
            for s in self.table.grammar.symbols() {
                let (kernel, closures) = if let Some((k, c)) = self.goto(row.clone(), &s) {
                    (k, c)
                } else {
                    continue;
                };
                self.table.kernels.insert(kernel, self.table.states.len());
                self.table.states.push(closures);
            }
            idx += 1;
        }
    }

    fn decision(&self, start: Rule, pos: &Position, row: &State) -> Map<Term, Action> {
        if let Some(locus) = pos.top() {
            let filter = Tabler::sym_filter(row, &locus);
            let state = self
                .table
                .kernels
                .get(&filter)
                .expect("`kernels` is incomplete");
            if self.table.grammar.is_terminal(&locus) {
                Map::from([(locus, Action::Shift(*state))])
            } else {
                Map::from([(locus, Action::Goto(*state))])
            }
        } else {
            self.table.follow[pos.rule]
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

    fn goto(&self, kernels: State, sym: &Term) -> Option<(State, State)> {
        let kernels = Tabler::sym_filter(&kernels, sym);
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

    fn proc_actions(&mut self) {
        self.proc_closures();
        let start = self.table.basis_pos().rule;
        for row in self.table.states.iter() {
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
}

impl Lalr {
    pub fn proc_closures_first_row(&mut self) {
        let start = self.prop_closure(State::from([self.table.basis_pos()]));
        self.table.kernels.insert(State::new(), 0);
        self.table.states.push(start.clone());
    }
}