use std::{
    fmt::{Debug, Display, Write},
    rc::Rc,
};

use crate::{grammar::Production, Set};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position<T>
where
    T: Clone + PartialEq + PartialOrd + Ord + Debug,
{
    pub rule: T,
    pub seq: Rc<Production<T>>,
    pub point: usize,
    pub look: Set<T>,
}

impl<T> Position<T>
where
    T: Clone + PartialEq + PartialOrd + Ord + Debug,
{
    #[must_use]
    pub fn new(rule: T, seq: Rc<Production<T>>, point: usize, look: Set<T>) -> Self {
        Self {
            rule,
            seq,
            point,
            look,
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn with_look(self, look: Set<T>) -> Self {
        Self { look, ..self }
    }

    /// Returns the next item after current position. I.e:
    /// locus([S -> C. D f; $]) = f
    #[must_use]
    pub fn next(&self) -> Option<T> {
        self.peek(1)
    }

    #[must_use]
    pub fn peek(&self, qty: usize) -> Option<T> {
        self.seq.0.get(self.point + qty).cloned()
    }

    /// Returns the current position item. I.e:
    /// locus([S -> C. D f; $]) = D
    #[must_use]
    pub fn top(&self) -> Option<T> {
        self.peek(0)
    }

    #[must_use]
    pub fn finished(&self) -> bool {
        self.point >= self.seq.0.len()
    }

    #[must_use]
    pub fn body_eq(&self, rhs: &Self) -> bool {
        self.point == rhs.point && self.seq == rhs.seq && self.rule == rhs.rule
    }

    pub fn adv(&mut self) {
        self.point += 1;
    }

    #[must_use]
    pub fn abs_idx(&self, idx: usize) -> Option<T> {
        self.seq.0.get(idx).cloned()
    }

    #[must_use]
    pub fn no_look(&self) -> Self {
        Self::new(self.rule.clone(), self.seq.clone(), self.point, Set::new())
    }

    #[must_use]
    pub fn clone_next(&self) -> Option<Self> {
        if self.finished() {
            None
        } else {
            let mut next = self.clone();
            next.adv();
            Some(next)
        }
    }
}

impl<T> Display for Position<T>
where
    T: Clone + PartialEq + PartialOrd + Ord + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} =", self.rule))?;
        for i in 0..=self.point.max(self.seq.0.len()) {
            f.write_char(' ')?;
            if i == self.point {
                f.write_str(". ")?;
            }
            if let Some(term) = self.abs_idx(i) {
                f.write_fmt(format_args!("{term:?}"))?;
            }
        }
        f.write_fmt(format_args!(" {:?}", self.look))
    }
}

impl<T> Debug for Position<T>
where
    T: Clone + PartialEq + PartialOrd + Ord + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self}"))
    }
}
