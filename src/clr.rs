use crate::{ActTable, Action, Grammar, Map, Parser, Tabler, Term};

pub struct Clr {
    pub table: Tabler,
}

impl Parser for Clr {
    fn new(grammar: Grammar) -> Self {
        let mut parser = Clr {
            table: Tabler::new(grammar),
        };
        parser.proc_actions();
        parser
    }

    fn gotos(&self) -> &ActTable {
        &self.table.actions
    }
    fn actions(&self) -> &ActTable {
        &self.table.actions
    }

    fn proc_actions(&mut self) {
        self.table.proc_closures();
        let start = self.table.basis_pos().rule;
        for row in self.table.states.iter() {
            let mut map: Map<Term, Action> = Map::new();
            for item in row {
                for (term, act) in self.table.decision(start, item, row) {
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

