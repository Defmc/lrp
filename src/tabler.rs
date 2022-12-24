use std::collections::HashSet;

use crate::{transitive, Grammar, Table, TermSet};

#[derive(Debug, Default)]
pub struct Tabler {
    pub grammar: Grammar,
    pub terminals: TermSet,
    pub first: Table,
    pub follow: Table,
}

impl Tabler {
    #[must_use]
    pub fn new(grammar: Grammar, terminals: TermSet) -> Self {
        let mut buf = Self {
            grammar,
            terminals,
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
            table.insert(name, HashSet::new());
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
                    if !self.is_terminal(rule[term_idx]) {
                        let entry = table.entry(rule[term_idx]).or_insert_with(HashSet::new);
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
                if !self.is_terminal(last) {
                    // A = . . . . T -> {T: FOLLOW(A)}
                    table.entry(last).or_insert_with(HashSet::new).insert(name);
                }
            }
        }
        table
    }

    pub fn proc_first(&mut self) {
        self.first = transitive(self.first.clone(), |t| self.first_step(&t));
    }

    pub fn proc_follow(&mut self) {
        self.follow = transitive(self.follow.clone(), |t| self.follow_step(&t));
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn first_step(&self, input: &Table) -> Table {
        let mut table = Table::new();
        for (name, firsts) in input {
            table.insert(name, HashSet::new());
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
            table.insert(noterm, HashSet::new());
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
}
