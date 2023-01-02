use std::{fmt, iter::Peekable, rc::Rc};

use crate::{ActTable, Production, Rule, RuleName, Term};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Shift(usize),
    Goto(usize),
    Reduce(RuleName, Rc<Production>),
    Acc,
    Conflict(Box<Action>, Box<Action>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    Simple(Rule),
    Compound(Rule, Vec<Item>),
}

impl Item {
    #[must_use]
    pub const fn name(&self) -> Rule {
        match self {
            Self::Simple(n) | Self::Compound(n, ..) => n,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())?;
        if let Self::Compound(_, elms) = self {
            f.write_fmt(format_args!(
                " -> ({})",
                elms.iter()
                    .map(|f| format!("{f}"))
                    .collect::<Vec<String>>()
                    .join(" ")
            ))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackEl {
    Item(Item),
    State(usize),
}

impl fmt::Display for StackEl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Item(it) => f.write_fmt(format_args!("{it}")),
            Self::State(n) => f.write_fmt(format_args!("{n}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dfa<I: Iterator<Item = Term>> {
    pub buffer: Peekable<I>,
    pub stack: Vec<StackEl>,
    pub table: ActTable,
    pub top: usize,
    pub finished: Option<bool>,
}

impl<I: Iterator<Item = Term>> Dfa<I> {
    #[must_use]
    pub fn new(buffer: I, table: ActTable) -> Self {
        Self {
            stack: vec![StackEl::State(0)],
            buffer: buffer.peekable(),
            top: 0,
            table,
            finished: None,
        }
    }

    /// # Panics
    /// When buffer is empty
    pub fn shift(&mut self, to: usize) {
        let item = Item::Simple(self.buffer.next().unwrap());
        self.stack.push(StackEl::Item(item));
        self.top = to;
        self.stack.push(StackEl::State(self.top));
    }

    pub fn accept(&mut self) {
        self.finished = Some(self.buffer.peek() == Some(&"$"));
    }

    pub fn start(&mut self) {
        self.trace(|_| {});
    }

    pub fn trace(&mut self, mut f: impl FnMut(&mut Self)) {
        while self.finished.is_none() {
            f(self);
            let symbol = *self.buffer.peek().unwrap_or(&crate::EOF);
            self.travel(symbol);
        }
    }

    pub fn goto(&mut self, to: usize) {
        self.stack.push(StackEl::State(to));
        self.top = to;
    }

    /// # Panics
    /// When trying to execute a conflicting action
    pub fn travel(&mut self, symbol: Term) {
        match &self.table[self.top][symbol] {
            Action::Shift(to) => self.shift(*to),
            Action::Goto(to) => self.goto(*to),
            Action::Reduce(name, prod) => self.reduce(name, &prod.clone()),
            Action::Acc => self.accept(),
            Action::Conflict(a, b) => panic!("conflict between {a:?} and {b:?}"),
        }
    }

    /// # Panics
    /// When missing state shifts
    pub fn reduce(&mut self, name: RuleName, prod: &Rc<Production>) {
        let mut item = Vec::with_capacity(prod.len());
        while item.len() != prod.len() {
            let poped = self.stack.pop().unwrap();
            if let StackEl::Item(i) = poped {
                item.push(i);
            }
        }
        let item = Item::Compound(name, item);
        let state = self
            .stack
            .iter()
            .rev()
            .find_map(|i| {
                if let StackEl::State(n) = i {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap();
        self.stack.push(StackEl::Item(item));
        self.top = state;
        self.travel(name);
    }

    pub fn reset(&mut self) {
        self.finished = None;
        self.stack.clear();
        self.stack.push(StackEl::State(0));
        self.top = 0;
    }

    #[must_use]
    pub fn parse(&mut self, input: I) -> StackEl {
        self.reset();
        self.buffer = input.peekable();
        self.start();
        let res = self.stack[1].clone();
        self.reset();
        res
    }

    #[must_use]
    pub fn stack_fmt(&self) -> String {
        self.stack
            .iter()
            .map(|f| format!("{f}"))
            .collect::<Vec<String>>()
            .join(" ")
    }
}
