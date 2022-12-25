use std::fmt::{Display, Write};

use crate::{Rule, Set, Term};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    rule: Rule,
    seq: Vec<Term>,
    point: usize,
    look: Set<Term>,
}

impl Position {
    #[must_use]
    pub fn new(rule: Rule, seq: Vec<Term>, point: usize, look: Set<Term>) -> Self {
        Self {
            rule,
            seq,
            point,
            look,
        }
    }

    #[must_use]
    pub fn rule(mut self, rule: Rule) -> Self {
        self.rule = rule;
        self
    }

    #[must_use]
    pub fn seq(mut self, seq: Vec<Term>) -> Self {
        self.seq = seq;
        self
    }

    #[must_use]
    pub fn point(mut self, point: usize) -> Self {
        self.point = point;
        self
    }

    #[must_use]
    pub fn look(mut self, look: Set<Term>) -> Self {
        self.look = look;
        self
    }

    #[must_use]
    pub fn locus(&self) -> Option<Term> {
        self.seq.get(self.point).copied()
    }

    #[must_use]
    pub fn peek(&self, qty: usize) -> Option<Term> {
        self.seq.get(self.point + qty - 1).copied()
    }

    #[must_use]
    pub fn top(&self) -> Option<Term> {
        self.seq.get(self.point.checked_sub(1)?).copied()
    }

    #[must_use]
    pub fn can_adv(&self) -> bool {
        self.point <= self.seq.len()
    }

    pub fn adv(&mut self) {
        self.point += 1;
    }

    pub fn abs_idx(&self, idx: usize) -> Option<Term> {
        self.seq.get(idx).copied()
    }
}

impl Display for Position {
    #[must_use]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} = ", self.rule))?;
        for i in 0..=self.point.max(self.seq.len()) {
            if i == self.point {
                f.write_str(". ")?;
            }
            if let Some(term) = self.abs_idx(i) {
                f.write_str(term)?;
            }
            f.write_char(' ')?;
        }
        f.write_fmt(format_args!("{:?}", self.look))
    }
}
