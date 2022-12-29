use std::{
    fmt::{Debug, Display, Write},
    rc::Rc,
};

use crate::{grammar::Production, Rule, Set, Term};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub rule: Rule,
    pub seq: Rc<Production>,
    pub point: usize,
    pub look: Set<Term>,
}

impl Position {
    #[must_use]
    pub fn new(rule: Rule, seq: Rc<Production>, point: usize, look: Set<Term>) -> Self {
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
    pub fn seq(mut self, seq: Rc<Production>) -> Self {
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
        self.peek(1)
    }

    #[must_use]
    pub fn peek(&self, qty: usize) -> Option<Term> {
        self.seq.get(self.point + qty).copied()
    }

    #[must_use]
    pub fn top(&self) -> Option<Term> {
        self.seq.get(self.point).copied()
    }

    #[must_use]
    pub fn next_and_look(&self) -> Set<Term> {
        self.locus()
            .iter()
            .chain(self.look.iter())
            .copied()
            .collect()
    }

    #[must_use]
    pub fn finished(&self) -> bool {
        self.point >= self.seq.len()
    }

    #[must_use]
    pub fn body_eq(&self, rhs: &Self) -> bool {
        self.point == rhs.point && self.seq == rhs.seq && self.rule == rhs.rule
    }

    pub fn adv(&mut self) {
        self.point += 1;
    }

    #[must_use]
    pub fn abs_idx(&self, idx: usize) -> Option<Term> {
        self.seq.get(idx).copied()
    }

    #[must_use]
    pub fn clone_next(&self) -> Option<Self> {
        if !self.finished() {
            let mut next = self.clone();
            next.adv();
            Some(next)
        } else {
            None
        }
    }
}

impl Display for Position {
    #[must_use]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} =", self.rule))?;
        for i in 0..=self.point.max(self.seq.len()) {
            f.write_char(' ')?;
            if i == self.point {
                f.write_str(". ")?;
            }
            if let Some(term) = self.abs_idx(i) {
                f.write_str(term)?;
            }
        }
        f.write_fmt(format_args!(" {:?}", self.look))
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self}"))
    }
}
